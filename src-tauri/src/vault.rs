//! Vault storage format, header serialization, manifest structures, and blob encoding.
//!
//! Implements architectural decisions established in ADR 0002 and VAULT-FORMAT.md:
//! - Unencrypted `header` verifying magic ("MAGRATHEA"), version 1, and Argon2id parameters.
//! - Envelope encryption: Passphrase Slot 0 and Recovery Key Slot 1 wrapping the DEK.
//! - Encrypted `manifest.blob` with monotonic versioning.
//! - Encrypted content payloads in `blobs/` bound by AAD (`MAGRATHEA-V1:blob:<blob_id>`).

use crate::crypto::{
    decrypt_payload, derive_key, encrypt_payload, generate_salt, unwrap_key, wrap_key,
    Argon2Params, CryptoError, Nonce, SecretKey, DEFAULT_M_COST, DEFAULT_P_COST, DEFAULT_T_COST,
    NONCE_LEN, SALT_LEN,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Expected magic identifier in the vault header.
pub const VAULT_MAGIC: &str = "MAGRATHEA";

/// Current vault storage format version.
pub const CURRENT_FORMAT_VERSION: u16 = 1;

/// Minimum allowed Argon2id memory cost in KiB to prevent downgrade attacks (64 MiB).
pub const MIN_ARGON2_MEMORY_KIB: u32 = 65536;

/// Minimum allowed Argon2id iterations to prevent downgrade attacks.
pub const MIN_ARGON2_ITERATIONS: u32 = 3;

/// Erros decorrentes de operações no cofre e manipulação de arquivos.
#[derive(Debug, PartialEq, Eq)]
pub enum VaultError {
    /// Bytes mágicos do cabeçalho não conferem com "MAGRATHEA".
    InvalidMagic(String),
    /// Versão do formato é mais recente do que a suportada pelo software.
    UnsupportedVersion(u16),
    /// Parâmetros KDF inseguros detectados no cabeçalho.
    InsecureKdfParameters(String),
    /// Falha em operação criptográfica (chave incorreta, MAC inválido ou payload corrompido).
    Crypto(String),
    /// Erro de serialização ou desserialização JSON.
    Serialization(String),
    /// Payload de blob mais curto que o tamanho mínimo de nonce.
    CorruptedBlob,
    /// Mismatch de Vault ID durante autenticação do manifesto.
    VaultIdMismatch,
    /// Cofre encontra-se bloqueado em memória.
    Locked,
    /// Erro de entrada e saída (I/O) no sistema de arquivos.
    Io(String),
    /// Blob referenciado no manifesto não foi encontrado no disco.
    MissingBlob(String),
    /// Tentativa de ataque de rollback detectada no manifesto.
    RollbackDetected { expected_min: u64, on_disk: u64 },
    /// Cofre já existe no diretório informado.
    AlreadyExists(String),
    /// Arquivo ou diretório do cofre não foi encontrado.
    NotFound(String),
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic(m) => write!(f, "Invalid vault magic identifier: '{m}'"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported vault format version: {v}"),
            Self::InsecureKdfParameters(msg) => write!(f, "Insecure KDF parameters: {msg}"),
            Self::Crypto(msg) => write!(f, "Cryptographic operation failed: {msg}"),
            Self::Serialization(msg) => write!(f, "Vault serialization error: {msg}"),
            Self::CorruptedBlob => write!(f, "Corrupted blob payload: missing nonce or truncated"),
            Self::VaultIdMismatch => write!(f, "Vault ID mismatch in manifest AAD"),
            Self::Locked => write!(f, "Cofre está bloqueado em memória"),
            Self::Io(msg) => write!(f, "Erro de I/O no disco: {msg}"),
            Self::MissingBlob(id) => write!(f, "Blob '{id}' referenciado no manifesto não existe"),
            Self::RollbackDetected { expected_min, on_disk } => write!(
                f,
                "Ataque de rollback detectado: versão em disco ({on_disk}) é conflitante com a esperada ({expected_min})"
            ),
            Self::AlreadyExists(p) => write!(f, "Cofre já existe em: '{p}'"),
            Self::NotFound(p) => write!(f, "Cofre ou arquivo não encontrado: '{p}'"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<CryptoError> for VaultError {
    fn from(err: CryptoError) -> Self {
        Self::Crypto(err.to_string())
    }
}

impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Serialized KDF parameters recorded in the vault header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParams {
    /// KDF algorithm identifier (must be "argon2id").
    pub algorithm: String,
    /// Argon2 version (must be 19).
    pub version: u32,
    /// Memory cost in KiB.
    pub memory_kib: u32,
    /// Iterations.
    pub iterations: u32,
    /// Parallelism lanes.
    pub parallelism: u32,
    /// Salt encoded as hexadecimal string (32 hex characters = 16 bytes).
    pub salt_hex: String,
}

/// A wrapped key slot holding a 192-bit nonce and wrapped DEK ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeySlot {
    /// Nonce encoded as hexadecimal (48 hex chars = 24 bytes).
    pub nonce_hex: String,
    /// Wrapped key bytes with Poly1305 MAC tag encoded as hex (96 hex chars = 48 bytes).
    pub wrapped_dek_hex: String,
}

/// Vault header stored in plaintext at the root of a `.magrathea` vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultHeader {
    /// Magic identifier (must be "MAGRATHEA").
    pub magic: String,
    /// Format version number.
    pub format_version: u16,
    /// Unique UUID of this vault.
    pub vault_id: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// KDF parameters for password derivation.
    pub kdf_params: KdfParams,
    /// Slot 0: DEK wrapped with master passphrase KEK.
    pub slot_passphrase: KeySlot,
    /// Slot 1: DEK wrapped with recovery key KEK.
    pub slot_recovery: KeySlot,
}

impl VaultHeader {
    /// Validates header fields against security constraints.
    pub fn validate(&self) -> Result<(), VaultError> {
        if self.magic != VAULT_MAGIC {
            return Err(VaultError::InvalidMagic(self.magic.clone()));
        }
        if self.format_version > CURRENT_FORMAT_VERSION {
            return Err(VaultError::UnsupportedVersion(self.format_version));
        }
        if self.kdf_params.algorithm != "argon2id" {
            return Err(VaultError::InsecureKdfParameters(format!(
                "unsupported algorithm: {}",
                self.kdf_params.algorithm
            )));
        }
        if self.kdf_params.memory_kib < MIN_ARGON2_MEMORY_KIB {
            return Err(VaultError::InsecureKdfParameters(format!(
                "memory cost {} KiB is lower than minimum allowed {} KiB",
                self.kdf_params.memory_kib, MIN_ARGON2_MEMORY_KIB
            )));
        }
        if self.kdf_params.iterations < MIN_ARGON2_ITERATIONS {
            return Err(VaultError::InsecureKdfParameters(format!(
                "iterations {} is lower than minimum allowed {}",
                self.kdf_params.iterations, MIN_ARGON2_ITERATIONS
            )));
        }
        Ok(())
    }

    /// Serializes this header to canonical JSON.
    pub fn to_json(&self) -> Result<String, VaultError> {
        serde_json::to_string_pretty(self).map_err(|e| VaultError::Serialization(e.to_string()))
    }

    /// Deserializes and validates a header from JSON.
    pub fn from_json(json_str: &str) -> Result<Self, VaultError> {
        let header: Self =
            serde_json::from_str(json_str).map_err(|e| VaultError::Serialization(e.to_string()))?;
        header.validate()?;
        Ok(header)
    }

    /// Unlocks the vault by decrypting Slot 0 with the author's passphrase.
    pub fn unlock_with_passphrase(&self, passphrase: &[u8]) -> Result<SecretKey, VaultError> {
        self.validate()?;

        let salt_bytes = hex_decode(&self.kdf_params.salt_hex)
            .map_err(|e| VaultError::Serialization(format!("bad salt hex: {e}")))?;
        if salt_bytes.len() != SALT_LEN {
            return Err(VaultError::Serialization("invalid salt length".into()));
        }
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&salt_bytes);

        let params = Argon2Params {
            memory_kib: self.kdf_params.memory_kib,
            iterations: self.kdf_params.iterations,
            parallelism: self.kdf_params.parallelism,
        };

        let kek = derive_key(passphrase, &salt, &params)?;

        let nonce_bytes = hex_decode(&self.slot_passphrase.nonce_hex)
            .map_err(|e| VaultError::Serialization(format!("bad nonce hex: {e}")))?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(VaultError::Serialization("invalid nonce length".into()));
        }
        let mut nonce_arr = [0u8; NONCE_LEN];
        nonce_arr.copy_from_slice(&nonce_bytes);
        let nonce = Nonce::new(nonce_arr);

        let wrapped_dek = hex_decode(&self.slot_passphrase.wrapped_dek_hex)
            .map_err(|e| VaultError::Serialization(format!("bad wrapped dek hex: {e}")))?;

        let slot_aad = format!("MAGRATHEA-V1:slot:0:{}", self.vault_id);
        let dek = unwrap_key(&kek, &nonce, &wrapped_dek, slot_aad.as_bytes())?;
        Ok(dek)
    }

    /// Unlocks the vault by decrypting Slot 1 with the recovery key.
    pub fn unlock_with_recovery_key(
        &self,
        recovery_key: &SecretKey,
    ) -> Result<SecretKey, VaultError> {
        self.validate()?;

        let nonce_bytes = hex_decode(&self.slot_recovery.nonce_hex)
            .map_err(|e| VaultError::Serialization(format!("bad nonce hex: {e}")))?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(VaultError::Serialization("invalid nonce length".into()));
        }
        let mut nonce_arr = [0u8; NONCE_LEN];
        nonce_arr.copy_from_slice(&nonce_bytes);
        let nonce = Nonce::new(nonce_arr);

        let wrapped_dek = hex_decode(&self.slot_recovery.wrapped_dek_hex)
            .map_err(|e| VaultError::Serialization(format!("bad wrapped dek hex: {e}")))?;

        let slot_aad = format!("MAGRATHEA-V1:slot:1:{}", self.vault_id);
        let dek = unwrap_key(recovery_key, &nonce, &wrapped_dek, slot_aad.as_bytes())?;
        Ok(dek)
    }

    /// Altera a frase-senha mestra re-envelopando a DEK no Slot 0 com uma nova KEK.
    pub fn change_passphrase(
        &mut self,
        current_passphrase: &[u8],
        new_passphrase: &[u8],
        custom_params: Option<Argon2Params>,
    ) -> Result<(), VaultError> {
        let dek = self.unlock_with_passphrase(current_passphrase)?;

        let params = custom_params.unwrap_or(Argon2Params {
            memory_kib: self.kdf_params.memory_kib,
            iterations: self.kdf_params.iterations,
            parallelism: self.kdf_params.parallelism,
        });

        let new_salt = generate_salt();
        let new_kek = derive_key(new_passphrase, &new_salt, &params)?;

        let slot0_aad = format!("MAGRATHEA-V1:slot:0:{}", self.vault_id);
        let (slot0_nonce, slot0_wrapped) = wrap_key(&new_kek, &dek, slot0_aad.as_bytes())?;

        self.kdf_params.salt_hex = hex_encode(&new_salt);
        self.kdf_params.memory_kib = params.memory_kib;
        self.kdf_params.iterations = params.iterations;
        self.kdf_params.parallelism = params.parallelism;

        self.slot_passphrase.nonce_hex = hex_encode(slot0_nonce.as_bytes());
        self.slot_passphrase.wrapped_dek_hex = hex_encode(&slot0_wrapped);

        Ok(())
    }

    /// Re-envelopa uma nova DEK em ambos os Slots 0 e 1 durante o processo de recifragem integral do cofre.
    pub fn rekey(
        &mut self,
        new_dek: &SecretKey,
        passphrase: &[u8],
        new_recovery_key: &SecretKey,
        custom_params: Option<Argon2Params>,
    ) -> Result<(), VaultError> {
        let params = custom_params.unwrap_or(Argon2Params {
            memory_kib: self.kdf_params.memory_kib,
            iterations: self.kdf_params.iterations,
            parallelism: self.kdf_params.parallelism,
        });

        let new_salt = generate_salt();
        let new_kek = derive_key(passphrase, &new_salt, &params)?;

        let slot0_aad = format!("MAGRATHEA-V1:slot:0:{}", self.vault_id);
        let (slot0_nonce, slot0_wrapped) = wrap_key(&new_kek, new_dek, slot0_aad.as_bytes())?;

        let slot1_aad = format!("MAGRATHEA-V1:slot:1:{}", self.vault_id);
        let (slot1_nonce, slot1_wrapped) =
            wrap_key(new_recovery_key, new_dek, slot1_aad.as_bytes())?;

        self.kdf_params.salt_hex = hex_encode(&new_salt);
        self.kdf_params.memory_kib = params.memory_kib;
        self.kdf_params.iterations = params.iterations;
        self.kdf_params.parallelism = params.parallelism;

        self.slot_passphrase.nonce_hex = hex_encode(slot0_nonce.as_bytes());
        self.slot_passphrase.wrapped_dek_hex = hex_encode(&slot0_wrapped);

        self.slot_recovery.nonce_hex = hex_encode(slot1_nonce.as_bytes());
        self.slot_recovery.wrapped_dek_hex = hex_encode(&slot1_wrapped);

        Ok(())
    }
}

/// Creates a new vault initialization package: header and unencrypted DEK.
pub fn initialize_vault(
    vault_id: &str,
    created_at_iso: &str,
    passphrase: &[u8],
    recovery_key: &SecretKey,
    custom_params: Option<Argon2Params>,
) -> Result<(VaultHeader, SecretKey), VaultError> {
    let params = custom_params.unwrap_or(Argon2Params {
        memory_kib: DEFAULT_M_COST,
        iterations: DEFAULT_T_COST,
        parallelism: DEFAULT_P_COST,
    });

    let salt = generate_salt();
    let passphrase_kek = derive_key(passphrase, &salt, &params)?;
    let dek = SecretKey::generate();

    // Wrap Slot 0 (Passphrase)
    let slot0_aad = format!("MAGRATHEA-V1:slot:0:{vault_id}");
    let (slot0_nonce, slot0_wrapped) = wrap_key(&passphrase_kek, &dek, slot0_aad.as_bytes())?;

    // Wrap Slot 1 (Recovery Key)
    let slot1_aad = format!("MAGRATHEA-V1:slot:1:{vault_id}");
    let (slot1_nonce, slot1_wrapped) = wrap_key(recovery_key, &dek, slot1_aad.as_bytes())?;

    let header = VaultHeader {
        magic: VAULT_MAGIC.to_string(),
        format_version: CURRENT_FORMAT_VERSION,
        vault_id: vault_id.to_string(),
        created_at: created_at_iso.to_string(),
        kdf_params: KdfParams {
            algorithm: "argon2id".to_string(),
            version: 19,
            memory_kib: params.memory_kib,
            iterations: params.iterations,
            parallelism: params.parallelism,
            salt_hex: hex_encode(&salt),
        },
        slot_passphrase: KeySlot {
            nonce_hex: hex_encode(slot0_nonce.as_bytes()),
            wrapped_dek_hex: hex_encode(&slot0_wrapped),
        },
        slot_recovery: KeySlot {
            nonce_hex: hex_encode(slot1_nonce.as_bytes()),
            wrapped_dek_hex: hex_encode(&slot1_wrapped),
        },
    };

    Ok((header, dek))
}

/// An item within the manuscript hierarchy in the encrypted manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestItem {
    /// Item UUID.
    pub id: String,
    /// Node kind ("part", "chapter", "scene", "entity").
    pub kind: String,
    /// Parent node UUID (None for root parts).
    pub parent_id: Option<String>,
    /// Human-readable title.
    pub title: String,
    /// Normalized logical slug for `[[wikilink]]` resolution.
    pub slug: String,
    /// Opaque blob UUID in `blobs/` containing document text.
    pub blob_id: Option<String>,
    /// Sorting order within parent.
    pub order: u32,
    /// Associated metadata tags.
    pub tags: Vec<String>,
    /// Workflow status ("draft", "revised", "final", etc.).
    pub status: Option<String>,
}

/// Decrypted structure of `manifest.blob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultManifest {
    /// ID of the parent vault.
    pub vault_id: String,
    /// Title of the novel or literary project.
    pub project_title: String,
    /// Strictly increasing monotonic version counter against rollback attacks.
    pub monotonic_version: u64,
    /// ISO timestamp of creation.
    pub created_at: String,
    /// ISO timestamp of last modification.
    pub updated_at: String,
    /// Tree items in the manuscript.
    pub items: Vec<ManifestItem>,
    /// Internal encrypted trash items.
    pub trash: Vec<ManifestItem>,
}

impl VaultManifest {
    /// Creates a new empty manifest for a newly initialized project.
    pub fn new(vault_id: &str, project_title: &str, timestamp_iso: &str) -> Self {
        Self {
            vault_id: vault_id.to_string(),
            project_title: project_title.to_string(),
            monotonic_version: 1,
            created_at: timestamp_iso.to_string(),
            updated_at: timestamp_iso.to_string(),
            items: Vec::new(),
            trash: Vec::new(),
        }
    }

    /// Increments monotonic version and updates timestamp.
    pub fn bump_version(&mut self, timestamp_iso: &str) {
        self.monotonic_version = self.monotonic_version.saturating_add(1);
        self.updated_at = timestamp_iso.to_string();
    }

    /// Encrypts this manifest into a binary blob payload using the vault DEK.
    pub fn encrypt(&self, dek: &SecretKey) -> Result<Vec<u8>, VaultError> {
        let json_bytes =
            serde_json::to_vec(self).map_err(|e| VaultError::Serialization(e.to_string()))?;
        let nonce = Nonce::generate();
        let aad = format!("MAGRATHEA-V1:manifest:{}", self.vault_id);
        let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &json_bytes)?;

        // Payload format: [24 bytes Nonce] || [Ciphertext + Tag]
        let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        blob.extend_from_slice(nonce.as_bytes());
        blob.extend_from_slice(&ciphertext);
        Ok(blob)
    }

    /// Decrypts and deserializes a manifest blob using the vault DEK.
    pub fn decrypt(
        dek: &SecretKey,
        expected_vault_id: &str,
        blob: &[u8],
    ) -> Result<Self, VaultError> {
        if blob.len() < NONCE_LEN {
            return Err(VaultError::CorruptedBlob);
        }

        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes.copy_from_slice(&blob[..NONCE_LEN]);
        let nonce = Nonce::new(nonce_bytes);
        let ciphertext = &blob[NONCE_LEN..];

        let aad = format!("MAGRATHEA-V1:manifest:{expected_vault_id}");
        let decrypted_bytes = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

        let manifest: Self = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;

        if manifest.vault_id != expected_vault_id {
            return Err(VaultError::VaultIdMismatch);
        }

        Ok(manifest)
    }
}

/// Encrypts an individual content scene into a binary blob bound by blob_id AAD.
pub fn encrypt_blob(
    dek: &SecretKey,
    blob_id: &str,
    plaintext: &[u8],
) -> Result<Vec<u8>, VaultError> {
    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:blob:{blob_id}");
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), plaintext)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_bytes());
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

/// Decrypts an individual content blob and verifies AAD binding.
pub fn decrypt_blob(
    dek: &SecretKey,
    blob_id: &str,
    blob_bytes: &[u8],
) -> Result<Vec<u8>, VaultError> {
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let aad = format!("MAGRATHEA-V1:blob:{blob_id}");
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;
    Ok(plaintext)
}

/// Simple hexadecimal encoder.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Simple hexadecimal decoder.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("hex string length must be even".to_string());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte_str = &s[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("invalid hex char '{byte_str}': {e}"))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Relatório da varredura de integridade e detecção de arquivos órfãos/ausentes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    /// Total de nós declarados no manifesto (itens ativos + lixeira).
    pub total_items: usize,
    /// Quantidade de blobs que foram decifrados e autenticados com sucesso.
    pub valid_blobs: usize,
    /// Lista de IDs de blobs corrompidos ou adulterados (falha de tag Poly1305 ou AAD).
    pub corrupted_blobs: Vec<String>,
    /// Lista de IDs de blobs referenciados no manifesto que não existem no disco.
    pub missing_blobs: Vec<String>,
    /// Lista de IDs de blobs presentes na pasta `blobs/` que não estão no manifesto.
    pub orphan_blobs: Vec<String>,
}

impl IntegrityReport {
    /// Retorna verdadeiro se o cofre estiver 100% íntegro sem blobs ausentes ou corrompidos.
    pub fn is_clean(&self) -> bool {
        self.corrupted_blobs.is_empty() && self.missing_blobs.is_empty()
    }
}

/// Grava dados em disco de forma estritamente atômica utilizando arquivo temporário, fsync e renomeação.
pub fn atomic_write(target_path: &Path, data: &[u8]) -> Result<(), VaultError> {
    let parent = target_path.parent().ok_or_else(|| {
        VaultError::Io("Diretório pai inválido para gravação atômica".to_string())
    })?;
    fs::create_dir_all(parent).map_err(|e| VaultError::Io(e.to_string()))?;

    let file_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let random_suffix: u64 = rand::random();
    let tmp_name = format!(".{file_name}.{random_suffix}.tmp");
    let tmp_path = parent.join(tmp_name);

    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(|e| VaultError::Io(e.to_string()))?;

        file.write_all(data)
            .map_err(|e| VaultError::Io(e.to_string()))?;
        file.sync_all().map_err(|e| VaultError::Io(e.to_string()))?;
    }

    fs::rename(&tmp_path, target_path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        VaultError::Io(e.to_string())
    })?;

    Ok(())
}

/// Gera timestamp ISO 8601 UTC a partir de SystemTime sem dependências externas.
pub fn current_timestamp_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{seconds:02}Z")
}

/// Instância de um cofre aberto e ativo em memória.
///
/// Protege a chave DEK e dados sensíveis através de `ZeroizeOnDrop` e bloqueio explícito.
pub struct UnlockedVault {
    root_path: PathBuf,
    header: VaultHeader,
    dek: Option<SecretKey>,
    manifest: Option<VaultManifest>,
    is_locked: bool,
}

impl fmt::Debug for UnlockedVault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnlockedVault")
            .field("root_path", &self.root_path)
            .field("header", &self.header)
            .field(
                "dek",
                &if self.dek.is_some() {
                    "[PRESENT]"
                } else {
                    "[NONE]"
                },
            )
            .field("manifest", &self.manifest)
            .field("is_locked", &self.is_locked)
            .finish()
    }
}

impl UnlockedVault {
    /// Cria uma nova instância de cofre desbloqueado.
    pub fn new(
        root_path: PathBuf,
        header: VaultHeader,
        dek: SecretKey,
        manifest: VaultManifest,
    ) -> Self {
        Self {
            root_path,
            header,
            dek: Some(dek),
            manifest: Some(manifest),
            is_locked: false,
        }
    }

    /// Retorna o caminho raiz do diretório `.magrathea`.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Retorna referência imutável ao cabeçalho público do cofre.
    pub fn header(&self) -> &VaultHeader {
        &self.header
    }

    /// Retorna referência mutável ao cabeçalho público do cofre.
    pub fn header_mut(&mut self) -> &mut VaultHeader {
        &mut self.header
    }

    /// Obtém referência imutável à chave DEK ativa em memória.
    pub fn dek(&self) -> Result<&SecretKey, VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        self.dek.as_ref().ok_or(VaultError::Locked)
    }

    /// Grava o cabeçalho público no arquivo `header` de forma atômica.
    pub fn save_header(&self) -> Result<(), VaultError> {
        let header_path = self.root_path.join("header");
        let header_json = serde_json::to_string_pretty(&self.header)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;
        atomic_write(&header_path, header_json.as_bytes())?;
        Ok(())
    }

    /// Recifra integralmente todos os blobs (cenas ativas e lixeira) e o manifesto com uma nova DEK.
    pub fn reencrypt_all(
        &mut self,
        new_dek: SecretKey,
        timestamp_iso: &str,
    ) -> Result<(), VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let old_dek = self.dek.as_ref().ok_or(VaultError::Locked)?;
        let manifest = self.manifest.as_ref().ok_or(VaultError::Locked)?;

        let mut blob_ids = Vec::new();
        for it in manifest.items.iter().chain(manifest.trash.iter()) {
            if let Some(ref bid) = it.blob_id {
                blob_ids.push(bid.clone());
            }
        }

        // 1. Recifra cada blob individual
        for bid in &blob_ids {
            let blob_path = self.root_path.join("blobs").join(format!("{bid}.blob"));
            if blob_path.exists() {
                let old_bytes = fs::read(&blob_path).map_err(|e| VaultError::Io(e.to_string()))?;
                let plaintext = decrypt_blob(old_dek, bid, &old_bytes)?;
                let new_blob = encrypt_blob(&new_dek, bid, &plaintext)?;
                atomic_write(&blob_path, &new_blob)?;
            }
        }

        // 2. Incrementa e recifra o manifesto
        let manifest_mut = self.manifest.as_mut().unwrap();
        manifest_mut.bump_version(timestamp_iso);
        let manifest_path = self.root_path.join("manifest.blob");
        let new_manifest_bytes = manifest_mut.encrypt(&new_dek)?;
        atomic_write(&manifest_path, &new_manifest_bytes)?;

        // 3. Atualiza chave DEK em memória
        self.dek = Some(new_dek);

        Ok(())
    }

    /// Verifica se o cofre está bloqueado.
    pub fn is_locked(&self) -> bool {
        self.is_locked || self.dek.is_none()
    }

    /// Bloqueia o cofre imediatamente, zeroizando e descartando a DEK da memória.
    pub fn lock(&mut self) {
        self.is_locked = true;
        self.dek = None; // Descarta SecretKey, disparando zeroização do ZeroizeOnDrop
        self.manifest = None;
    }

    /// Obtém referência imutável ao manifesto decifrado em memória.
    pub fn manifest(&self) -> Result<&VaultManifest, VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        self.manifest.as_ref().ok_or(VaultError::Locked)
    }

    /// Obtém referência mutável ao manifesto decifrado em memória.
    pub fn manifest_mut(&mut self) -> Result<&mut VaultManifest, VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        self.manifest.as_mut().ok_or(VaultError::Locked)
    }

    /// Salva o manifesto em disco de forma atômica com validação de rollback e incremento de versão.
    pub fn save_manifest(&mut self, timestamp_iso: &str) -> Result<(), VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let dek = self.dek.as_ref().ok_or(VaultError::Locked)?;
        let manifest_path = self.root_path.join("manifest.blob");

        // Detecção de rollback: verifica se o arquivo em disco possui versão superior à esperada
        if manifest_path.exists() {
            let disk_bytes = fs::read(&manifest_path).map_err(|e| VaultError::Io(e.to_string()))?;
            if let Ok(disk_manifest) =
                VaultManifest::decrypt(dek, &self.header.vault_id, &disk_bytes)
            {
                let current_version = self.manifest.as_ref().unwrap().monotonic_version;
                if disk_manifest.monotonic_version > current_version {
                    return Err(VaultError::RollbackDetected {
                        expected_min: current_version,
                        on_disk: disk_manifest.monotonic_version,
                    });
                }
            }
        }

        // Incrementa contador monotônico e atualiza timestamp
        let manifest = self.manifest.as_mut().unwrap();
        manifest.bump_version(timestamp_iso);

        // Cifra e grava atomicamente
        let encrypted_blob = manifest.encrypt(dek)?;
        atomic_write(&manifest_path, &encrypted_blob)?;

        Ok(())
    }

    /// Lê e decifra uma cena individual especificada pelo `blob_id`.
    pub fn read_blob(&self, blob_id: &str) -> Result<Vec<u8>, VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let dek = self.dek.as_ref().ok_or(VaultError::Locked)?;
        let blob_path = self.root_path.join("blobs").join(format!("{blob_id}.blob"));

        if !blob_path.exists() {
            return Err(VaultError::MissingBlob(blob_id.to_string()));
        }

        let blob_bytes = fs::read(&blob_path).map_err(|e| VaultError::Io(e.to_string()))?;
        decrypt_blob(dek, blob_id, &blob_bytes)
    }

    /// Cifra e grava atomicamente uma cena individual no diretório `blobs/`.
    pub fn write_blob(&self, blob_id: &str, plaintext: &[u8]) -> Result<(), VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let dek = self.dek.as_ref().ok_or(VaultError::Locked)?;
        let blob_path = self.root_path.join("blobs").join(format!("{blob_id}.blob"));

        let encrypted_blob = encrypt_blob(dek, blob_id, plaintext)?;
        atomic_write(&blob_path, &encrypted_blob)?;

        Ok(())
    }

    /// Exclui um arquivo de blob do disco.
    pub fn delete_blob(&self, blob_id: &str) -> Result<(), VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let blob_path = self.root_path.join("blobs").join(format!("{blob_id}.blob"));
        if blob_path.exists() {
            fs::remove_file(blob_path).map_err(|e| VaultError::Io(e.to_string()))?;
        }
        Ok(())
    }

    /// Realiza varredura integral de integridade criptográfica e detecção de arquivos órfãos/ausentes.
    pub fn verify_integrity(&self) -> Result<IntegrityReport, VaultError> {
        if self.is_locked() {
            return Err(VaultError::Locked);
        }
        let dek = self.dek.as_ref().ok_or(VaultError::Locked)?;
        let manifest = self.manifest.as_ref().ok_or(VaultError::Locked)?;

        let mut referenced_blobs = std::collections::HashSet::new();
        let mut total_items = 0;
        let mut valid_blobs = 0;
        let mut corrupted_blobs = Vec::new();
        let mut missing_blobs = Vec::new();

        // 1. Coleta e valida todos os blobs referenciados em itens e lixeira
        for item in manifest.items.iter().chain(manifest.trash.iter()) {
            total_items += 1;
            if let Some(ref bid) = item.blob_id {
                referenced_blobs.insert(bid.clone());
                let blob_path = self.root_path.join("blobs").join(format!("{bid}.blob"));
                if !blob_path.exists() {
                    missing_blobs.push(bid.clone());
                    continue;
                }
                match fs::read(&blob_path) {
                    Ok(bytes) => match decrypt_blob(dek, bid, &bytes) {
                        Ok(_) => valid_blobs += 1,
                        Err(_) => corrupted_blobs.push(bid.clone()),
                    },
                    Err(_) => corrupted_blobs.push(bid.clone()),
                }
            }
        }

        // 2. Varre o diretório blobs/ para identificar arquivos órfãos
        let mut orphan_blobs = Vec::new();
        let blobs_dir = self.root_path.join("blobs");
        if blobs_dir.exists() {
            if let Ok(entries) = fs::read_dir(blobs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "blob" {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    if !referenced_blobs.contains(stem) {
                                        orphan_blobs.push(stem.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(IntegrityReport {
            total_items,
            valid_blobs,
            corrupted_blobs,
            missing_blobs,
            orphan_blobs,
        })
    }
}

/// Ponto de entrada estático para criação e abertura de cofres em disco.
pub struct VaultStorage;

impl VaultStorage {
    /// Cria e inicializa um novo cofre `.magrathea` em disco de forma atômica.
    pub fn create(
        dir: &Path,
        project_title: &str,
        passphrase: &[u8],
        recovery_key: &SecretKey,
        custom_params: Option<Argon2Params>,
    ) -> Result<UnlockedVault, VaultError> {
        let header_path = dir.join("header");
        if header_path.exists() {
            return Err(VaultError::AlreadyExists(dir.display().to_string()));
        }

        fs::create_dir_all(dir).map_err(|e| VaultError::Io(e.to_string()))?;
        let blobs_dir = dir.join("blobs");
        fs::create_dir_all(&blobs_dir).map_err(|e| VaultError::Io(e.to_string()))?;

        let vault_id = format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>() & 0x0fff,
            (rand::random::<u16>() & 0x3fff) | 0x8000,
            rand::random::<u64>() & 0xffffffffffff
        );

        let timestamp_iso = current_timestamp_iso();

        let (header, dek) = initialize_vault(
            &vault_id,
            &timestamp_iso,
            passphrase,
            recovery_key,
            custom_params,
        )?;

        let manifest = VaultManifest::new(&vault_id, project_title, &timestamp_iso);

        // Grava o header de forma atômica
        let header_json = header.to_json()?;
        atomic_write(&header_path, header_json.as_bytes())?;

        // Cifra e grava o manifesto de forma atômica
        let manifest_bytes = manifest.encrypt(&dek)?;
        let manifest_path = dir.join("manifest.blob");
        atomic_write(&manifest_path, &manifest_bytes)?;

        Ok(UnlockedVault::new(dir.to_path_buf(), header, dek, manifest))
    }

    /// Abre um cofre existente decifrando o Slot 0 com a frase-senha.
    pub fn open_with_passphrase(
        dir: &Path,
        passphrase: &[u8],
    ) -> Result<UnlockedVault, VaultError> {
        let header_path = dir.join("header");
        if !header_path.exists() {
            return Err(VaultError::NotFound(header_path.display().to_string()));
        }

        let header_content =
            fs::read_to_string(&header_path).map_err(|e| VaultError::Io(e.to_string()))?;
        let header = VaultHeader::from_json(&header_content)?;

        let dek = header.unlock_with_passphrase(passphrase)?;

        let manifest_path = dir.join("manifest.blob");
        if !manifest_path.exists() {
            return Err(VaultError::NotFound(manifest_path.display().to_string()));
        }

        let manifest_bytes = fs::read(&manifest_path).map_err(|e| VaultError::Io(e.to_string()))?;
        let manifest = VaultManifest::decrypt(&dek, &header.vault_id, &manifest_bytes)?;

        Ok(UnlockedVault::new(dir.to_path_buf(), header, dek, manifest))
    }

    /// Abre um cofre existente decifrando o Slot 1 com a chave de recuperação.
    pub fn open_with_recovery_key(
        dir: &Path,
        recovery_key: &SecretKey,
    ) -> Result<UnlockedVault, VaultError> {
        let header_path = dir.join("header");
        if !header_path.exists() {
            return Err(VaultError::NotFound(header_path.display().to_string()));
        }

        let header_content =
            fs::read_to_string(&header_path).map_err(|e| VaultError::Io(e.to_string()))?;
        let header = VaultHeader::from_json(&header_content)?;

        let dek = header.unlock_with_recovery_key(recovery_key)?;

        let manifest_path = dir.join("manifest.blob");
        if !manifest_path.exists() {
            return Err(VaultError::NotFound(manifest_path.display().to_string()));
        }

        let manifest_bytes = fs::read(&manifest_path).map_err(|e| VaultError::Io(e.to_string()))?;
        let manifest = VaultManifest::decrypt(&dek, &header.vault_id, &manifest_bytes)?;

        Ok(UnlockedVault::new(dir.to_path_buf(), header, dek, manifest))
    }
}

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

/// Expected magic identifier in the vault header.
pub const VAULT_MAGIC: &str = "MAGRATHEA";

/// Current vault storage format version.
pub const CURRENT_FORMAT_VERSION: u16 = 1;

/// Minimum allowed Argon2id memory cost in KiB to prevent downgrade attacks (64 MiB).
pub const MIN_ARGON2_MEMORY_KIB: u32 = 65536;

/// Minimum allowed Argon2id iterations to prevent downgrade attacks.
pub const MIN_ARGON2_ITERATIONS: u32 = 3;

/// Errors arising from vault format operations.
#[derive(Debug, PartialEq, Eq)]
pub enum VaultError {
    /// Header magic bytes did not match "MAGRATHEA".
    InvalidMagic(String),
    /// Vault format version is newer than supported by this software version.
    UnsupportedVersion(u16),
    /// Insecure KDF parameters detected in header.
    InsecureKdfParameters(String),
    /// Cryptographic operation failed (wrong key, bad MAC tag, or corrupted payload).
    Crypto(String),
    /// Serialization or deserialization error.
    Serialization(String),
    /// Blob data is shorter than the minimum nonce length.
    CorruptedBlob,
    /// Vault ID mismatch during manifest authentication.
    VaultIdMismatch,
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
        }
    }
}

impl std::error::Error for VaultError {}

impl From<CryptoError> for VaultError {
    fn from(err: CryptoError) -> Self {
        Self::Crypto(err.to_string())
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

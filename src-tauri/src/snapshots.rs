//! Módulo de snapshots e controle de versões por cena para o Magrathea Writer (M5-01, M5-02).
//!
//! Garante histórico de revisões com opacidade estrita: todos os snapshots e seus índices
//! são criptografados com XChaCha20-Poly1305 no disco utilizando a DEK do cofre.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::crypto::{decrypt_payload, encrypt_payload, Nonce, NONCE_LEN};
use crate::vault::{atomic_write, current_timestamp_iso, UnlockedVault, VaultError};

/// Metadados de um snapshot de cena.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMeta {
    pub id: String,
    pub scene_id: String,
    pub timestamp: String,
    pub label: String,
    pub trigger: String, // "manual" ou "auto"
    pub word_count: usize,
    pub byte_size: usize,
}

/// Índice de snapshots para uma cena específica.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneSnapshotIndex {
    pub snapshots: Vec<SnapshotMeta>,
}

/// Uma linha no comparador visual de diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: String, // "equal", "added", "removed"
    pub content: String,
}

/// Resultado da comparação (diff) entre um snapshot e o conteúdo atual da cena.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotDiff {
    pub snapshot_id: String,
    pub snapshot_timestamp: String,
    pub current_word_count: usize,
    pub snapshot_word_count: usize,
    pub lines: Vec<DiffLine>,
}

/// Caminho da pasta de snapshots de uma cena no cofre.
fn scene_snapshots_dir(vault: &UnlockedVault, scene_id: &str) -> PathBuf {
    vault.root_path().join("snapshots").join(scene_id)
}

/// Caminho do arquivo de índice cifrado de snapshots da cena.
fn scene_index_path(vault: &UnlockedVault, scene_id: &str) -> PathBuf {
    scene_snapshots_dir(vault, scene_id).join("index.blob")
}

/// Carrega o índice de snapshots de uma cena.
pub fn load_scene_snapshot_index(
    vault: &UnlockedVault,
    scene_id: &str,
) -> Result<SceneSnapshotIndex, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let index_file = scene_index_path(vault, scene_id);
    if !index_file.exists() {
        return Ok(SceneSnapshotIndex::default());
    }

    let blob_bytes = fs::read(&index_file).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let aad = format!("MAGRATHEA-V1:snapshots_index:{}", scene_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    serde_json::from_slice(&plaintext).map_err(|e| VaultError::Serialization(e.to_string()))
}

/// Salva o índice de snapshots de uma cena de forma atômica e cifrada.
pub fn save_scene_snapshot_index(
    vault: &UnlockedVault,
    scene_id: &str,
    index: &SceneSnapshotIndex,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let dir = scene_snapshots_dir(vault, scene_id);
    fs::create_dir_all(&dir).map_err(|e| VaultError::Io(e.to_string()))?;

    let plaintext =
        serde_json::to_vec_pretty(index).map_err(|e| VaultError::Serialization(e.to_string()))?;

    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:snapshots_index:{}", scene_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &plaintext)?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(nonce.as_bytes());
    payload.extend_from_slice(&ciphertext);

    let final_path = scene_index_path(vault, scene_id);
    atomic_write(&final_path, &payload)?;

    Ok(())
}

/// Conta palavras de um texto de maneira consistente.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Cria um novo snapshot de uma cena.
pub fn create_scene_snapshot_internal(
    vault: &UnlockedVault,
    scene_id: &str,
    content: &str,
    label: &str,
    trigger: &str,
) -> Result<SnapshotMeta, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let timestamp_iso = current_timestamp_iso();
    let snap_id = format!(
        "snap_{:08x}{:08x}",
        rand::random::<u32>(),
        rand::random::<u32>()
    );

    let dir = scene_snapshots_dir(vault, scene_id);
    fs::create_dir_all(&dir).map_err(|e| VaultError::Io(e.to_string()))?;

    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:snapshot:{}:{}", scene_id, snap_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), content.as_bytes())?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(nonce.as_bytes());
    payload.extend_from_slice(&ciphertext);

    let snap_blob_path = dir.join(format!("{}.blob", snap_id));
    atomic_write(&snap_blob_path, &payload)?;

    let meta = SnapshotMeta {
        id: snap_id,
        scene_id: scene_id.to_string(),
        timestamp: timestamp_iso,
        label: if label.trim().is_empty() {
            if trigger == "auto" {
                "Snapshot automático de segurança".to_string()
            } else {
                "Snapshot manual".to_string()
            }
        } else {
            label.trim().to_string()
        },
        trigger: trigger.to_string(),
        word_count: count_words(content),
        byte_size: content.len(),
    };

    let mut index = load_scene_snapshot_index(vault, scene_id)?;
    // Adiciona o snapshot mais recente no topo
    index.snapshots.insert(0, meta.clone());
    save_scene_snapshot_index(vault, scene_id, &index)?;

    Ok(meta)
}

/// Lista todos os snapshots de uma cena ordenados do mais recente para o mais antigo.
pub fn list_scene_snapshots_internal(
    vault: &UnlockedVault,
    scene_id: &str,
) -> Result<Vec<SnapshotMeta>, VaultError> {
    let index = load_scene_snapshot_index(vault, scene_id)?;
    Ok(index.snapshots)
}

/// Carrega e decifra o conteúdo de um snapshot específico.
pub fn get_scene_snapshot_content_internal(
    vault: &UnlockedVault,
    scene_id: &str,
    snapshot_id: &str,
) -> Result<String, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let snap_blob_path = scene_snapshots_dir(vault, scene_id).join(format!("{}.blob", snapshot_id));
    if !snap_blob_path.exists() {
        return Err(VaultError::NotFound(format!(
            "Snapshot {} não encontrado",
            snapshot_id
        )));
    }

    let blob_bytes = fs::read(&snap_blob_path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let aad = format!("MAGRATHEA-V1:snapshot:{}:{}", scene_id, snapshot_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    String::from_utf8(plaintext).map_err(|e| VaultError::Serialization(e.to_string()))
}

/// Calcula a diferença linha a linha entre dois textos (LCS diff simplificado e determinístico).
pub fn compute_line_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let n = old_lines.len();
    let m = new_lines.len();

    // Se um dos textos for vazio, gera inserção ou remoção completa
    if n == 0 {
        return new_lines
            .into_iter()
            .map(|l| DiffLine {
                kind: "added".to_string(),
                content: l.to_string(),
            })
            .collect();
    }
    if m == 0 {
        return old_lines
            .into_iter()
            .map(|l| DiffLine {
                kind: "removed".to_string(),
                content: l.to_string(),
            })
            .collect();
    }

    // Matriz DP para maior subsequência comum (LCS)
    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    for i in 0..n {
        for j in 0..m {
            if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i][j].max(dp[i + 1][j]);
            }
        }
    }

    // Reconstrução do diff
    let mut diff = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            diff.push(DiffLine {
                kind: "equal".to_string(),
                content: old_lines[i - 1].to_string(),
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            diff.push(DiffLine {
                kind: "added".to_string(),
                content: new_lines[j - 1].to_string(),
            });
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
            diff.push(DiffLine {
                kind: "removed".to_string(),
                content: old_lines[i - 1].to_string(),
            });
            i -= 1;
        }
    }

    diff.reverse();
    diff
}

/// Compara um snapshot com o conteúdo atual de uma cena.
pub fn compare_snapshot_with_current_internal(
    vault: &UnlockedVault,
    scene_id: &str,
    snapshot_id: &str,
    current_content: &str,
) -> Result<SnapshotDiff, VaultError> {
    let snap_content = get_scene_snapshot_content_internal(vault, scene_id, snapshot_id)?;
    let index = load_scene_snapshot_index(vault, scene_id)?;
    let snap_meta = index
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or_else(|| {
            VaultError::NotFound(format!("Snapshot {} não encontrado no índice", snapshot_id))
        })?;

    let lines = compute_line_diff(&snap_content, current_content);

    Ok(SnapshotDiff {
        snapshot_id: snapshot_id.to_string(),
        snapshot_timestamp: snap_meta.timestamp.clone(),
        current_word_count: count_words(current_content),
        snapshot_word_count: snap_meta.word_count,
        lines,
    })
}

/// Restaura o snapshot para a cena atual.
/// Antes de restaurar, gera automaticamente um snapshot de segurança com o conteúdo atual.
pub fn restore_scene_snapshot_internal(
    vault: &UnlockedVault,
    scene_id: &str,
    snapshot_id: &str,
    current_content: &str,
) -> Result<(String, SnapshotMeta), VaultError> {
    // 1. Snapshot de segurança do estado presente
    let safety_meta = create_scene_snapshot_internal(
        vault,
        scene_id,
        current_content,
        "Snapshot automático antes da restauração",
        "auto",
    )?;

    // 2. Recupera o conteúdo do snapshot desejado
    let restored_content = get_scene_snapshot_content_internal(vault, scene_id, snapshot_id)?;

    Ok((restored_content, safety_meta))
}

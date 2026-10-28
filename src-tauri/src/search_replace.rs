//! Módulo de busca e substituição global em memória com prévia e rollback (M5-03, M5-04).
//!
//! Todas as buscas são realizadas decifrando em memória com a DEK do cofre, sem nenhum
//! arquivo temporário em claro no disco. A substituição global gera snapshot prévio de
//! segurança atômico permitindo reversão (rollback) completa.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::crypto::{decrypt_payload, encrypt_payload, Nonce, NONCE_LEN};
use crate::index::VaultIndex;
use crate::vault::{atomic_write, current_timestamp_iso, ManifestItem, UnlockedVault, VaultError};

/// Uma ocorrência encontrada na busca dentro de uma linha.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchOccurrence {
    pub line_number: usize,
    pub snippet: String,
    pub match_start: usize,
    pub match_end: usize,
}

/// Resultado de busca para um documento específico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentSearchResult {
    pub item_id: String,
    pub title: String,
    pub kind: String,
    pub match_count: usize,
    pub occurrences: Vec<SearchOccurrence>,
}

/// Resposta completa da busca global no cofre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalSearchResponse {
    pub query: String,
    pub match_case: bool,
    pub total_matches: usize,
    pub documents: Vec<DocumentSearchResult>,
}

/// Item alterado na prévia de substituição global.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplacePreviewItem {
    pub item_id: String,
    pub title: String,
    pub kind: String,
    pub occurrences_count: usize,
    pub before_snippet: String,
    pub after_snippet: String,
}

/// Prévia de substituição global antes de ser aplicada.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalReplacePreview {
    pub query: String,
    pub replacement: String,
    pub match_case: bool,
    pub total_affected_docs: usize,
    pub total_occurrences: usize,
    pub items: Vec<ReplacePreviewItem>,
}

/// Resultado da execução de uma substituição global.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyGlobalReplaceResult {
    pub query: String,
    pub replacement: String,
    pub modified_docs_count: usize,
    pub total_occurrences_replaced: usize,
    pub rollback_snapshot_id: String,
}

/// Snapshot global para rollback de substituição em massa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalReplaceSnapshot {
    pub id: String,
    pub timestamp: String,
    pub query: String,
    pub replacement: String,
    pub documents: HashMap<String, String>, // item_id -> conteúdo antes da substituição
}

/// Caminho da pasta de snapshots globais no cofre.
fn global_snapshots_dir(vault: &UnlockedVault) -> PathBuf {
    vault.root_path().join("snapshots").join("global")
}

/// Extrai ocorrências de uma consulta em um texto.
pub fn search_in_text(text: &str, query: &str, match_case: bool) -> Vec<SearchOccurrence> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut occurrences = Vec::new();
    let query_cmp = if match_case {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    for (zero_idx, line) in text.lines().enumerate() {
        let line_num = zero_idx + 1;
        let line_cmp = if match_case {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        let mut start_search = 0;
        while let Some(pos) = line_cmp[start_search..].find(&query_cmp) {
            let actual_match_start = start_search + pos;
            let actual_match_end = actual_match_start + query.len();

            let snippet_start = actual_match_start.saturating_sub(30);
            let snippet_end = (actual_match_end + 30).min(line.len());

            let mut snippet = String::new();
            if snippet_start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&line[snippet_start..snippet_end]);
            if snippet_end < line.len() {
                snippet.push_str("...");
            }

            occurrences.push(SearchOccurrence {
                line_number: line_num,
                snippet,
                match_start: actual_match_start,
                match_end: actual_match_end,
            });

            start_search = actual_match_end;
            if start_search >= line.len() {
                break;
            }
        }
    }

    occurrences
}

/// Substitui ocorrências preservando maiúsculas/minúsculas conforme solicitado.
pub fn replace_in_text(
    text: &str,
    query: &str,
    replacement: &str,
    match_case: bool,
) -> (String, usize) {
    if query.is_empty() {
        return (text.to_string(), 0);
    }

    if match_case {
        let count = text.matches(query).count();
        (text.replace(query, replacement), count)
    } else {
        let query_lower = query.to_lowercase();
        let mut result = String::with_capacity(text.len());
        let mut count = 0;
        let mut last_idx = 0;

        let text_lower = text.to_lowercase();
        while let Some(pos) = text_lower[last_idx..].find(&query_lower) {
            let match_idx = last_idx + pos;
            result.push_str(&text[last_idx..match_idx]);
            result.push_str(replacement);
            last_idx = match_idx + query.len();
            count += 1;
        }
        result.push_str(&text[last_idx..]);
        (result, count)
    }
}

/// Coleta todos os itens de conteúdo elegíveis (cenas ativas e entidades de mundo).
fn collect_searchable_items<'a>(items: &'a [ManifestItem], out: &mut Vec<&'a ManifestItem>) {
    for item in items {
        if item.kind == "scene" || (item.kind != "part" && item.kind != "chapter") {
            out.push(item);
        }
    }
}

/// Executa busca textual global em todo o cofre literário.
pub fn execute_global_search(
    vault: &UnlockedVault,
    query: &str,
    match_case: bool,
) -> Result<GlobalSearchResponse, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(GlobalSearchResponse {
            query: query.to_string(),
            match_case,
            total_matches: 0,
            documents: Vec::new(),
        });
    }

    let manifest = vault.manifest()?;
    let mut eligible_items = Vec::new();
    collect_searchable_items(&manifest.items, &mut eligible_items);

    let mut doc_results = Vec::new();
    let mut total_matches = 0;

    for item in eligible_items {
        if let Ok(bytes) = vault.read_blob(&item.id) {
            if let Ok(content) = String::from_utf8(bytes) {
                let occurrences = search_in_text(&content, trimmed, match_case);
                if !occurrences.is_empty() {
                    total_matches += occurrences.len();
                    doc_results.push(DocumentSearchResult {
                        item_id: item.id.clone(),
                        title: item.title.clone(),
                        kind: item.kind.clone(),
                        match_count: occurrences.len(),
                        occurrences,
                    });
                }
            }
        }
    }

    Ok(GlobalSearchResponse {
        query: trimmed.to_string(),
        match_case,
        total_matches,
        documents: doc_results,
    })
}

/// Gera uma pré-visualização das substituições que serão efetuadas em massa no cofre.
pub fn preview_global_replace_internal(
    vault: &UnlockedVault,
    query: &str,
    replacement: &str,
    match_case: bool,
) -> Result<GlobalReplacePreview, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(GlobalReplacePreview {
            query: query.to_string(),
            replacement: replacement.to_string(),
            match_case,
            total_affected_docs: 0,
            total_occurrences: 0,
            items: Vec::new(),
        });
    }

    let manifest = vault.manifest()?;
    let mut eligible_items = Vec::new();
    collect_searchable_items(&manifest.items, &mut eligible_items);

    let mut items_preview = Vec::new();
    let mut total_occurrences = 0;

    for item in eligible_items {
        if let Ok(bytes) = vault.read_blob(&item.id) {
            if let Ok(content) = String::from_utf8(bytes) {
                let (replaced_content, count) =
                    replace_in_text(&content, trimmed, replacement, match_case);
                if count > 0 {
                    total_occurrences += count;

                    let occurrences = search_in_text(&content, trimmed, match_case);
                    let before_snippet = occurrences
                        .first()
                        .map(|o| o.snippet.clone())
                        .unwrap_or_default();
                    let after_occurrences =
                        search_in_text(&replaced_content, replacement, match_case);
                    let after_snippet = after_occurrences
                        .first()
                        .map(|o| o.snippet.clone())
                        .unwrap_or_default();

                    items_preview.push(ReplacePreviewItem {
                        item_id: item.id.clone(),
                        title: item.title.clone(),
                        kind: item.kind.clone(),
                        occurrences_count: count,
                        before_snippet,
                        after_snippet,
                    });
                }
            }
        }
    }

    let total_affected = items_preview.len();
    Ok(GlobalReplacePreview {
        query: trimmed.to_string(),
        replacement: replacement.to_string(),
        match_case,
        total_affected_docs: total_affected,
        total_occurrences,
        items: items_preview,
    })
}

/// Aplica a substituição global atômica com criação prévia de snapshot global de rollback.
pub fn apply_global_replace_internal(
    vault: &mut UnlockedVault,
    index: &mut VaultIndex,
    query: &str,
    replacement: &str,
    match_case: bool,
) -> Result<ApplyGlobalReplaceResult, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(VaultError::Serialization(
            "O termo de busca não pode ser vazio.".to_string(),
        ));
    }

    let mut eligible_items = Vec::new();
    {
        let manifest = vault.manifest()?;
        collect_searchable_items(&manifest.items, &mut eligible_items);
    }

    let mut original_contents = HashMap::new();
    let mut modified_contents = HashMap::new();
    let mut total_occurrences = 0;

    for item in &eligible_items {
        if let Ok(bytes) = vault.read_blob(&item.id) {
            if let Ok(content) = String::from_utf8(bytes) {
                let (replaced, count) = replace_in_text(&content, trimmed, replacement, match_case);
                if count > 0 {
                    total_occurrences += count;
                    original_contents.insert(item.id.clone(), content);
                    modified_contents.insert(item.id.clone(), replaced);
                }
            }
        }
    }

    if modified_contents.is_empty() {
        return Ok(ApplyGlobalReplaceResult {
            query: trimmed.to_string(),
            replacement: replacement.to_string(),
            modified_docs_count: 0,
            total_occurrences_replaced: 0,
            rollback_snapshot_id: String::new(),
        });
    }

    // 1. Cria snapshot global de segurança cifrado
    let dek = vault.dek()?;
    let timestamp_iso = current_timestamp_iso();
    let snapshot_id = format!(
        "global_replace_{:08x}{:08x}",
        rand::random::<u32>(),
        rand::random::<u32>()
    );

    let snapshot_obj = GlobalReplaceSnapshot {
        id: snapshot_id.clone(),
        timestamp: timestamp_iso,
        query: trimmed.to_string(),
        replacement: replacement.to_string(),
        documents: original_contents,
    };

    let dir = global_snapshots_dir(vault);
    fs::create_dir_all(&dir).map_err(|e| VaultError::Io(e.to_string()))?;

    let snap_bytes = serde_json::to_vec_pretty(&snapshot_obj)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;

    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:global_snapshot:{}", snapshot_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &snap_bytes)?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(nonce.as_bytes());
    payload.extend_from_slice(&ciphertext);

    let snap_path = dir.join(format!("{}.blob", snapshot_id));
    atomic_write(&snap_path, &payload)?;

    // 2. Aplica as substituições nos blobs de cada documento e atualiza o índice
    let item_meta_map: HashMap<String, (String, String)> = eligible_items
        .iter()
        .map(|it| (it.id.clone(), (it.title.clone(), it.kind.clone())))
        .collect();

    for (item_id, new_content) in &modified_contents {
        vault.write_blob(item_id, new_content.as_bytes())?;

        if let Some((title, kind)) = item_meta_map.get(item_id) {
            index.index_document_content(item_id, title, kind, new_content);
        }
    }

    let modified_count = modified_contents.len();
    Ok(ApplyGlobalReplaceResult {
        query: trimmed.to_string(),
        replacement: replacement.to_string(),
        modified_docs_count: modified_count,
        total_occurrences_replaced: total_occurrences,
        rollback_snapshot_id: snapshot_id,
    })
}

/// Reverte as alterações de uma substituição global a partir do snapshot de rollback.
pub fn rollback_global_replace_internal(
    vault: &mut UnlockedVault,
    index: &mut VaultIndex,
    snapshot_id: &str,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    let snap_path = global_snapshots_dir(vault).join(format!("{}.blob", snapshot_id));
    if !snap_path.exists() {
        return Err(VaultError::NotFound(format!(
            "Snapshot de rollback {} não encontrado",
            snapshot_id
        )));
    }

    let blob_bytes = fs::read(&snap_path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let dek = vault.dek()?;
    let aad = format!("MAGRATHEA-V1:global_snapshot:{}", snapshot_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    let snapshot_obj: GlobalReplaceSnapshot =
        serde_json::from_slice(&plaintext).map_err(|e| VaultError::Serialization(e.to_string()))?;

    let item_meta_map: HashMap<String, (String, String)> = {
        let manifest = vault.manifest()?;
        let mut eligible_items = Vec::new();
        collect_searchable_items(&manifest.items, &mut eligible_items);
        eligible_items
            .into_iter()
            .map(|it| (it.id.clone(), (it.title.clone(), it.kind.clone())))
            .collect()
    };

    // Restaura o conteúdo original de cada documento
    for (item_id, original_content) in snapshot_obj.documents {
        vault.write_blob(&item_id, original_content.as_bytes())?;

        if let Some((title, kind)) = item_meta_map.get(&item_id) {
            index.index_document_content(&item_id, title, kind, &original_content);
        }
    }

    Ok(())
}

//! Módulo de refatoração global e renomeação segura de entidades (PRD E4 / M2-05).
//!
//! Requisitos e Garantias:
//! - Prévia detalhada de cada ocorrência de `[[old_name]]` e `[[old_name|alias]]` em todo o cofre.
//! - Criação de snapshot em memória antes de aplicar qualquer alteração nos blobs.
//! - Substituição atômica e cifrada em disco com zero vazamento de texto claro.
//! - Suporte a rollback instantâneo para restaurar o estado original se o autor desfizer.

use crate::document::Document;
use crate::index::VaultIndex;
use crate::vault::{UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};

/// Ocorrência individual identificada para pré-visualização de substituição.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OccurrencePreview {
    /// Número da linha (1-based) onde a ocorrência se encontra.
    pub line_number: usize,
    /// Texto bruto original (ex: `[[Jaci]]` ou `[[Jaci|Guardiã]]`).
    pub raw_match: String,
    /// Trecho de contexto original antes da substituição.
    pub old_snippet: String,
    /// Trecho de contexto simulado após a substituição.
    pub new_snippet: String,
}

/// Agrupamento de ocorrências encontradas dentro de um documento literário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentRefactorPreview {
    /// Identificador UUID do item.
    pub item_id: String,
    /// Título do documento.
    pub title: String,
    /// Tipo de nó ("scene", "character", "note", etc.).
    pub kind: String,
    /// Lista de ocorrências encontradas no corpo deste documento.
    pub occurrences: Vec<OccurrencePreview>,
}

/// Resposta completa da prévia de refatoração para exibição no modal da interface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenameRefactorPreviewResponse {
    /// Nome original a ser substituído.
    pub old_name: String,
    /// Novo nome proposto.
    pub new_name: String,
    /// Quantidade total de ocorrências encontradas no projeto.
    pub total_occurrences: usize,
    /// Documentos afetados com suas respectivas prévias.
    pub affected_documents: Vec<DocumentRefactorPreview>,
}

/// Resposta retornada após aplicar a refatoração atômica em todo o projeto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyRenameRefactorResponse {
    /// Identificador do snapshot gerado para rollback.
    pub snapshot_id: String,
    /// Quantidade de documentos gravados e atualizados.
    pub modified_docs_count: usize,
    /// Total de links e menções substituídos.
    pub total_replacements: usize,
    /// Timestamp ISO da operação.
    pub applied_at: String,
}

/// Snapshot contendo o conteúdo integral e metadados antes da refatoração.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorSnapshot {
    pub snapshot_id: String,
    pub created_at: String,
    pub old_name: String,
    pub new_name: String,
    /// Tuplas: (item_id, blob_id_opt, original_text, original_title, original_slug)
    pub original_documents: Vec<(String, Option<String>, String, String, String)>,
}

/// Gera a pré-visualização completa das substituições sem alterar nada no cofre.
pub fn preview_rename(
    vault: &UnlockedVault,
    old_name: &str,
    new_name: &str,
) -> Result<RenameRefactorPreviewResponse, VaultError> {
    let trimmed_old = old_name.trim();
    let trimmed_new = new_name.trim();

    if trimmed_old.is_empty() || trimmed_new.is_empty() {
        return Ok(RenameRefactorPreviewResponse {
            old_name: trimmed_old.to_string(),
            new_name: trimmed_new.to_string(),
            total_occurrences: 0,
            affected_documents: Vec::new(),
        });
    }

    let manifest = vault.manifest()?;
    let mut affected_documents = Vec::new();
    let mut total_occurrences = 0;

    for item in &manifest.items {
        if let Some(ref blob_id) = item.blob_id {
            if let Ok(bytes) = vault.read_blob(blob_id) {
                if let Ok(content) = String::from_utf8(bytes) {
                    let occurrences =
                        find_and_preview_occurrences(&content, trimmed_old, trimmed_new);
                    if !occurrences.is_empty() {
                        total_occurrences += occurrences.len();
                        affected_documents.push(DocumentRefactorPreview {
                            item_id: item.id.clone(),
                            title: item.title.clone(),
                            kind: item.kind.clone(),
                            occurrences,
                        });
                    }
                }
            }
        }
    }

    Ok(RenameRefactorPreviewResponse {
        old_name: trimmed_old.to_string(),
        new_name: trimmed_new.to_string(),
        total_occurrences,
        affected_documents,
    })
}

/// Aplica a substituição em todos os documentos afetados, grava o snapshot e atualiza o manifesto.
pub fn apply_rename(
    vault: &mut UnlockedVault,
    index: &mut VaultIndex,
    old_name: &str,
    new_name: &str,
) -> Result<(ApplyRenameRefactorResponse, RefactorSnapshot), VaultError> {
    let trimmed_old = old_name.trim();
    let trimmed_new = new_name.trim();

    if trimmed_old.is_empty() || trimmed_new.is_empty() {
        return Err(VaultError::Crypto(
            "Os nomes antigo e novo não podem ser vazios.".to_string(),
        ));
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let snapshot_id = format!(
        "snap-{:08x}-{:04x}",
        rand::random::<u32>(),
        rand::random::<u16>()
    );

    let mut original_documents = Vec::new();
    let mut modified_docs_count = 0;
    let mut total_replacements = 0;

    let manifest = vault.manifest()?.clone();

    // 1. Itera sobre os documentos e realiza a substituição nos blobs
    for item in &manifest.items {
        let is_target_entity = item.title.eq_ignore_ascii_case(trimmed_old)
            || item
                .slug
                .eq_ignore_ascii_case(&VaultIndex::normalize_key(trimmed_old));

        let mut doc_content = if let Some(ref blob_id) = item.blob_id {
            vault
                .read_blob(blob_id)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let occurrences = find_and_preview_occurrences(&doc_content, trimmed_old, trimmed_new);
        let has_text_occurrences = !occurrences.is_empty();

        if has_text_occurrences || is_target_entity {
            // Guarda o estado original no snapshot antes de qualquer gravação
            original_documents.push((
                item.id.clone(),
                item.blob_id.clone(),
                doc_content.clone(),
                item.title.clone(),
                item.slug.clone(),
            ));

            if has_text_occurrences {
                total_replacements += occurrences.len();
                doc_content = replace_wikilinks(&doc_content, trimmed_old, trimmed_new);
            }

            // Se for a própria entidade sendo renomeada, atualiza também o frontmatter se houver
            if is_target_entity {
                if let Ok(mut parsed_doc) = Document::parse(&doc_content) {
                    if parsed_doc.title.is_some() {
                        parsed_doc.title = Some(trimmed_new.to_string());
                        if let Ok(serialized) = parsed_doc.serialize() {
                            doc_content = serialized;
                        }
                    }
                }
            }

            if let Some(ref blob_id) = item.blob_id {
                vault.write_blob(blob_id, doc_content.as_bytes())?;
            }

            modified_docs_count += 1;
        }
    }

    // 2. Atualiza o título e o slug da entidade no manifesto do cofre
    let new_slug = trimmed_new
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let manifest_mut = vault.manifest_mut()?;
    for it in &mut manifest_mut.items {
        if it.title.eq_ignore_ascii_case(trimmed_old)
            || it
                .slug
                .eq_ignore_ascii_case(&VaultIndex::normalize_key(trimmed_old))
        {
            it.title = trimmed_new.to_string();
            it.slug = new_slug.clone();
        }
    }

    // 3. Persistência atômica do manifesto criptografado
    vault.save_manifest(&timestamp_iso)?;

    // 4. Re-indexação completa em memória
    *index = VaultIndex::build_from_vault(vault)?;

    let snapshot = RefactorSnapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: timestamp_iso.clone(),
        old_name: trimmed_old.to_string(),
        new_name: trimmed_new.to_string(),
        original_documents,
    };

    let response = ApplyRenameRefactorResponse {
        snapshot_id,
        modified_docs_count,
        total_replacements,
        applied_at: timestamp_iso,
    };

    Ok((response, snapshot))
}

/// Restaura todos os documentos e metadados para o estado exato anterior à refatoração.
pub fn rollback_rename(
    vault: &mut UnlockedVault,
    index: &mut VaultIndex,
    snapshot: &RefactorSnapshot,
) -> Result<(), VaultError> {
    let timestamp_iso = crate::vault::current_timestamp_iso();

    // 1. Restaura os conteúdos dos blobs
    for (item_id, blob_id_opt, original_text, orig_title, orig_slug) in &snapshot.original_documents
    {
        if let Some(ref blob_id) = blob_id_opt {
            vault.write_blob(blob_id, original_text.as_bytes())?;
        }

        // Restaura no manifesto
        let manifest_mut = vault.manifest_mut()?;
        if let Some(it) = manifest_mut.items.iter_mut().find(|it| it.id == *item_id) {
            it.title = orig_title.clone();
            it.slug = orig_slug.clone();
        }
    }

    // 2. Salva manifesto restaurado
    vault.save_manifest(&timestamp_iso)?;

    // 3. Re-indexa em memória
    *index = VaultIndex::build_from_vault(vault)?;

    Ok(())
}

/// Identifica todas as ocorrências de `[[alvo]]` ou `[[alvo|alias]]` e gera a prévia de substituição.
pub fn find_and_preview_occurrences(
    text: &str,
    target: &str,
    replacement: &str,
) -> Vec<OccurrencePreview> {
    let mut list = Vec::new();
    let norm_target = target.trim().to_lowercase();

    for (line_idx, line) in text.lines().enumerate() {
        let line_num = line_idx + 1;
        let mut cursor = 0;

        while let Some(start_pos) = line[cursor..].find("[[") {
            let abs_start = cursor + start_pos;
            if let Some(end_pos) = line[abs_start..].find("]]") {
                let abs_end = abs_start + end_pos + 2;
                let inside = &line[abs_start + 2..abs_start + end_pos];
                let raw_match = &line[abs_start..abs_end];

                let (match_target, match_alias) = if let Some((t, a)) = inside.split_once('|') {
                    (t.trim(), Some(a.trim()))
                } else {
                    (inside.trim(), None)
                };

                if match_target.to_lowercase() == norm_target {
                    let new_raw = if let Some(alias) = match_alias {
                        format!("[[{replacement}|{alias}]]")
                    } else {
                        format!("[[{replacement}]]")
                    };

                    let old_snippet = line.trim().to_string();
                    let new_snippet = line.replace(raw_match, &new_raw).trim().to_string();

                    list.push(OccurrencePreview {
                        line_number: line_num,
                        raw_match: raw_match.to_string(),
                        old_snippet,
                        new_snippet,
                    });
                }

                cursor = abs_end;
            } else {
                break;
            }
        }
    }

    list
}

/// Substitui todas as menções de `[[alvo]]` e `[[alvo|alias]]` no texto de forma determinística.
pub fn replace_wikilinks(text: &str, target: &str, replacement: &str) -> String {
    let norm_target = target.trim().to_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(start_pos) = text[cursor..].find("[[") {
        let abs_start = cursor + start_pos;
        result.push_str(&text[cursor..abs_start]);

        if let Some(end_pos) = text[abs_start..].find("]]") {
            let abs_end = abs_start + end_pos + 2;
            let inside = &text[abs_start + 2..abs_start + end_pos];

            let (match_target, match_alias) = if let Some((t, a)) = inside.split_once('|') {
                (t.trim(), Some(a.trim()))
            } else {
                (inside.trim(), None)
            };

            if match_target.to_lowercase() == norm_target {
                if let Some(alias) = match_alias {
                    result.push_str(&format!("[[{replacement}|{alias}]]"));
                } else {
                    result.push_str(&format!("[[{replacement}]]"));
                }
            } else {
                result.push_str(&text[abs_start..abs_end]);
            }

            cursor = abs_end;
        } else {
            result.push_str(&text[abs_start..]);
            cursor = text.len();
            break;
        }
    }

    if cursor < text.len() {
        result.push_str(&text[cursor..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_and_replace_wikilinks() {
        let original = "O marinheiro viu [[Jaci]] nas docas.\nDepois chamou [[Jaci|A Guardiã]].\nOutro: [[Outra Pessoa]].";
        let occurrences = find_and_preview_occurrences(original, "Jaci", "Iara");

        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].line_number, 1);
        assert_eq!(occurrences[0].raw_match, "[[Jaci]]");
        assert!(occurrences[0].new_snippet.contains("[[Iara]]"));

        assert_eq!(occurrences[1].line_number, 2);
        assert_eq!(occurrences[1].raw_match, "[[Jaci|A Guardiã]]");
        assert!(occurrences[1].new_snippet.contains("[[Iara|A Guardiã]]"));

        let replaced = replace_wikilinks(original, "Jaci", "Iara");
        assert!(replaced.contains("[[Iara]]"));
        assert!(replaced.contains("[[Iara|A Guardiã]]"));
        assert!(replaced.contains("[[Outra Pessoa]]"));
        assert!(!replaced.contains("[[Jaci]]"));
    }
}

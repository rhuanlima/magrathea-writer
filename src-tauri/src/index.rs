//! Índice em memória de entidades, links e backlinks para o Segundo Cérebro (PRD E4 / M2-01).
//!
//! Requisitos de segurança e integridade (SECURITY §3.4 / PRD E1):
//! - O índice reside 100% em memória; nada é persistido em claro no disco.
//! - Reconstruído rapidamente ao desbloquear o cofre (< 2s para projetos de 500 cenas / 300 entidades).
//! - Atualizado reativamente quando documentos são salvos, criados, renomeados ou excluídos.
//! - Destruído/limpo da memória no momento em que o cofre for bloqueado.

use crate::document::Document;
use crate::vault::{UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representa uma menção de `[[wikilink]]` detectada no corpo de um documento literário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Wikilink {
    /// Texto bruto original (ex: `[[Jaci]]` ou `[[Jaci|Guardiã das Águas]]`).
    pub raw: String,
    /// Alvo lógico citado (ex: `Jaci`).
    pub target: String,
    /// Rótulo ou apelido visual opcional (ex: `Guardiã das Águas`).
    pub label: Option<String>,
    /// Número da linha (1-based) onde a menção ocorre no texto.
    pub line_number: usize,
    /// Offset em bytes do início da menção no texto.
    pub byte_offset: usize,
    /// Trecho conciso de contexto ao redor da menção para exibição em backlinks.
    pub context_snippet: String,
}

/// Metadados de uma entidade indexada no cofre (cena, personagem, local, objeto ou nota).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityEntry {
    /// Identificador UUID único do item no manifesto.
    pub id: String,
    /// Título humano da entidade ou cena.
    pub title: String,
    /// Slug normalizado para resolução insensível a maiúsculas/minúsculas.
    pub slug: String,
    /// Tipo de nó ("scene", "character", "location", "item", "faction", "lore", "note", "chapter", "part").
    pub kind: String,
    /// Tags associadas.
    pub tags: Vec<String>,
    /// Status no fluxo de escrita ("draft", "revised", "final", etc.).
    pub status: Option<String>,
    /// Resumo curto para hover preview (extraído do frontmatter ou do 1º parágrafo).
    pub summary: Option<String>,
    /// Contagem total de vezes que esta entidade é mencionada em outros documentos do projeto.
    pub mention_count: usize,
}

/// Ocorrência de citação (backlink) apontando para uma entidade/cena.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BacklinkEntry {
    /// UUID do documento que fez a citação.
    pub source_id: String,
    /// Título do documento que fez a citação.
    pub source_title: String,
    /// Tipo do documento de origem ("scene", "note", etc.).
    pub source_kind: String,
    /// Alvo exato como escrito no texto (ex: "Jaci").
    pub target_query: String,
    /// Rótulo/alias utilizado, se houver.
    pub alias: Option<String>,
    /// Linha onde o link se encontra.
    pub line_number: usize,
    /// Offset em bytes.
    pub byte_offset: usize,
    /// Trecho de contexto ao redor do link.
    pub snippet: String,
}

/// Resumo serializável do índice para o frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexOverviewResponse {
    /// Lista de todas as entidades e cenas catalogadas.
    pub entities: Vec<EntityEntry>,
    /// Total de links bidirecionais resolvidos no projeto.
    pub total_links_count: usize,
    /// Total de referências a notas ainda não criadas.
    pub unresolved_links_count: usize,
    /// Mapa de links pendentes (alvo sem documento correspondente -> ocorrências).
    pub unresolved_targets: Vec<String>,
}

/// Estrutura central do índice mantida estritamente em memória RAM.
#[derive(Debug, Clone, Default)]
pub struct VaultIndex {
    /// Mapeamento de UUID do item -> Entidade catalogada.
    entities_by_id: HashMap<String, EntityEntry>,
    /// Mapeamento de slug normalizado -> UUID do item.
    slug_to_id: HashMap<String, String>,
    /// Mapeamento de título normalizado (minúsculo) -> UUID do item.
    title_to_id: HashMap<String, String>,
    /// Links que saem de um documento (source_id -> Vec<Wikilink>).
    outbound_links: HashMap<String, Vec<Wikilink>>,
    /// Índice invertido de backlinks resolvidos (target_id -> Vec<BacklinkEntry>).
    backlinks: HashMap<String, Vec<BacklinkEntry>>,
    /// Referências que apontam para nomes que ainda não têm documento (target_normalized -> Vec<BacklinkEntry>).
    unresolved_backlinks: HashMap<String, Vec<BacklinkEntry>>,
}

impl VaultIndex {
    /// Cria uma nova instância vazia de índice em memória.
    pub fn new() -> Self {
        Self::default()
    }

    /// Normaliza uma string para casamento de slug e títulos insensível a maiúsculas/acentos básicos.
    pub fn normalize_key(input: &str) -> String {
        input.trim().to_lowercase()
    }

    /// Reconstrói integralmente o índice a partir dos itens do cofre aberto.
    pub fn build_from_vault(vault: &UnlockedVault) -> Result<Self, VaultError> {
        let manifest = vault.manifest()?;
        let mut index = Self::new();

        // 1. Cadastra todas as entidades do manifesto (cenas e entidades)
        for item in &manifest.items {
            let entity = EntityEntry {
                id: item.id.clone(),
                title: item.title.clone(),
                slug: item.slug.clone(),
                kind: item.kind.clone(),
                tags: item.tags.clone(),
                status: item.status.clone(),
                summary: None,
                mention_count: 0,
            };
            index.register_entity(entity);
        }

        // 2. Para cada item que possui blob de texto, lê, decifra e analisa links
        for item in &manifest.items {
            if let Some(blob_id) = &item.blob_id {
                if let Ok(bytes) = vault.read_blob(blob_id) {
                    if let Ok(raw_text) = String::from_utf8(bytes) {
                        index.index_document_content(&item.id, &item.title, &item.kind, &raw_text);
                    }
                }
            }
        }

        // 3. Atualiza as contagens de menções das entidades
        index.recalculate_mention_counts();

        Ok(index)
    }

    /// Registra uma entidade no catálogo.
    pub fn register_entity(&mut self, entity: EntityEntry) {
        let norm_slug = Self::normalize_key(&entity.slug);
        let norm_title = Self::normalize_key(&entity.title);

        self.slug_to_id.insert(norm_slug, entity.id.clone());
        self.title_to_id.insert(norm_title, entity.id.clone());
        self.entities_by_id.insert(entity.id.clone(), entity);
    }

    /// Remove uma entidade do catálogo e limpa seus links e backlinks.
    pub fn remove_entity(&mut self, item_id: &str) {
        if let Some(removed) = self.entities_by_id.remove(item_id) {
            self.slug_to_id.remove(&Self::normalize_key(&removed.slug));
            self.title_to_id
                .remove(&Self::normalize_key(&removed.title));
        }

        // Limpa links que saíam do documento
        self.outbound_links.remove(item_id);

        // Remove backlinks onde este documento era a fonte
        for list in self.backlinks.values_mut() {
            list.retain(|b| b.source_id != item_id);
        }
        for list in self.unresolved_backlinks.values_mut() {
            list.retain(|b| b.source_id != item_id);
        }

        // Remove o balde de backlinks do próprio documento
        self.backlinks.remove(item_id);

        self.recalculate_mention_counts();
    }

    /// Atualiza ou reindexa o conteúdo textual de um documento (ao salvar ou editar).
    pub fn index_document_content(
        &mut self,
        source_id: &str,
        source_title: &str,
        source_kind: &str,
        raw_markdown: &str,
    ) {
        // Remove links antigos de saída e referências antigas deste documento
        self.clear_outbound_links_for_source(source_id);

        // Faz o parse do documento para extrair frontmatter e corpo
        let (body, summary) = match Document::parse(raw_markdown) {
            Ok(doc) => {
                let summary_text = doc
                    .extra
                    .get("summary")
                    .or_else(|| doc.extra.get("description"))
                    .or_else(|| doc.extra.get("bio"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .or_else(|| extract_first_paragraph_summary(&doc.body));

                (doc.body, summary_text)
            }
            Err(_) => (
                raw_markdown.to_string(),
                extract_first_paragraph_summary(raw_markdown),
            ),
        };

        // Atualiza o resumo da entidade se existir
        if let Some(entity) = self.entities_by_id.get_mut(source_id) {
            if summary.is_some() {
                entity.summary = summary;
            }
        }

        // Extrai todos os wikilinks do corpo Markdown
        let wikilinks = extract_wikilinks(&body);

        for link in &wikilinks {
            let norm_target = Self::normalize_key(&link.target);

            // Verifica se o alvo aponta para uma entidade conhecida (por título ou slug)
            let resolved_id = self
                .title_to_id
                .get(&norm_target)
                .or_else(|| self.slug_to_id.get(&norm_target))
                .cloned();

            let entry = BacklinkEntry {
                source_id: source_id.to_string(),
                source_title: source_title.to_string(),
                source_kind: source_kind.to_string(),
                target_query: link.target.clone(),
                alias: link.label.clone(),
                line_number: link.line_number,
                byte_offset: link.byte_offset,
                snippet: link.context_snippet.clone(),
            };

            if let Some(target_id) = resolved_id {
                self.backlinks.entry(target_id).or_default().push(entry);
            } else {
                self.unresolved_backlinks
                    .entry(norm_target)
                    .or_default()
                    .push(entry);
            }
        }

        self.outbound_links.insert(source_id.to_string(), wikilinks);
        self.recalculate_mention_counts();
    }

    /// Limpa os links emitidos por um documento de origem.
    fn clear_outbound_links_for_source(&mut self, source_id: &str) {
        self.outbound_links.remove(source_id);

        for list in self.backlinks.values_mut() {
            list.retain(|b| b.source_id != source_id);
        }
        for list in self.unresolved_backlinks.values_mut() {
            list.retain(|b| b.source_id != source_id);
        }
    }

    /// Recalcula os contadores de menções de todas as entidades ativas.
    fn recalculate_mention_counts(&mut self) {
        for (id, entity) in self.entities_by_id.iter_mut() {
            let count = self.backlinks.get(id).map(|list| list.len()).unwrap_or(0);
            entity.mention_count = count;
        }
    }

    /// Retorna uma entidade pelo seu ID único.
    pub fn get_entity(&self, id: &str) -> Option<&EntityEntry> {
        self.entities_by_id.get(id)
    }

    /// Retorna todas as entidades catalogadas no índice.
    pub fn get_all_entities(&self) -> Vec<EntityEntry> {
        self.entities_by_id.values().cloned().collect()
    }

    /// Resolve um termo digitado (ex: "Jaci" ou "jaci") para a entidade correspondente.
    pub fn resolve_wikilink(&self, query: &str) -> Option<&EntityEntry> {
        let norm = Self::normalize_key(query);
        let item_id = self
            .title_to_id
            .get(&norm)
            .or_else(|| self.slug_to_id.get(&norm))?;
        self.entities_by_id.get(item_id)
    }

    /// Retorna todos os backlinks que citam a entidade identificada por `item_id`.
    pub fn get_backlinks(&self, item_id: &str) -> Vec<BacklinkEntry> {
        self.backlinks.get(item_id).cloned().unwrap_or_default()
    }

    /// Retorna todos os links que saem de um documento de origem.
    pub fn get_outbound_links(&self, source_id: &str) -> Vec<Wikilink> {
        self.outbound_links
            .get(source_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Retorna uma visão geral de todas as entidades e estatísticas do índice.
    pub fn get_overview(&self) -> IndexOverviewResponse {
        let mut entities: Vec<EntityEntry> = self.entities_by_id.values().cloned().collect();
        entities.sort_by_key(|a| a.title.to_lowercase());

        let total_links = self.backlinks.values().map(|v| v.len()).sum();
        let unresolved_count = self.unresolved_backlinks.values().map(|v| v.len()).sum();
        let mut unresolved_targets: Vec<String> =
            self.unresolved_backlinks.keys().cloned().collect();
        unresolved_targets.sort();

        IndexOverviewResponse {
            entities,
            total_links_count: total_links,
            unresolved_links_count: unresolved_count,
            unresolved_targets,
        }
    }
}

/// Extrai todos os `[[wikilink]]` e `[[alvo|alias]]` do texto com linha, offset e trecho de contexto.
pub fn extract_wikilinks(text: &str) -> Vec<Wikilink> {
    let mut links = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut cursor = 0;
    let mut line_number = 1;
    let mut line_start_offset = 0;

    while cursor + 3 < len {
        if bytes[cursor] == b'\n' {
            line_number += 1;
            line_start_offset = cursor + 1;
            cursor += 1;
            continue;
        }

        // Verifica início de `[[`
        if bytes[cursor] == b'[' && bytes[cursor + 1] == b'[' {
            let link_start = cursor;
            let content_start = cursor + 2;
            let mut search = content_start;
            let mut found_close = false;

            // Busca o fechamento `]]` na mesma linha ou parágrafo (sem quebra de linha excessiva)
            while search + 1 < len && bytes[search] != b'\n' {
                if bytes[search] == b']' && bytes[search + 1] == b']' {
                    found_close = true;
                    break;
                }
                search += 1;
            }

            if found_close {
                let inner = &text[content_start..search];
                let raw = &text[link_start..search + 2];

                if !inner.trim().is_empty() {
                    let (target, label) = if let Some((t, l)) = inner.split_once('|') {
                        (t.trim().to_string(), Some(l.trim().to_string()))
                    } else {
                        (inner.trim().to_string(), None)
                    };

                    // Extrai contexto da linha atual
                    let line_end = text[search + 2..]
                        .find('\n')
                        .map(|pos| search + 2 + pos)
                        .unwrap_or(len);

                    let line_snippet = text[line_start_offset..line_end].trim().to_string();

                    links.push(Wikilink {
                        raw: raw.to_string(),
                        target,
                        label,
                        line_number,
                        byte_offset: link_start,
                        context_snippet: line_snippet,
                    });
                }

                cursor = search + 2;
                continue;
            }
        }

        cursor += 1;
    }

    links
}

/// Extrai os primeiros 180 caracteres do primeiro parágrafo significativo do corpo como resumo.
fn extract_first_paragraph_summary(body: &str) -> Option<String> {
    for paragraph in body.split("\n\n") {
        let trimmed = paragraph.trim();
        // Ignora títulos Markdown ou linhas vazias
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let clean = trimmed.replace('\n', " ");
            if clean.chars().count() > 180 {
                let truncated: String = clean.chars().take(177).collect();
                return Some(format!("{truncated}..."));
            }
            return Some(clean);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_and_aliased_wikilinks() {
        let text = "No porto de Maré Alta, [[Jaci]] encontrou [[Beto|Mestre Beto]].\nMais tarde foram até [[Cofre Secreto]].";
        let links = extract_wikilinks(text);

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].target, "Jaci");
        assert_eq!(links[0].label, None);
        assert_eq!(links[0].line_number, 1);
        assert!(links[0].context_snippet.contains("Maré Alta"));

        assert_eq!(links[1].target, "Beto");
        assert_eq!(links[1].label, Some("Mestre Beto".to_string()));
        assert_eq!(links[1].line_number, 1);

        assert_eq!(links[2].target, "Cofre Secreto");
        assert_eq!(links[2].label, None);
        assert_eq!(links[2].line_number, 2);
    }

    #[test]
    fn test_in_memory_index_lifecycle() {
        let mut index = VaultIndex::new();

        // 1. Cadastra entidade "Jaci" (personagem)
        index.register_entity(EntityEntry {
            id: "char_jaci_uuid".to_string(),
            title: "Jaci".to_string(),
            slug: "jaci".to_string(),
            kind: "character".to_string(),
            tags: vec!["protagonista".to_string()],
            status: Some("active".to_string()),
            summary: Some("Guardiã das correntes ancestrais.".to_string()),
            mention_count: 0,
        });

        // 2. Indexa cena 1 que cita Jaci
        let scene_1_text = "---\ntitle: Cena 1\ntype: scene\n---\nQuando o sol se pôs, [[Jaci]] olhou para o horizonte.";
        index.index_document_content("scene_1_uuid", "Cena 1", "scene", scene_1_text);

        // 3. Indexa cena 2 que cita Jaci com alias e cita uma entidade ainda inexistente
        let scene_2_text =
            "Ela conversou com [[Jaci|A Guardiã]]. Também mencionou [[Farol Abandonado]].";
        index.index_document_content("scene_2_uuid", "Cena 2", "scene", scene_2_text);

        // 4. Verifica backlinks de Jaci
        let jaci_backlinks = index.get_backlinks("char_jaci_uuid");
        assert_eq!(jaci_backlinks.len(), 2);
        assert_eq!(jaci_backlinks[0].source_title, "Cena 1");
        assert_eq!(jaci_backlinks[1].source_title, "Cena 2");
        assert_eq!(jaci_backlinks[1].alias, Some("A Guardiã".to_string()));

        // 5. Verifica contagem de menções
        let resolved = index.resolve_wikilink("jaci").expect("deve achar jaci");
        assert_eq!(resolved.mention_count, 2);

        // 6. Verifica links não resolvidos
        let overview = index.get_overview();
        assert_eq!(overview.unresolved_links_count, 1);
        assert!(overview
            .unresolved_targets
            .contains(&"farol abandonado".to_string()));

        // 7. Agora cria o Farol e reindexa
        index.register_entity(EntityEntry {
            id: "loc_farol_uuid".to_string(),
            title: "Farol Abandonado".to_string(),
            slug: "farol-abandonado".to_string(),
            kind: "location".to_string(),
            tags: vec!["cenário".to_string()],
            status: None,
            summary: None,
            mention_count: 0,
        });
        // Reindexando a cena 2 para mover de não resolvido para resolvido
        index.index_document_content("scene_2_uuid", "Cena 2", "scene", scene_2_text);

        let farol_backlinks = index.get_backlinks("loc_farol_uuid");
        assert_eq!(farol_backlinks.len(), 1);
        assert_eq!(farol_backlinks[0].source_title, "Cena 2");
    }
}

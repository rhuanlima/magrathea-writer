//! Grafo narrativo e rede de conexões do Segundo Cérebro (PRD E4 / M8-01 e M8-02).
//!
//! Constrói o modelo de grafo a partir do índice em memória (`VaultIndex`),
//! calculando conexões derivadas de wikilinks (`[[...]]`), identificando nós órfãos
//! e suportando extração de subgrafos locais com 1 ou 2 graus de profundidade.

use crate::index::VaultIndex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Representa um nó no grafo de conexões da obra literária.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphNode {
    /// Identificador único (UUID) do item no mundo.
    pub id: String,
    /// Título humano da cena, entidade ou nota.
    pub title: String,
    /// Tipo de nó ("scene", "character", "location", "item", "faction", "lore", "note", "chapter", "part").
    pub kind: String,
    /// Resumo curto para pré-visualização no hover.
    pub summary: Option<String>,
    /// Tags associadas ao item.
    pub tags: Vec<String>,
    /// Status de desenvolvimento ("draft", "revised", "final", etc.).
    pub status: Option<String>,
    /// Total de vezes que a entidade é citada em outros documentos.
    pub mention_count: usize,
    /// Total de conexões ativas (arestas) deste nó (grau de conectividade).
    pub connections_count: usize,
    /// Indica se o nó é órfão (sem nenhuma aresta que entre ou saia).
    pub is_orphan: bool,
}

/// Representa uma aresta direcionada ou bidirecional entre dois nós narrativos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    /// ID do nó de origem da conexão.
    pub source: String,
    /// ID do nó de destino da conexão.
    pub target: String,
    /// Rótulo opcional ou contexto do link.
    pub label: Option<String>,
    /// Peso da conexão (quantidade de referências entre ambos).
    pub weight: usize,
}

/// Estrutura consolidada do Grafo Global da obra.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphData {
    /// Lista de todos os nós visíveis conforme filtros aplicados.
    pub nodes: Vec<GraphNode>,
    /// Lista de arestas conectando os nós visíveis.
    pub edges: Vec<GraphEdge>,
    /// Contagem total de nós na obra.
    pub total_nodes: usize,
    /// Contagem total de conexões entre documentos.
    pub total_edges: usize,
    /// Quantidade de nós sem nenhuma conexão (órfãos).
    pub orphan_nodes_count: usize,
}

/// Estrutura de subgrafo focalizado (Grafo Local do Inspetor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalGraphData {
    /// ID do nó central em foco.
    pub center_node_id: String,
    /// Nós contidos no raio de busca (1 ou 2 graus).
    pub nodes: Vec<GraphNode>,
    /// Arestas internas do subgrafo focal.
    pub edges: Vec<GraphEdge>,
    /// Profundidade selecionada (1 ou 2).
    pub depth: usize,
}

/// Constrói o grafo global de conexões a partir do `VaultIndex` em memória.
pub fn build_global_graph(
    index: &VaultIndex,
    filter_kinds: Option<&[String]>,
    include_orphans: bool,
) -> GraphData {
    let all_entities = index.get_all_entities();
    let mut edges_map: HashMap<(String, String), usize> = HashMap::new();
    let mut node_connections: HashMap<String, usize> = HashMap::new();

    // 1. Mapeia todas as arestas existentes via links resolvidos e backlinks
    for entity in &all_entities {
        let outbound = index.get_outbound_links(&entity.id);
        for link in outbound {
            if let Some(target_entity) = index.resolve_wikilink(&link.target) {
                if target_entity.id != entity.id {
                    let key = (entity.id.clone(), target_entity.id.clone());
                    *edges_map.entry(key).or_insert(0) += 1;
                    *node_connections.entry(entity.id.clone()).or_insert(0) += 1;
                    *node_connections
                        .entry(target_entity.id.clone())
                        .or_insert(0) += 1;
                }
            }
        }
    }

    let filter_set: Option<HashSet<String>> =
        filter_kinds.map(|kinds| kinds.iter().map(|k| k.to_lowercase()).collect());

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut orphan_count = 0;

    // 2. Constrói a lista de nós respeitando os filtros e identificando órfãos
    for entity in &all_entities {
        let conn_count = *node_connections.get(&entity.id).unwrap_or(&0);
        let is_orphan = conn_count == 0;

        if is_orphan {
            orphan_count += 1;
        }

        // Filtro de tipo (kind)
        if let Some(ref set) = filter_set {
            if !set.contains(&entity.kind.to_lowercase()) {
                continue;
            }
        }

        // Filtro de órfãos
        if !include_orphans && is_orphan {
            continue;
        }

        nodes.push(GraphNode {
            id: entity.id.clone(),
            title: entity.title.clone(),
            kind: entity.kind.clone(),
            summary: entity.summary.clone(),
            tags: entity.tags.clone(),
            status: entity.status.clone(),
            mention_count: entity.mention_count,
            connections_count: conn_count,
            is_orphan,
        });
    }

    let valid_node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    // 3. Monta a lista de arestas conectando apenas nós visíveis
    let mut edges: Vec<GraphEdge> = Vec::new();
    for ((source, target), weight) in edges_map {
        if valid_node_ids.contains(&source) && valid_node_ids.contains(&target) {
            edges.push(GraphEdge {
                source,
                target,
                label: None,
                weight,
            });
        }
    }

    let total_nodes = all_entities.len();
    let total_edges = edges.len();

    GraphData {
        nodes,
        edges,
        total_nodes,
        total_edges,
        orphan_nodes_count: orphan_count,
    }
}

/// Constrói o subgrafo focalizado (Grafo Local) centrado em um nó específico.
pub fn build_local_graph(
    index: &VaultIndex,
    center_id: &str,
    depth: usize,
) -> Result<LocalGraphData, String> {
    let center_entity = index
        .get_entity(center_id)
        .ok_or_else(|| format!("Nó com ID '{}' não foi encontrado no índice.", center_id))?;

    let depth = if depth == 0 { 1 } else { depth.min(2) };

    // Coleta nós a 1 grau
    let mut depth_1_ids: HashSet<String> = HashSet::new();
    depth_1_ids.insert(center_id.to_string());

    let outbound_center = index.get_outbound_links(center_id);
    for link in outbound_center {
        if let Some(target_entity) = index.resolve_wikilink(&link.target) {
            if target_entity.id != center_id {
                depth_1_ids.insert(target_entity.id.clone());
            }
        }
    }

    let backlinks_center = index.get_backlinks(center_id);
    for backlink in backlinks_center {
        if backlink.source_id != center_id {
            depth_1_ids.insert(backlink.source_id);
        }
    }

    let mut all_included_ids = depth_1_ids.clone();

    // Se depth == 2, coleta vizinhos de segundo grau
    if depth >= 2 {
        for node_id in &depth_1_ids {
            if node_id == center_id {
                continue;
            }
            let out_links = index.get_outbound_links(node_id);
            for l in out_links {
                if let Some(target) = index.resolve_wikilink(&l.target) {
                    all_included_ids.insert(target.id.clone());
                }
            }
            let b_links = index.get_backlinks(node_id);
            for b in b_links {
                all_included_ids.insert(b.source_id);
            }
        }
    }

    // Coleta entidades e calcula conexões internas
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut node_connections: HashMap<String, usize> = HashMap::new();
    let mut edges_map: HashMap<(String, String), usize> = HashMap::new();

    for id in &all_included_ids {
        let out_links = index.get_outbound_links(id);
        for l in out_links {
            if let Some(target) = index.resolve_wikilink(&l.target) {
                if target.id != *id && all_included_ids.contains(&target.id) {
                    let key = (id.clone(), target.id.clone());
                    *edges_map.entry(key).or_insert(0) += 1;
                    *node_connections.entry(id.clone()).or_insert(0) += 1;
                    *node_connections.entry(target.id.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    for id in &all_included_ids {
        if let Some(ent) = index.get_entity(id) {
            let conn_count = *node_connections.get(id).unwrap_or(&0);
            nodes.push(GraphNode {
                id: ent.id.clone(),
                title: ent.title.clone(),
                kind: ent.kind.clone(),
                summary: ent.summary.clone(),
                tags: ent.tags.clone(),
                status: ent.status.clone(),
                mention_count: ent.mention_count,
                connections_count: conn_count,
                is_orphan: conn_count == 0,
            });
        }
    }

    let edges: Vec<GraphEdge> = edges_map
        .into_iter()
        .map(|((source, target), weight)| GraphEdge {
            source,
            target,
            label: None,
            weight,
        })
        .collect();

    Ok(LocalGraphData {
        center_node_id: center_entity.id.clone(),
        nodes,
        edges,
        depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{EntityEntry, VaultIndex};

    #[test]
    fn test_graph_generation_and_orphans() {
        let mut index = VaultIndex::new();

        index.register_entity(EntityEntry {
            id: "scene-1".to_string(),
            title: "Capítulo 1".to_string(),
            slug: "capitulo-1".to_string(),
            kind: "scene".to_string(),
            tags: vec![],
            status: Some("draft".to_string()),
            summary: None,
            mention_count: 0,
        });

        index.register_entity(EntityEntry {
            id: "char-1".to_string(),
            title: "Jaci".to_string(),
            slug: "jaci".to_string(),
            kind: "character".to_string(),
            tags: vec![],
            status: None,
            summary: Some("Protagonista".to_string()),
            mention_count: 0,
        });

        index.register_entity(EntityEntry {
            id: "char-2".to_string(),
            title: "Iara Solitária".to_string(),
            slug: "iara-solitaria".to_string(),
            kind: "character".to_string(),
            tags: vec![],
            status: None,
            summary: None,
            mention_count: 0,
        });

        // scene-1 cita Jaci
        index.index_document_content(
            "scene-1",
            "Capítulo 1",
            "scene",
            "O vento soprava para [[Jaci]].",
        );

        // Grafo com órfãos
        let graph_all = build_global_graph(&index, None, true);
        assert_eq!(graph_all.nodes.len(), 3);
        assert_eq!(graph_all.edges.len(), 1);
        assert_eq!(graph_all.orphan_nodes_count, 1);

        // Grafo sem órfãos
        let graph_no_orphans = build_global_graph(&index, None, false);
        assert_eq!(graph_no_orphans.nodes.len(), 2);
        assert!(!graph_no_orphans.nodes.iter().any(|n| n.id == "char-2"));

        // Grafo local centrado em Jaci
        let local_jaci = build_local_graph(&index, "char-1", 1).expect("deve gerar local");
        assert_eq!(local_jaci.center_node_id, "char-1");
        assert_eq!(local_jaci.nodes.len(), 2);
        assert_eq!(local_jaci.edges.len(), 1);
    }
}

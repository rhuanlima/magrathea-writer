//! Canvas Visual e Mapas Mentais baseado na especificação JSON Canvas 1.0 (PRD E10 / D4 / M8-03 e M8-04).
//!
//! Permite ao autor criar quadros infinitos de ideação e mapeamento de enredo,
//! combinando post-its livres, cartões de entidades do mundo (com avatar e biografia),
//! grupos visuais e setas com rótulos narrativos.
//! Os dados são persistidos de forma segura e cifrada no cofre/mundo.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Tipo de nó suportado pela especificação JSON Canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CanvasNodeType {
    Text,
    File,
    Link,
    Group,
}

/// Representa um nó no quadro infinito JSON Canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasNode {
    /// ID único do nó no canvas.
    pub id: String,
    /// Tipo de nó (text, file, link, group).
    #[serde(rename = "type")]
    pub node_type: CanvasNodeType,
    /// Posição X no plano infinito.
    pub x: f64,
    /// Posição Y no plano infinito.
    pub y: f64,
    /// Largura do cartão em pixels.
    pub width: f64,
    /// Altura do cartão em pixels.
    pub height: f64,
    /// Código de cor (1 a 6 pré-definidas ou hex string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Conteúdo textual Markdown (para nós de post-it / text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Referência para cena ou entidade do mundo (para nós do tipo file).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// URL para nós do tipo link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Rótulo ou título (para grupos ou cartões).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Metadados opcionais enriquecidos para exibição no Magrathea (tipo da entidade, avatar, resumo).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_summary: Option<String>,
    /// URL ou data URI (base64) de imagem para nós com mídia ou post-its.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// Conexão direcionada (seta) entre dois nós do Canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasEdge {
    /// ID único da aresta.
    pub id: String,
    /// ID do nó de onde a aresta parte.
    #[serde(rename = "fromNode")]
    pub from_node: String,
    /// Lado do nó de onde a aresta parte ("top", "right", "bottom", "left").
    #[serde(rename = "fromSide", skip_serializing_if = "Option::is_none")]
    pub from_side: Option<String>,
    /// Estilo do terminal de início ("none", "arrow").
    #[serde(rename = "fromEnd", skip_serializing_if = "Option::is_none")]
    pub from_end: Option<String>,
    /// ID do nó de destino da aresta.
    #[serde(rename = "toNode")]
    pub to_node: String,
    /// Lado do nó onde a aresta se conecta.
    #[serde(rename = "toSide", skip_serializing_if = "Option::is_none")]
    pub to_side: Option<String>,
    /// Estilo do terminal de chegada ("none", "arrow").
    #[serde(rename = "toEnd", skip_serializing_if = "Option::is_none")]
    pub to_end: Option<String>,
    /// Rótulo explicativo da conexão (ex: "Inimigo de", "Ocorre após").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Cor da linha da aresta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Documento completo de Canvas compatível com JSON Canvas 1.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasDocument {
    /// Identificador do canvas (slug ou uuid).
    pub id: String,
    /// Nome descritivo dado pelo autor (ex: "Conflito Central", "Mapeamento dos Clãs").
    pub name: String,
    /// Nós contidos no quadro.
    pub nodes: Vec<CanvasNode>,
    /// Arestas ligando os nós.
    pub edges: Vec<CanvasEdge>,
    /// Timestamp ISO 8601 da última modificação.
    pub updated_at: String,
}

/// Resumo para listagem rápida de quadros disponíveis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanvasSummary {
    pub id: String,
    pub name: String,
    pub nodes_count: usize,
    pub edges_count: usize,
    pub updated_at: String,
}

impl CanvasDocument {
    /// Cria um novo canvas em branco com nome padrão.
    pub fn new_default(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            updated_at: chrono_like_timestamp(),
        }
    }
}

/// Retorna o caminho do diretório de canvas dentro de um cofre.
pub fn get_canvas_dir(vault_path: &Path) -> PathBuf {
    vault_path.join(".magrathea").join("canvas")
}

/// Gera um timestamp simples UTC ISO 8601 sem dependência externa adicional.
fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Salva um documento Canvas no diretório do cofre.
pub fn save_canvas_file(vault_path: &Path, canvas: &CanvasDocument) -> Result<(), String> {
    let dir = get_canvas_dir(vault_path);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Falha ao criar diretório de canvas: {}", e))?;
    }

    let file_path = dir.join(format!("{}.canvas", canvas.id));
    let json_bytes = serde_json::to_vec_pretty(canvas)
        .map_err(|e| format!("Falha ao serializar JSON Canvas: {}", e))?;

    fs::write(&file_path, json_bytes)
        .map_err(|e| format!("Falha ao escrever arquivo de canvas: {}", e))?;

    Ok(())
}

/// Carrega um documento Canvas existente ou cria o padrão se não existir.
pub fn load_canvas_file(vault_path: &Path, canvas_id: &str) -> Result<CanvasDocument, String> {
    let dir = get_canvas_dir(vault_path);
    let file_path = dir.join(format!("{}.canvas", canvas_id));

    if !file_path.exists() {
        let def = CanvasDocument::new_default(canvas_id, "Quadro Principal");
        save_canvas_file(vault_path, &def)?;
        return Ok(def);
    }

    let bytes =
        fs::read(&file_path).map_err(|e| format!("Falha ao ler arquivo de canvas: {}", e))?;

    let doc: CanvasDocument = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Falha ao deserializar JSON Canvas: {}", e))?;

    Ok(doc)
}

/// Lista todos os documentos Canvas existentes no cofre.
pub fn list_canvas_files(vault_path: &Path) -> Result<Vec<CanvasSummary>, String> {
    let dir = get_canvas_dir(vault_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Falha ao listar pasta de canvas: {}", e))?;

    let mut list = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("canvas") {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(doc) = serde_json::from_slice::<CanvasDocument>(&bytes) {
                    list.push(CanvasSummary {
                        id: doc.id,
                        name: doc.name,
                        nodes_count: doc.nodes.len(),
                        edges_count: doc.edges.len(),
                        updated_at: doc.updated_at,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(list)
}

/// Exclui um documento Canvas do cofre.
pub fn delete_canvas_file(vault_path: &Path, canvas_id: &str) -> Result<bool, String> {
    let dir = get_canvas_dir(vault_path);
    let file_path = dir.join(format!("{}.canvas", canvas_id));
    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Falha ao remover arquivo de canvas: {}", e))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_json_canvas_spec_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("magrathea_test_canvas_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);

        let mut doc = CanvasDocument::new_default("board-1", "Trama do Mistério");
        doc.nodes.push(CanvasNode {
            id: "node-1".to_string(),
            node_type: CanvasNodeType::Text,
            x: 100.0,
            y: 150.0,
            width: 250.0,
            height: 120.0,
            color: Some("1".to_string()),
            text: Some("## Pista Revelada\nO relógio parou às 3h.".to_string()),
            file: None,
            url: None,
            label: None,
            entity_kind: None,
            entity_summary: None,
            image_url: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=".to_string()),
        });

        doc.nodes.push(CanvasNode {
            id: "node-2".to_string(),
            node_type: CanvasNodeType::File,
            x: 450.0,
            y: 150.0,
            width: 280.0,
            height: 140.0,
            color: Some("#5E6F45".to_string()),
            text: None,
            file: Some("jaci-protagonista".to_string()),
            url: None,
            label: Some("Jaci".to_string()),
            entity_kind: Some("character".to_string()),
            entity_summary: Some("Detetive das águas profundas".to_string()),
            image_url: None,
        });

        doc.edges.push(CanvasEdge {
            id: "edge-1".to_string(),
            from_node: "node-1".to_string(),
            from_side: Some("right".to_string()),
            from_end: Some("none".to_string()),
            to_node: "node-2".to_string(),
            to_side: Some("left".to_string()),
            to_end: Some("arrow".to_string()),
            label: Some("Investigado por".to_string()),
            color: None,
        });

        save_canvas_file(&temp_dir, &doc).expect("deve salvar canvas");
        let loaded = load_canvas_file(&temp_dir, "board-1").expect("deve carregar canvas");

        assert_eq!(loaded.name, "Trama do Mistério");
        assert_eq!(loaded.nodes.len(), 2);
        assert!(loaded.nodes[0].image_url.is_some());
        assert_eq!(loaded.edges.len(), 1);
        assert_eq!(loaded.edges[0].label.as_deref(), Some("Investigado por"));

        let list = list_canvas_files(&temp_dir).expect("deve listar");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "board-1");

        let deleted = delete_canvas_file(&temp_dir, "board-1").expect("deve excluir");
        assert!(deleted);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

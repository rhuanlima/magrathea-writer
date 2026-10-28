//! Modelo interno de documento literário Markdown com frontmatter YAML e preservação de metadados.
//!
//! Em conformidade com o PRD (E1):
//! - Texto = Markdown + frontmatter YAML.
//! - Preservação estrita de chaves desconhecidas do frontmatter (roundtrip fidelity).
//! - Métricas de palavras/caracteres calculadas sobre o corpo da narrativa (excluindo metadados).
//! - Tratamento resiliente e seguro contra fuzzing e frontmatters malformados.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Representação estruturada de um documento literário (cena, personagem, local ou nota).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Document {
    /// Identificador único (UUID) do nó, quando presente.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Título legível da cena ou entidade.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Slug semântico para links `[[wikilink]]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Tipo de nó ("scene", "character", "location", "lore", etc.).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// Status no fluxo de trabalho literário ("draft", "revised", "final").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Etiquetas temáticas livres.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Data real para a linha do tempo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Tempo relativo/fictício na narrativa.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    /// Chaves desconhecidas ou personalizadas adicionadas por plugins ou pelo autor,
    /// preservadas de forma transparente na serialização (ordenadas para determinismo).
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,

    /// Corpo da prosa literária em Markdown puro (excluindo frontmatter).
    #[serde(skip)]
    pub body: String,
}

impl Document {
    /// Cria uma nova cena padrão com título e corpo.
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            id: None,
            title: if title.trim().is_empty() {
                None
            } else {
                Some(title.trim().to_string())
            },
            slug: None,
            kind: Some("scene".to_string()),
            status: Some("draft".to_string()),
            tags: Vec::new(),
            date: None,
            time: None,
            extra: BTreeMap::new(),
            body: body.to_string(),
        }
    }

    /// Faz o parse seguro de uma string contendo (ou não) frontmatter YAML + corpo Markdown.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (yaml_content, body_content) = split_frontmatter(raw);

        match yaml_content {
            Some(yaml_str) => {
                let trimmed_yaml = yaml_str.trim();
                if trimmed_yaml.is_empty() {
                    return Ok(Document {
                        body: body_content.to_string(),
                        ..Default::default()
                    });
                }

                match serde_yaml::from_str::<Document>(trimmed_yaml) {
                    Ok(mut doc) => {
                        doc.body = body_content.to_string();
                        Ok(doc)
                    }
                    Err(err) => {
                        // Se o YAML estiver malformado, retorna erro descritivo
                        Err(format!("Erro ao processar frontmatter YAML: {err}"))
                    }
                }
            }
            None => Ok(Document {
                body: raw.to_string(),
                ..Default::default()
            }),
        }
    }

    /// Serializa o documento completo de volta para Markdown com delimitadores `---` se houver metadados.
    pub fn serialize(&self) -> Result<String, String> {
        let has_metadata = self.id.is_some()
            || self.title.is_some()
            || self.slug.is_some()
            || self.kind.is_some()
            || self.status.is_some()
            || !self.tags.is_empty()
            || self.date.is_some()
            || self.time.is_some()
            || !self.extra.is_empty();

        if !has_metadata {
            return Ok(self.body.clone());
        }

        let yaml_str = serde_yaml::to_string(self)
            .map_err(|e| format!("Falha ao serializar metadados para YAML: {e}"))?;

        let trimmed_yaml = yaml_str.trim();

        if self.body.is_empty() {
            Ok(format!("---\n{trimmed_yaml}\n---\n"))
        } else if self.body.starts_with('\n') {
            Ok(format!("---\n{trimmed_yaml}\n---{}", self.body))
        } else {
            Ok(format!("---\n{trimmed_yaml}\n---\n\n{}", self.body))
        }
    }

    /// Contagem precisa de palavras apenas da narrativa ativa (excluindo frontmatter).
    pub fn word_count(&self) -> usize {
        if self.body.trim().is_empty() {
            0
        } else {
            self.body.split_whitespace().count()
        }
    }

    /// Contagem precisa de caracteres apenas da narrativa ativa.
    pub fn char_count(&self) -> usize {
        self.body.chars().count()
    }
}

/// Separa com precisão o bloco frontmatter delimitado por `---` do corpo do documento.
/// Suporta quebras de linha Unix (`\n`) e Windows (`\r\n`).
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let normalized_start = content.strip_prefix("\u{feff}").unwrap_or(content); // remove BOM se presente

    if !normalized_start.starts_with("---") {
        return (None, content);
    }

    let remainder = &normalized_start[3..];
    let after_first_delimiter = if let Some(stripped) = remainder.strip_prefix("\r\n") {
        stripped
    } else if let Some(stripped) = remainder.strip_prefix('\n') {
        stripped
    } else {
        return (None, content);
    };

    // Busca o delimitador de fechamento `\n---` ou `\r\n---`
    let mut search_idx = 0;
    while let Some(pos) = after_first_delimiter[search_idx..].find("---") {
        let abs_pos = search_idx + pos;

        // O delimitador deve ser precedido por início de linha
        let is_line_start = if abs_pos == 0 {
            true
        } else {
            let prev_slice = &after_first_delimiter[..abs_pos];
            prev_slice.ends_with('\n') || prev_slice.ends_with("\r\n")
        };

        if is_line_start {
            let after_closing = &after_first_delimiter[abs_pos + 3..];
            // O delimitador de fechamento deve ser seguido por quebra de linha ou fim de texto
            if after_closing.is_empty()
                || after_closing.starts_with('\n')
                || after_closing.starts_with("\r\n")
            {
                let yaml_part = &after_first_delimiter[..abs_pos];
                let body_part = if let Some(b) = after_closing.strip_prefix("\r\n\r\n") {
                    b
                } else if let Some(b) = after_closing.strip_prefix("\n\n") {
                    b
                } else if let Some(b) = after_closing.strip_prefix("\r\n") {
                    b
                } else if let Some(b) = after_closing.strip_prefix('\n') {
                    b
                } else {
                    after_closing
                };

                return (Some(yaml_part), body_part);
            }
        }

        search_idx = abs_pos + 3;
    }

    (None, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown_without_frontmatter() {
        let text = "# Capítulo 1\n\nEra uma vez um escritor focado.";
        let doc = Document::parse(text).expect("deve fazer parse sem erro");
        assert_eq!(doc.body, text);
        assert_eq!(doc.title, None);
        assert_eq!(doc.word_count(), 9);
        assert_eq!(doc.char_count(), text.chars().count());
    }

    #[test]
    fn test_parse_frontmatter_with_standard_and_unknown_fields() {
        let raw = r#"---
id: "cena-001"
title: "O Encontro"
slug: "o-encontro"
type: "scene"
status: "draft"
tags:
  - climax
  - suspense
pov_character: "Arthur Dent"
custom_score: 42
plugin_settings:
  color: "olive"
  active: true
---

Arthur olhou para o céu estrelado e suspirou."#;

        let doc = Document::parse(raw).expect("deve fazer parse de frontmatter");
        assert_eq!(doc.id.as_deref(), Some("cena-001"));
        assert_eq!(doc.title.as_deref(), Some("O Encontro"));
        assert_eq!(doc.slug.as_deref(), Some("o-encontro"));
        assert_eq!(doc.kind.as_deref(), Some("scene"));
        assert_eq!(doc.status.as_deref(), Some("draft"));
        assert_eq!(doc.tags, vec!["climax", "suspense"]);

        // Preservação de chaves desconhecidas
        assert!(doc.extra.contains_key("pov_character"));
        assert_eq!(
            doc.extra.get("pov_character").unwrap().as_str(),
            Some("Arthur Dent")
        );
        assert_eq!(doc.extra.get("custom_score").unwrap().as_i64(), Some(42));
        assert!(doc.extra.contains_key("plugin_settings"));

        // Corpo
        assert_eq!(doc.body, "Arthur olhou para o céu estrelado e suspirou.");
        assert_eq!(doc.word_count(), 8); // Apenas as 8 palavras do corpo!

        // Roundtrip serialization
        let serialized = doc.serialize().expect("deve serializar com sucesso");
        let reparsed = Document::parse(&serialized).expect("deve re-parsear");
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn test_horizontal_rule_in_body_is_not_treated_as_closing_frontmatter() {
        let raw = r#"---
title: "Divisores Literários"
---

Primeira seção da narrativa.

---

Segunda seção separada por divisor horizontal."#;

        let doc = Document::parse(raw).expect("deve fazer parse");
        assert_eq!(doc.title.as_deref(), Some("Divisores Literários"));
        assert!(doc.body.contains("---"));
        assert!(doc
            .body
            .contains("Segunda seção separada por divisor horizontal."));
    }
}

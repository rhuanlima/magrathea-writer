//! Teste intensivo de Fuzzing e Roundtrip do modelo Markdown + Frontmatter: M1-05
//!
//! Requisito estrito do PRD (E1):
//! "Modelo interno Markdown+frontmatter com preservação de chaves desconhecidas + fuzz"

use magrathea_writer_lib::document::Document;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;

#[test]
fn test_frontmatter_roundtrip_property_and_unknown_keys_preservation() {
    let mut rng = StdRng::seed_from_u64(0x42_1337_cafe);

    for iteration in 0..1000 {
        let mut doc = Document {
            title: Some(format!("Capítulo Fuzz {iteration}")),
            kind: Some("scene".to_string()),
            status: Some(if iteration % 2 == 0 {
                "draft".to_string()
            } else {
                "final".to_string()
            }),
            ..Default::default()
        };

        // Adiciona tags aleatórias
        let num_tags = rng.gen_range(0..5);
        for t in 0..num_tags {
            doc.tags.push(format!("tag_{t}_{iteration}"));
        }

        // Adiciona chaves desconhecidas de múltiplos tipos de dados
        let mut extra = BTreeMap::new();
        extra.insert(
            "custom_number".to_string(),
            serde_yaml::Value::Number(rng.gen_range(-1000..1000).into()),
        );
        extra.insert(
            "custom_bool".to_string(),
            serde_yaml::Value::Bool(rng.gen_bool(0.5)),
        );
        extra.insert(
            "custom_text".to_string(),
            serde_yaml::Value::String(format!(
                "Texto extra de teste {iteration} com caracteres especiais: áéíóú ç ã"
            )),
        );

        // Subobjeto aninhado desconhecido
        let mut nested_map = serde_yaml::Mapping::new();
        nested_map.insert(
            serde_yaml::Value::String("sub_key".to_string()),
            serde_yaml::Value::String(format!("valor_aninhado_{iteration}")),
        );
        extra.insert(
            "nested_config".to_string(),
            serde_yaml::Value::Mapping(nested_map),
        );

        doc.extra = extra;

        // Corpo da cena com divisores horizontais e caracteres literários
        doc.body = format!(
            "Parágrafo 1 da narrativa {iteration}.\n\n---\n\nParágrafo 2 após divisor horizontal.\n\n***\n\n— Travessão e reticências..."
        );

        // 1. Serializa para string Markdown
        let serialized = doc.serialize().expect("deve serializar com sucesso");

        // 2. Faz o parse de volta
        let parsed = Document::parse(&serialized).expect("deve fazer o re-parse com sucesso");

        // 3. Validação de equivalência total (roundtrip e preservação de chaves)
        assert_eq!(
            doc.title, parsed.title,
            "Título divergiu na iteração {iteration}"
        );
        assert_eq!(
            doc.status, parsed.status,
            "Status divergiu na iteração {iteration}"
        );
        assert_eq!(
            doc.tags, parsed.tags,
            "Tags divergiram na iteração {iteration}"
        );
        assert_eq!(
            doc.extra, parsed.extra,
            "Chaves desconhecidas não foram preservadas na iteração {iteration}"
        );
        assert_eq!(
            doc.body, parsed.body,
            "Corpo da narrativa divergiu na iteração {iteration}"
        );
    }
}

#[test]
fn test_fuzz_malformed_and_edge_case_inputs_never_panic() {
    let mut rng = StdRng::seed_from_u64(0xdead_beef_1234);

    let sample_tokens = [
        "---",
        "title: test",
        ":::: invalid yaml ::::",
        "\r\n",
        "\n",
        "null",
        "~",
        "[1, 2, 3]",
        "{ a: 1 }",
        "body text line",
        "\0",
        "\u{feff}", // BOM
        "áéíóú",
        "🤖👽🚀",
        "\"unterminated string",
        "- item 1\n  nested without indent",
        "---",
        "---",
        "key: &anchor value",
        "reference: *anchor",
    ];

    // Executa 5000 iterações com inputs completamente aleatórios e caóticos
    for _ in 0..5000 {
        let length = rng.gen_range(1..15);
        let mut chaotic_input = String::new();
        for _ in 0..length {
            let choice = sample_tokens[rng.gen_range(0..sample_tokens.len())];
            chaotic_input.push_str(choice);
            if rng.gen_bool(0.4) {
                chaotic_input.push('\n');
            }
        }

        // A invariante fundamental de robustez: parse NUNCA pode entrar em pânico!
        let _ = Document::parse(&chaotic_input);
    }
}

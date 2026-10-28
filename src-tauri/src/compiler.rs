//! Módulo de Compilação do Manuscrito e Motores de Exportação (M6).
//!
//! Fornece unificação estruturada do livro com seções pré/pós-textuais (Dedicatória,
//! Epígrafe, Referências, Posfácio), temas de diagramação editorial e exportação
//! nativa para Obsidian Markdown, DOCX, EPUB, PDF (HTML diagramado) e TXT com Dossiê.

use crate::crypto::{decrypt_payload, encrypt_payload, Nonce, NONCE_LEN};
use crate::document::split_frontmatter;
use crate::vault::{atomic_write, current_timestamp_iso, ManifestItem, UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Configurações editoriais do compilador e seções pré/pós-textuais.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerSettings {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub dedication: Option<String>,
    pub epigraph: Option<String>,
    pub epigraph_source: Option<String>,
    pub foreword: Option<String>,
    pub afterword: Option<String>,
    pub acknowledgments: Option<String>,
    pub references: Option<String>,
    pub about_author: Option<String>,
    pub include_scene_titles: bool,
    pub scene_separator: String,
    pub page_break_after_chapter: bool,
    pub theme_preset: String, // "novel", "essay", "poetry"
}

impl Default for CompilerSettings {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            author: None,
            dedication: None,
            epigraph: None,
            epigraph_source: None,
            foreword: None,
            afterword: None,
            acknowledgments: None,
            references: None,
            about_author: None,
            include_scene_titles: false,
            scene_separator: "* * *".to_string(),
            page_break_after_chapter: true,
            theme_preset: "novel".to_string(),
        }
    }
}

/// Capítulo ou seção estruturada no livro compilado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledSection {
    pub id: String,
    pub title: String,
    pub kind: String, // "frontmatter", "part", "chapter", "scene", "backmatter"
    pub content: String,
    pub word_count: usize,
}

/// Resultado da compilação do manuscrito em memória.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledManuscript {
    pub settings: CompilerSettings,
    pub full_text: String,
    pub sections: Vec<CompiledSection>,
    pub total_words: usize,
    pub total_characters: usize,
    pub compiled_at: String,
}

/// Resultado da exportação de pasta Markdown (Obsidian).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMarkdownResult {
    pub destination_dir: String,
    pub files_count: usize,
    pub byte_size: u64,
}

/// Resultado de exportação de documento (DOCX, EPUB, PDF, TXT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDocumentResult {
    pub file_path: String,
    pub format: String,
    pub byte_size: u64,
}

/// Caminho do arquivo cifrado de configurações do compilador.
fn compiler_settings_blob_path(vault_root: &Path) -> PathBuf {
    vault_root.join("compiler_settings.blob")
}

/// Carrega as configurações do compilador salvas no cofre.
pub fn load_compiler_settings_internal(
    vault: &UnlockedVault,
) -> Result<CompilerSettings, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let path = compiler_settings_blob_path(vault.root_path());
    if !path.exists() {
        let mut def = CompilerSettings::default();
        let manifest = vault.manifest()?;
        def.title = Some(manifest.project_title.clone());
        return Ok(def);
    }

    let blob_bytes = fs::read(&path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let dek = vault.dek()?;
    let aad = format!("MAGRATHEA-V1:compiler_settings:{}", vault.header().vault_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    let settings: CompilerSettings =
        serde_json::from_slice(&plaintext).map_err(|e| VaultError::Serialization(e.to_string()))?;

    Ok(settings)
}

/// Salva as configurações de compilação cifradas no cofre de forma atômica.
pub fn save_compiler_settings_internal(
    vault: &UnlockedVault,
    settings: &CompilerSettings,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let json_bytes = serde_json::to_vec_pretty(settings)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;

    let dek = vault.dek()?;
    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:compiler_settings:{}", vault.header().vault_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &json_bytes)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_bytes());
    blob.extend_from_slice(&ciphertext);

    let path = compiler_settings_blob_path(vault.root_path());
    atomic_write(&path, &blob)?;

    Ok(())
}

/// Compila todo o manuscrito do cofre em ordem unificada aplicando as configurações editoriais.
pub fn compile_manuscript_internal(
    vault: &UnlockedVault,
    settings: Option<CompilerSettings>,
) -> Result<CompiledManuscript, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let cfg = match settings {
        Some(s) => s,
        None => load_compiler_settings_internal(vault)?,
    };

    let manifest = vault.manifest()?;
    let mut sections = Vec::new();

    // 1. Folha de Rosto / Metadados
    let book_title = cfg
        .title
        .as_deref()
        .unwrap_or(manifest.project_title.as_str());
    let author_name = cfg.author.as_deref().unwrap_or("Autor Desconhecido");

    // 2. Front Matter: Epígrafe
    if let Some(ref epigraph) = cfg.epigraph {
        if !epigraph.trim().is_empty() {
            let mut ep_text = epigraph.trim().to_string();
            if let Some(ref src) = cfg.epigraph_source {
                if !src.trim().is_empty() {
                    ep_text.push_str("\n\n— ");
                    ep_text.push_str(src.trim());
                }
            }
            sections.push(CompiledSection {
                id: "epigraph".to_string(),
                title: "Epígrafe".to_string(),
                kind: "frontmatter".to_string(),
                content: ep_text.clone(),
                word_count: ep_text.split_whitespace().count(),
            });
        }
    }

    // 3. Front Matter: Dedicatória
    if let Some(ref dedication) = cfg.dedication {
        if !dedication.trim().is_empty() {
            sections.push(CompiledSection {
                id: "dedication".to_string(),
                title: "Dedicatória".to_string(),
                kind: "frontmatter".to_string(),
                content: dedication.trim().to_string(),
                word_count: dedication.split_whitespace().count(),
            });
        }
    }

    // 4. Front Matter: Prefácio
    if let Some(ref foreword) = cfg.foreword {
        if !foreword.trim().is_empty() {
            sections.push(CompiledSection {
                id: "foreword".to_string(),
                title: "Prefácio".to_string(),
                kind: "frontmatter".to_string(),
                content: foreword.trim().to_string(),
                word_count: foreword.split_whitespace().count(),
            });
        }
    }

    // 5. Miolo: Varredura recursiva de partes, capítulos e cenas (excluindo entidades de worldbuilding)
    let manuscript_items: Vec<_> = manifest
        .items
        .iter()
        .filter(|i| matches!(i.kind.as_str(), "part" | "chapter" | "scene"))
        .cloned()
        .collect();

    let roots: Vec<_> = manuscript_items
        .iter()
        .filter(|i| i.parent_id.is_none())
        .collect();

    for root in roots {
        compile_tree_item(vault, root, &manuscript_items, &cfg, &mut sections)?;
    }

    // 6. Back Matter: Posfácio
    if let Some(ref afterword) = cfg.afterword {
        if !afterword.trim().is_empty() {
            sections.push(CompiledSection {
                id: "afterword".to_string(),
                title: "Posfácio".to_string(),
                kind: "backmatter".to_string(),
                content: afterword.trim().to_string(),
                word_count: afterword.split_whitespace().count(),
            });
        }
    }

    // 7. Back Matter: Agradecimentos
    if let Some(ref acks) = cfg.acknowledgments {
        if !acks.trim().is_empty() {
            sections.push(CompiledSection {
                id: "acknowledgments".to_string(),
                title: "Agradecimentos".to_string(),
                kind: "backmatter".to_string(),
                content: acks.trim().to_string(),
                word_count: acks.split_whitespace().count(),
            });
        }
    }

    // 8. Back Matter: Referências
    if let Some(ref refs) = cfg.references {
        if !refs.trim().is_empty() {
            sections.push(CompiledSection {
                id: "references".to_string(),
                title: "Referências e Leituras".to_string(),
                kind: "backmatter".to_string(),
                content: refs.trim().to_string(),
                word_count: refs.split_whitespace().count(),
            });
        }
    }

    // 9. Back Matter: Sobre o Autor
    if let Some(ref about) = cfg.about_author {
        if !about.trim().is_empty() {
            sections.push(CompiledSection {
                id: "about_author".to_string(),
                title: "Sobre o Autor".to_string(),
                kind: "backmatter".to_string(),
                content: about.trim().to_string(),
                word_count: about.split_whitespace().count(),
            });
        }
    }

    // Monta o texto unificado completo
    let mut full_text = String::new();
    full_text.push_str(&format!("# {}\n", book_title));
    if let Some(ref sub) = cfg.subtitle {
        if !sub.trim().is_empty() {
            full_text.push_str(&format!("*{ }*\n\n", sub.trim()));
        }
    }
    full_text.push_str(&format!("**Por {}**\n\n", author_name));

    let mut total_words = full_text.split_whitespace().count();
    let mut total_characters = full_text.chars().count();

    for sec in &sections {
        full_text.push_str("\n\n---\n\n");
        match sec.kind.as_str() {
            "part" => full_text.push_str(&format!("# {}\n\n", sec.title)),
            "chapter" => full_text.push_str(&format!("## {}\n\n", sec.title)),
            "frontmatter" | "backmatter" => full_text.push_str(&format!("## {}\n\n", sec.title)),
            _ => {
                if cfg.include_scene_titles {
                    full_text.push_str(&format!("### {}\n\n", sec.title));
                }
            }
        }
        full_text.push_str(&sec.content);
        full_text.push('\n');

        total_words += sec.word_count;
        total_characters += sec.content.chars().count();
    }

    Ok(CompiledManuscript {
        settings: cfg,
        full_text,
        sections,
        total_words,
        total_characters,
        compiled_at: current_timestamp_iso(),
    })
}

/// Limpa marcas técnicas de escrita literária para a publicação final da obra.
/// - Converte `[[Alvo|Alias]]` para `Alias`
/// - Converte `[[Alvo]]` para `Alvo`
/// - Converte `[Alias](Alvo)` (links narrativos indiretos) para `Alias`
/// - Remove metadados e tags técnicas ocultas como `#data_20261027`, `#tempo_...`
pub fn clean_literary_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' && chars.peek() == Some(&'[') {
            chars.next(); // Consome '['
            let mut inner = String::new();
            let mut closed = false;
            while let Some(c) = chars.next() {
                if c == ']' && chars.peek() == Some(&']') {
                    chars.next(); // Consome ']'
                    closed = true;
                    break;
                }
                inner.push(c);
            }
            if closed {
                if let Some((_, alias)) = inner.split_once('|') {
                    result.push_str(alias.trim());
                } else {
                    result.push_str(inner.trim());
                }
            } else {
                result.push_str("[[");
                result.push_str(&inner);
            }
        } else if ch == '[' {
            let mut label = String::new();
            let mut has_bracket = false;
            for c in chars.by_ref() {
                if c == ']' {
                    has_bracket = true;
                    break;
                }
                label.push(c);
            }
            if has_bracket && chars.peek() == Some(&'(') {
                chars.next(); // Consome '('
                let mut target = String::new();
                let mut has_paren = false;
                for c in chars.by_ref() {
                    if c == ')' {
                        has_paren = true;
                        break;
                    }
                    target.push(c);
                }
                if has_paren {
                    let target_trim = target.trim();
                    if !target_trim.starts_with("http://") && !target_trim.starts_with("https://") {
                        result.push_str(&label);
                    } else {
                        result.push('[');
                        result.push_str(&label);
                        result.push_str("](");
                        result.push_str(&target);
                        result.push(')');
                    }
                } else {
                    result.push('[');
                    result.push_str(&label);
                    result.push_str("](");
                    result.push_str(&target);
                }
            } else {
                result.push('[');
                result.push_str(&label);
                if has_bracket {
                    result.push(']');
                }
            }
        } else if ch == '#' {
            let mut tag = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    tag.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if tag.starts_with("data_")
                || tag.starts_with("date_")
                || tag.starts_with("tempo_")
                || tag.starts_with("time_")
            {
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            } else {
                result.push('#');
                result.push_str(&tag);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn compile_tree_item(
    vault: &UnlockedVault,
    item: &ManifestItem,
    all_items: &[ManifestItem],
    cfg: &CompilerSettings,
    sections: &mut Vec<CompiledSection>,
) -> Result<(), VaultError> {
    match item.kind.as_str() {
        "part" => {
            let mut part_content = String::new();
            if let Some(ref blob_id) = item.blob_id {
                if let Ok(raw_bytes) = vault.read_blob(blob_id) {
                    let text = String::from_utf8_lossy(&raw_bytes);
                    let (_, body) = split_frontmatter(&text);
                    let cleaned = clean_literary_markdown(body.trim());
                    part_content.push_str(&cleaned);
                }
            }
            let word_count = part_content.split_whitespace().count();
            sections.push(CompiledSection {
                id: item.id.clone(),
                title: item.title.clone(),
                kind: "part".to_string(),
                content: part_content,
                word_count,
            });
            let children: Vec<_> = all_items
                .iter()
                .filter(|i| i.parent_id.as_deref() == Some(&item.id))
                .collect();
            for child in children {
                compile_tree_item(vault, child, all_items, cfg, sections)?;
            }
        }
        "chapter" => {
            let mut chapter_content = String::new();

            // 1. Se o próprio capítulo possui texto direto (abertura/introdução ou capítulo contínuo)
            if let Some(ref blob_id) = item.blob_id {
                if let Ok(raw_bytes) = vault.read_blob(blob_id) {
                    let text = String::from_utf8_lossy(&raw_bytes);
                    let (_, body) = split_frontmatter(&text);
                    let cleaned = clean_literary_markdown(body.trim());
                    if !cleaned.is_empty() {
                        chapter_content.push_str(&cleaned);
                    }
                }
            }

            // 2. Cenas filhas pertencentes a este capítulo
            let scenes: Vec<_> = all_items
                .iter()
                .filter(|i| i.parent_id.as_deref() == Some(&item.id))
                .collect();

            for (idx, scene) in scenes.iter().enumerate() {
                let has_previous = !chapter_content.is_empty() || idx > 0;
                if has_previous {
                    chapter_content.push_str(&format!("\n\n{}\n\n", cfg.scene_separator));
                }
                if cfg.include_scene_titles {
                    chapter_content.push_str(&format!("### {}\n\n", scene.title));
                }

                if let Some(ref blob_id) = scene.blob_id {
                    let raw_bytes = vault.read_blob(blob_id)?;
                    let scene_text = String::from_utf8_lossy(&raw_bytes);
                    let (_, body) = split_frontmatter(&scene_text);
                    let cleaned = clean_literary_markdown(body.trim());
                    chapter_content.push_str(&cleaned);
                }
            }

            let word_count = chapter_content.split_whitespace().count();
            sections.push(CompiledSection {
                id: item.id.clone(),
                title: item.title.clone(),
                kind: "chapter".to_string(),
                content: chapter_content,
                word_count,
            });
        }
        "scene" => {
            let mut content = String::new();
            if let Some(ref blob_id) = item.blob_id {
                let raw_bytes = vault.read_blob(blob_id)?;
                let scene_text = String::from_utf8_lossy(&raw_bytes);
                let (_, body) = split_frontmatter(&scene_text);
                let cleaned = clean_literary_markdown(body.trim());
                content.push_str(&cleaned);
            }
            let word_count = content.split_whitespace().count();
            sections.push(CompiledSection {
                id: item.id.clone(),
                title: item.title.clone(),
                kind: "scene".to_string(),
                content,
                word_count,
            });
        }
        _ => {}
    }
    Ok(())
}

/// M6-01: Exporta o cofre literário aberto em formato Markdown para uma pasta (Obsidian).
pub fn export_obsidian_markdown_internal(
    vault: &UnlockedVault,
    destination_dir: &Path,
) -> Result<ExportMarkdownResult, VaultError> {
    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir).map_err(|e| VaultError::Io(e.to_string()))?;
    }

    let mut files_count = 0;
    let mut total_bytes = 0u64;

    // 1. Manuscrito
    let manuscript_dir = destination_dir.join("manuscrito");
    fs::create_dir_all(&manuscript_dir).map_err(|e| VaultError::Io(e.to_string()))?;

    let manifest = vault.manifest()?;
    for item in manifest
        .items
        .iter()
        .filter(|i| matches!(i.kind.as_str(), "part" | "chapter" | "scene"))
    {
        if let Some(ref blob_id) = item.blob_id {
            let raw_bytes = vault.read_blob(blob_id)?;
            let content = String::from_utf8_lossy(&raw_bytes).to_string();
            let file_name = format!("{}.md", sanitize_filename(&item.title));
            let file_path = manuscript_dir.join(file_name);
            fs::write(&file_path, &content).map_err(|e| VaultError::Io(e.to_string()))?;
            files_count += 1;
            total_bytes += content.len() as u64;
        }
    }

    // 2. Criação de Mundo (Worldbuilding)
    let world_dir = destination_dir.join("mundo");
    fs::create_dir_all(&world_dir).map_err(|e| VaultError::Io(e.to_string()))?;

    let entities: Vec<_> = manifest
        .items
        .iter()
        .filter(|i| !matches!(i.kind.as_str(), "part" | "chapter" | "scene"))
        .collect();

    for ent in entities {
        let content = if let Some(ref blob_id) = ent.blob_id {
            let raw_bytes = vault.read_blob(blob_id)?;
            String::from_utf8_lossy(&raw_bytes).to_string()
        } else {
            String::new()
        };
        let file_name = format!("{}.md", sanitize_filename(&ent.title));
        let file_path = world_dir.join(file_name);
        fs::write(&file_path, &content).map_err(|e| VaultError::Io(e.to_string()))?;
        files_count += 1;
        total_bytes += content.len() as u64;
    }

    // 3. Imagens da Galeria (Anexos descriptografados)
    let attach_dir = destination_dir.join("anexos");
    let images_index = crate::worldbuilding::load_images_index(vault)?;
    let all_images: Vec<_> = images_index.values().flat_map(|list| list.iter()).collect();

    if !all_images.is_empty() {
        fs::create_dir_all(&attach_dir).map_err(|e| VaultError::Io(e.to_string()))?;
        for img in all_images {
            if let Ok(data) = crate::worldbuilding::load_entity_image_bytes(vault, &img.id) {
                let file_path = attach_dir.join(&img.file_name);
                fs::write(&file_path, &data).map_err(|e| VaultError::Io(e.to_string()))?;
                files_count += 1;
                total_bytes += data.len() as u64;
            }
        }
    }

    // 4. Livro Completo Compilado
    let compiled = compile_manuscript_internal(vault, None)?;
    let book_file = destination_dir.join("LIVRO_COMPILADO.md");
    fs::write(&book_file, &compiled.full_text).map_err(|e| VaultError::Io(e.to_string()))?;
    files_count += 1;
    total_bytes += compiled.full_text.len() as u64;

    Ok(ExportMarkdownResult {
        destination_dir: destination_dir.to_string_lossy().to_string(),
        files_count,
        byte_size: total_bytes,
    })
}

/// M6-06: Exporta o manuscrito compilado para arquivo DOCX (Microsoft Word).
pub fn export_docx_internal(
    vault: &UnlockedVault,
    destination_file: &Path,
    settings: Option<CompilerSettings>,
) -> Result<ExportDocumentResult, VaultError> {
    let compiled = compile_manuscript_internal(vault, settings)?;
    let file = fs::File::create(destination_file).map_err(|e| VaultError::Io(e.to_string()))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#).map_err(|e| VaultError::Io(e.to_string()))?;

    // _rels/.rels
    zip.start_file("_rels/.rels", options)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#).map_err(|e| VaultError::Io(e.to_string()))?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", options)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#).map_err(|e| VaultError::Io(e.to_string()))?;

    // word/styles.xml
    zip.start_file("word/styles.xml", options)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault>
      <w:rPr>
        <w:rFonts w:ascii="Georgia" w:hAnsi="Georgia" w:cs="Georgia"/>
        <w:sz w:val="24"/>
        <w:szCs w:val="24"/>
      </w:rPr>
    </w:rPrDefault>
  </w:docDefaults>
</w:styles>"#,
    )
    .map_err(|e| VaultError::Io(e.to_string()))?;

    // word/document.xml
    zip.start_file("word/document.xml", options)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    let mut doc_xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
"#,
    );

    // Título do Livro
    let title = compiled.settings.title.as_deref().unwrap_or("Manuscrito");
    let author = compiled.settings.author.as_deref().unwrap_or("");
    doc_xml.push_str(&format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:before="720" w:after="240"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="56"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
        xml_escape(title)
    ));
    if !author.is_empty() {
        doc_xml.push_str(&format!(
            r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="1440"/></w:pPr><w:r><w:rPr><w:i/><w:sz w:val="28"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
            xml_escape(author)
        ));
    }

    // Seções
    for (i, sec) in compiled.sections.iter().enumerate() {
        if i > 0
            && (sec.kind == "chapter"
                || sec.kind == "part"
                || sec.kind == "frontmatter"
                || sec.kind == "backmatter")
        {
            doc_xml.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        }

        if !sec.title.is_empty() {
            let heading_size = if sec.kind == "part" { "44" } else { "36" };
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:spacing w:before="480" w:after="240"/><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="{}"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                heading_size,
                xml_escape(&sec.title)
            ));
        }

        for para in sec.content.split("\n\n") {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "* * *" || trimmed == "---" {
                doc_xml.push_str(r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:before="240" w:after="240"/></w:pPr><w:r><w:t>* * *</w:t></w:r></w:p>"#);
            } else {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:spacing w:after="120" w:line="360" w:lineRule="auto"/><w:ind w:firstLine="480"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>"#,
                    xml_escape(trimmed)
                ));
            }
        }
    }

    doc_xml.push_str(r#"</w:body></w:document>"#);
    zip.write_all(doc_xml.as_bytes())
        .map_err(|e| VaultError::Io(e.to_string()))?;
    zip.finish()
        .map_err(|e| VaultError::Serialization(e.to_string()))?;

    let meta = fs::metadata(destination_file).map_err(|e| VaultError::Io(e.to_string()))?;
    Ok(ExportDocumentResult {
        file_path: destination_file.to_string_lossy().to_string(),
        format: "docx".to_string(),
        byte_size: meta.len(),
    })
}

/// M6-07: Exporta o manuscrito compilado para arquivo EPUB (e-book padrão EPUB3/EPUB2).
pub fn export_epub_internal(
    vault: &UnlockedVault,
    destination_file: &Path,
    settings: Option<CompilerSettings>,
) -> Result<ExportDocumentResult, VaultError> {
    let compiled = compile_manuscript_internal(vault, settings)?;
    let file = fs::File::create(destination_file).map_err(|e| VaultError::Io(e.to_string()))?;
    let mut zip = ZipWriter::new(file);

    // Regra estrita do EPUB: "mimetype" DEVE ser o primeiro arquivo e NÃO PODE ter compressão (STORED).
    let stored_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", stored_opts)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(b"application/epub+zip")
        .map_err(|e| VaultError::Io(e.to_string()))?;

    let deflated_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflated_opts)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )
    .map_err(|e| VaultError::Io(e.to_string()))?;

    // OEBPS/style.css
    zip.start_file("OEBPS/style.css", deflated_opts)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    zip.write_all(
        br#"
body {
  font-family: Georgia, serif;
  margin: 5% 8%;
  line-height: 1.5;
  color: #111;
}
h1, h2, h3 {
  text-align: center;
  margin-top: 1.5em;
  margin-bottom: 0.8em;
}
p {
  margin-top: 0;
  margin-bottom: 0.5em;
  text-indent: 1.5em;
}
p.scene-break {
  text-align: center;
  text-indent: 0;
  margin: 1.5em 0;
}
.dedication, .epigraph {
  text-align: center;
  font-style: italic;
  margin: 3em auto;
  max-width: 80%;
}
"#,
    )
    .map_err(|e| VaultError::Io(e.to_string()))?;

    let title = compiled.settings.title.as_deref().unwrap_or("Manuscrito");
    let author = compiled
        .settings
        .author
        .as_deref()
        .unwrap_or("Autor Desconhecido");

    // Capítulos em arquivos XHTML individuais
    for (idx, sec) in compiled.sections.iter().enumerate() {
        let xhtml_name = format!("OEBPS/section_{}.xhtml", idx);
        zip.start_file(&xhtml_name, deflated_opts)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;

        let mut xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="pt-BR">
<head>
  <title>{}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
  <h2>{}</h2>
"#,
            xml_escape(&sec.title),
            xml_escape(&sec.title)
        );

        for para in sec.content.split("\n\n") {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "* * *" || trimmed == "---" {
                xhtml.push_str(r#"  <p class="scene-break">* * *</p>"#);
            } else {
                xhtml.push_str(&format!("  <p>{}</p>\n", xml_escape(trimmed)));
            }
        }

        xhtml.push_str("</body>\n</html>");
        zip.write_all(xhtml.as_bytes())
            .map_err(|e| VaultError::Io(e.to_string()))?;
    }

    // OEBPS/toc.ncx (EPUB 2 / Kindle compatibility)
    zip.start_file("OEBPS/toc.ncx", deflated_opts)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    let mut ncx = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:magrathea-writer-export"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{}</text></docTitle>
  <navMap>
"#,
        xml_escape(title)
    );

    for (idx, sec) in compiled.sections.iter().enumerate() {
        ncx.push_str(&format!(
            r#"    <navPoint id="navPoint-{}" playOrder="{}">
      <navLabel><text>{}</text></navLabel>
      <content src="section_{}.xhtml"/>
    </navPoint>
"#,
            idx + 1,
            idx + 1,
            xml_escape(&sec.title),
            idx
        ));
    }
    ncx.push_str("  </navMap>\n</ncx>");
    zip.write_all(ncx.as_bytes())
        .map_err(|e| VaultError::Io(e.to_string()))?;

    // OEBPS/content.opf
    zip.start_file("OEBPS/content.opf", deflated_opts)
        .map_err(|e| VaultError::Serialization(e.to_string()))?;
    let mut opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="BookID" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <dc:language>pt-BR</dc:language>
    <dc:identifier id="BookID">urn:uuid:magrathea-writer-export</dc:identifier>
    <meta property="dcterms:modified">{}</meta>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="style" href="style.css" media-type="text/css"/>
"#,
        xml_escape(title),
        xml_escape(author),
        compiled.compiled_at
    );

    for (idx, _) in compiled.sections.iter().enumerate() {
        opf.push_str(&format!(
            r#"    <item id="sec_{}" href="section_{}.xhtml" media-type="application/xhtml+xml"/>
"#,
            idx, idx
        ));
    }

    opf.push_str("  </manifest>\n  <spine toc=\"ncx\">\n");
    for (idx, _) in compiled.sections.iter().enumerate() {
        opf.push_str(&format!(
            r#"    <itemref idref="sec_{}"/>
"#,
            idx
        ));
    }
    opf.push_str("  </spine>\n</package>");
    zip.write_all(opf.as_bytes())
        .map_err(|e| VaultError::Io(e.to_string()))?;

    zip.finish()
        .map_err(|e| VaultError::Serialization(e.to_string()))?;

    let meta = fs::metadata(destination_file).map_err(|e| VaultError::Io(e.to_string()))?;
    Ok(ExportDocumentResult {
        file_path: destination_file.to_string_lossy().to_string(),
        format: "epub".to_string(),
        byte_size: meta.len(),
    })
}

/// M6-05: Exporta para HTML diagramado e pronto para impressão em PDF (Print CSS).
pub fn export_pdf_html_internal(
    vault: &UnlockedVault,
    destination_file: &Path,
    settings: Option<CompilerSettings>,
) -> Result<ExportDocumentResult, VaultError> {
    let compiled = compile_manuscript_internal(vault, settings)?;
    let title = compiled.settings.title.as_deref().unwrap_or("Manuscrito");
    let author = compiled.settings.author.as_deref().unwrap_or("");

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <title>{}</title>
  <style>
    @page {{
      size: A5;
      margin: 22mm 18mm 20mm 18mm;
      @bottom-center {{
        content: counter(page);
        font-family: Georgia, serif;
        font-size: 9pt;
      }}
    }}
    body {{
      font-family: 'EB Garamond', Georgia, serif;
      font-size: 11pt;
      line-height: 1.5;
      color: #111;
      margin: 0;
      padding: 0;
      text-rendering: optimizeLegibility;
    }}
    .title-page {{
      page-break-after: always;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 80vh;
      text-align: center;
    }}
    .book-title {{
      font-size: 26pt;
      margin-bottom: 8pt;
      font-weight: bold;
    }}
    .book-subtitle {{
      font-size: 14pt;
      font-style: italic;
      margin-bottom: 24pt;
      color: #444;
    }}
    .book-author {{
      font-size: 13pt;
      font-weight: 500;
    }}
    .section-break {{
      page-break-before: always;
    }}
    h2.section-heading {{
      text-align: center;
      font-size: 16pt;
      margin-top: 30mm;
      margin-bottom: 12mm;
      letter-spacing: 0.5px;
    }}
    p {{
      text-indent: 6mm;
      margin-top: 0;
      margin-bottom: 0;
      text-align: justify;
    }}
    p.scene-break {{
      text-indent: 0;
      text-align: center;
      margin: 12pt 0;
    }}
    .dedication-box {{
      page-break-before: always;
      page-break-after: always;
      margin: 35vh auto 0 auto;
      text-align: center;
      font-style: italic;
      max-width: 80%;
      text-indent: 0;
    }}
  </style>
</head>
<body>
  <div class="title-page">
    <div class="book-title">{}</div>
"#,
        xml_escape(title),
        xml_escape(title)
    );

    if let Some(ref sub) = compiled.settings.subtitle {
        if !sub.trim().is_empty() {
            html.push_str(&format!(
                r#"    <div class="book-subtitle">{}</div>"#,
                xml_escape(sub.trim())
            ));
        }
    }
    if !author.is_empty() {
        html.push_str(&format!(
            r#"    <div class="book-author">{}</div>"#,
            xml_escape(author)
        ));
    }
    html.push_str("  </div>\n");

    // Seções
    for sec in &compiled.sections {
        html.push_str(r#"  <div class="section-break">"#);
        if sec.id == "dedication" || sec.id == "epigraph" {
            html.push_str(&format!(
                r#"    <div class="dedication-box">{}</div>"#,
                xml_escape(&sec.content).replace('\n', "<br/>")
            ));
        } else {
            if !sec.title.is_empty() {
                html.push_str(&format!(
                    r#"    <h2 class="section-heading">{}</h2>"#,
                    xml_escape(&sec.title)
                ));
            }
            for para in sec.content.split("\n\n") {
                let trimmed = para.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "* * *" || trimmed == "---" {
                    html.push_str(r#"    <p class="scene-break">* * *</p>"#);
                } else {
                    html.push_str(&format!("    <p>{}</p>\n", xml_escape(trimmed)));
                }
            }
        }
        html.push_str("  </div>\n");
    }

    html.push_str("</body>\n</html>");
    fs::write(destination_file, &html).map_err(|e| VaultError::Io(e.to_string()))?;

    let meta = fs::metadata(destination_file).map_err(|e| VaultError::Io(e.to_string()))?;
    Ok(ExportDocumentResult {
        file_path: destination_file.to_string_lossy().to_string(),
        format: "html_pdf".to_string(),
        byte_size: meta.len(),
    })
}

/// M6-08: Exporta para arquivo TXT contínuo limpo mais dossiê literário em pasta.
pub fn export_txt_dossier_internal(
    vault: &UnlockedVault,
    destination_dir: &Path,
    settings: Option<CompilerSettings>,
) -> Result<ExportDocumentResult, VaultError> {
    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir).map_err(|e| VaultError::Io(e.to_string()))?;
    }

    let compiled = compile_manuscript_internal(vault, settings)?;
    let main_txt = destination_dir.join("manuscrito_completo.txt");
    fs::write(&main_txt, &compiled.full_text).map_err(|e| VaultError::Io(e.to_string()))?;

    // Gera dossiê literário em txt com notas de mundo
    let mut dossier = String::from(
        "DOSSIÊ DE CRIAÇÃO DE MUNDO & PERSONAGENS\n=========================================\n\n",
    );
    let manifest = vault.manifest()?;
    let entities: Vec<_> = manifest
        .items
        .iter()
        .filter(|i| !matches!(i.kind.as_str(), "part" | "chapter" | "scene"))
        .collect();

    for ent in entities {
        dossier.push_str(&format!("### {} [{}]\n", ent.title, ent.kind));
        if !ent.tags.is_empty() {
            dossier.push_str(&format!("Tags: {}\n", ent.tags.join(", ")));
        }
        if let Some(ref blob_id) = ent.blob_id {
            if let Ok(raw_bytes) = vault.read_blob(blob_id) {
                let sheet = String::from_utf8_lossy(&raw_bytes);
                let (_, body) = split_frontmatter(&sheet);
                dossier.push_str(body.trim());
            }
        }
        dossier.push_str("\n\n-----------------------------------------\n\n");
    }

    let dossier_txt = destination_dir.join("dossie_de_criacao.txt");
    fs::write(&dossier_txt, &dossier).map_err(|e| VaultError::Io(e.to_string()))?;

    let meta = fs::metadata(&main_txt).map_err(|e| VaultError::Io(e.to_string()))?;
    Ok(ExportDocumentResult {
        file_path: main_txt.to_string_lossy().to_string(),
        format: "txt".to_string(),
        byte_size: meta.len(),
    })
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_literary_markdown_wikilinks_and_indirect_links() {
        let input = "A [[Jaci]] estava na [[Casa da Jaci|casa dela]] quando acordou... Usou a [espada](Espada Relíquia) para lutar. #data_20261027";
        let cleaned = clean_literary_markdown(input);
        assert_eq!(
            cleaned,
            "A Jaci estava na casa dela quando acordou... Usou a espada para lutar. "
        );
    }

    #[test]
    fn test_clean_literary_markdown_preserves_external_urls() {
        let input = "Consulte [este link](https://example.com) para mais informações.";
        let cleaned = clean_literary_markdown(input);
        assert_eq!(cleaned, input);
    }
}

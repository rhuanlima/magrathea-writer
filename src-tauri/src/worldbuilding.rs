//! Módulo de Worldbuilding, Enciclopédia Literária e Galeria de Imagens Cifradas (Épico 5).
//!
//! Requisitos atendidos:
//! - M4-01: Templates de ficha por `type` (personagem, local, objeto, facção, lore).
//! - M4-02: Ficha de personagem guiada com perguntas provocativas e arco narrativo.
//! - M4-03: Enciclopédia com listagem, busca e filtros.
//! - M4-04: Galeria de imagens com validação estrita de magic bytes e cifragem no cofre.
//! - M4-05: Rastreio automático de aparições em cenas derivado dos backlinks.

use crate::crypto::{decrypt_payload, encrypt_payload, Nonce, NONCE_LEN};
use crate::index::VaultIndex;
use crate::vault::{atomic_write, UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Limite máximo permitido por arquivo de imagem para evitar DoS em memória (5 MiB).
pub const MAX_IMAGE_BYTE_SIZE: usize = 5 * 1024 * 1024;

/// Metadados públicos de uma imagem anexada a uma entidade literária.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityImageMeta {
    /// Identificador único da imagem no cofre.
    pub id: String,
    /// Identificador da entidade à qual a imagem pertence.
    pub entity_id: String,
    /// Nome original do arquivo (ex: "mapa_costa_norte.png").
    pub file_name: String,
    /// Tipo MIME validado via magic bytes (ex: "image/png").
    pub mime_type: String,
    /// Tamanho do arquivo em bytes.
    pub byte_size: usize,
    /// Data ISO de importação.
    pub created_at: String,
}

/// Dados completos da imagem para renderização segura em memória no frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityImageData {
    /// Metadados descritivos da imagem.
    pub meta: EntityImageMeta,
    /// Conteúdo decifrado codificado em Base64 para exibição como Data URL.
    pub base64_data: String,
}

/// Registro de aparição de uma entidade em uma cena do manuscrito.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityAppearance {
    /// UUID da cena onde a entidade foi mencionada.
    pub scene_id: String,
    /// Título legível da cena.
    pub scene_title: String,
    /// Número da linha na cena.
    pub line_number: usize,
    /// Trecho do parágrafo contendo a menção para contexto literário.
    pub snippet: String,
    /// Pseudônimo ou alias utilizado no link, se houver (`[[Alvo|Alias]]`).
    pub alias: Option<String>,
}

/// Valida os magic bytes de um buffer binário e determina seu MIME type seguro.
///
/// Suporta apenas formatos estritamente gráficos: PNG, JPEG, WebP e GIF.
/// Rejeita executáveis, scripts ou arquivos disfarçados.
pub fn validate_image_magic_bytes(bytes: &[u8]) -> Result<&'static str, &'static str> {
    if bytes.is_empty() {
        return Err("O arquivo de imagem está vazio.");
    }
    if bytes.len() > MAX_IMAGE_BYTE_SIZE {
        return Err("A imagem excede o tamanho máximo permitido de 5 MB.");
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok("image/png");
    }

    // JPEG: FF D8 FF
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        return Ok("image/jpeg");
    }

    // WebP: RIFF .... WEBP
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok("image/webp");
    }

    // GIF: GIF87a ou GIF89a
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return Ok("image/gif");
    }

    Err("Formato de imagem inválido ou não suportado. Apenas PNG, JPEG, WebP e GIF são permitidos.")
}

/// Retorna o caminho do diretório de imagens cifradas dentro do cofre.
fn get_images_dir(vault: &UnlockedVault) -> PathBuf {
    vault.root_path().join("images")
}

/// Retorna o caminho do arquivo de índice de imagens cifrado do cofre.
fn get_images_index_path(vault: &UnlockedVault) -> PathBuf {
    get_images_dir(vault).join("images_index.blob")
}

/// Carrega o índice de imagens de todas as entidades do cofre (mapa entity_id -> Vec<EntityImageMeta>).
pub fn load_images_index(
    vault: &UnlockedVault,
) -> Result<HashMap<String, Vec<EntityImageMeta>>, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let index_path = get_images_index_path(vault);
    if !index_path.exists() {
        return Ok(HashMap::new());
    }

    let blob_bytes = fs::read(&index_path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let dek = vault.dek()?;
    let aad = format!("MAGRATHEA-V1:images_index:{}", vault.header().vault_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    let index: HashMap<String, Vec<EntityImageMeta>> =
        serde_json::from_slice(&plaintext).map_err(|e| VaultError::Serialization(e.to_string()))?;

    Ok(index)
}

/// Grava o índice de imagens cifrado no cofre de forma estritamente atômica.
pub fn save_images_index(
    vault: &UnlockedVault,
    index: &HashMap<String, Vec<EntityImageMeta>>,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let images_dir = get_images_dir(vault);
    if !images_dir.exists() {
        fs::create_dir_all(&images_dir).map_err(|e| VaultError::Io(e.to_string()))?;
    }

    let json_bytes =
        serde_json::to_vec_pretty(index).map_err(|e| VaultError::Serialization(e.to_string()))?;

    let dek = vault.dek()?;
    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:images_index:{}", vault.header().vault_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &json_bytes)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_bytes());
    blob.extend_from_slice(&ciphertext);

    let index_path = get_images_index_path(vault);
    atomic_write(&index_path, &blob)?;

    Ok(())
}

/// Importa, valida e cifra uma nova imagem anexando-a a uma entidade no cofre.
pub fn import_entity_image(
    vault: &UnlockedVault,
    entity_id: &str,
    file_name: &str,
    raw_bytes: &[u8],
    created_at_iso: &str,
) -> Result<EntityImageMeta, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    // 1. Validação estrita de magic bytes
    let mime_type = validate_image_magic_bytes(raw_bytes)
        .map_err(|err_msg| VaultError::Serialization(err_msg.to_string()))?;

    let image_id = format!(
        "img_{:08x}{:08x}",
        rand::random::<u32>(),
        rand::random::<u32>()
    );
    let images_dir = get_images_dir(vault);
    if !images_dir.exists() {
        fs::create_dir_all(&images_dir).map_err(|e| VaultError::Io(e.to_string()))?;
    }

    // 2. Cifra o conteúdo da imagem com AAD vinculado ao image_id
    let dek = vault.dek()?;
    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:image:{image_id}");
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), raw_bytes)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_bytes());
    blob.extend_from_slice(&ciphertext);

    let image_path = images_dir.join(format!("{image_id}.blob"));
    atomic_write(&image_path, &blob)?;

    // 3. Atualiza o índice cifrado de imagens da entidade
    let meta = EntityImageMeta {
        id: image_id,
        entity_id: entity_id.to_string(),
        file_name: file_name.to_string(),
        mime_type: mime_type.to_string(),
        byte_size: raw_bytes.len(),
        created_at: created_at_iso.to_string(),
    };

    let mut index = load_images_index(vault)?;
    index
        .entry(entity_id.to_string())
        .or_default()
        .push(meta.clone());
    save_images_index(vault, &index)?;

    Ok(meta)
}

/// Carrega e decifra uma imagem sob demanda para visualização segura em memória.
pub fn load_entity_image(
    vault: &UnlockedVault,
    image_id: &str,
) -> Result<EntityImageData, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let images_dir = get_images_dir(vault);
    let image_path = images_dir.join(format!("{image_id}.blob"));

    if !image_path.exists() {
        return Err(VaultError::NotFound(format!(
            "Imagem {image_id} não encontrada no cofre."
        )));
    }

    let blob_bytes = fs::read(&image_path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let dek = vault.dek()?;
    let aad = format!("MAGRATHEA-V1:image:{image_id}");
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    // Localiza metadados no índice
    let index = load_images_index(vault)?;
    let meta = index
        .values()
        .flat_map(|list| list.iter())
        .find(|m| m.id == image_id)
        .cloned()
        .ok_or_else(|| {
            VaultError::NotFound(format!(
                "Metadados da imagem {image_id} não constam no índice."
            ))
        })?;

    // Codifica em Base64
    let base64_data = simple_base64_encode(&plaintext);

    Ok(EntityImageData { meta, base64_data })
}

/// Exclui uma imagem do cofre e remove-a do índice cifrado.
pub fn delete_entity_image(
    vault: &UnlockedVault,
    entity_id: &str,
    image_id: &str,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let image_path = get_images_dir(vault).join(format!("{image_id}.blob"));
    if image_path.exists() {
        let _ = fs::remove_file(&image_path);
    }

    let mut index = load_images_index(vault)?;
    if let Some(list) = index.get_mut(entity_id) {
        list.retain(|m| m.id != image_id);
    }
    save_images_index(vault, &index)?;

    Ok(())
}

/// Lista todas as imagens associadas a uma entidade.
pub fn list_entity_images(
    vault: &UnlockedVault,
    entity_id: &str,
) -> Result<Vec<EntityImageMeta>, VaultError> {
    let index = load_images_index(vault)?;
    Ok(index.get(entity_id).cloned().unwrap_or_default())
}

/// Rastreia e agrega as aparições de uma entidade em todas as cenas do manuscrito.
pub fn get_entity_appearances(index: &VaultIndex, entity_id: &str) -> Vec<EntityAppearance> {
    let backlinks = index.get_backlinks(entity_id);
    let mut appearances = Vec::new();

    for b in backlinks {
        if b.source_kind == "scene" {
            appearances.push(EntityAppearance {
                scene_id: b.source_id,
                scene_title: b.source_title,
                line_number: b.line_number,
                snippet: b.snippet,
                alias: b.alias,
            });
        }
    }

    appearances
}

/// Gera o conteúdo padrão em Markdown + Frontmatter com perguntas provocativas para cada tipo.
pub fn generate_entity_template(kind: &str, title: &str) -> String {
    let trimmed_title = title.trim();
    let safe_title = if trimmed_title.is_empty() {
        "Sem Título"
    } else {
        trimmed_title
    };

    match kind.to_lowercase().as_str() {
        "character" => format!(
            r#"---
title: "{safe_title}"
type: character
role: Protagonista
aliases: []
age: ""
archetype: ""
desire: "Desejo consciente (o que busca abertamente)"
need: "Necessidade inconsciente (o que realmente precisa aprender/curar)"
fear: "Maior medo ou trauma raiz"
contradiction: "Contradição interna marcante"
arc_beginning: "Como começa no início da narrativa"
arc_climax: "Ponto de virada e prova de fogo"
arc_ending: "Como termina após a transformação"
---

## Presença & Descrição Física
<!-- Como esta pessoa é percebida ao entrar em um aposento? Olhar, postura, roupas e marcas. -->

## Histórico & Segredos
<!-- O que moldou seu caráter? Quais segredos guarda até de si mesma? -->

## Voz & Maneirismos
<!-- Padrões de fala, sotaques, palavras recorrentes e pequenos gestos involuntários. -->

## Conexões & Dinâmicas
<!-- Aliados de confiança, rivalidades veladas e dívidas de honra. -->
"#
        ),

        "location" => format!(
            r#"---
title: "{safe_title}"
type: location
scale: Cidade
atmosphere: "Sombria e nevoenta"
danger_level: "Médio"
tags: []
---

## Geografia & Sensações
<!-- O que se vê ao horizonte? Quais são os cheiros, sons e a temperatura marcante do lugar? -->

## História & Significância
<!-- Quem construiu este lugar e que tragédia ou vitória histórica reverbera em suas pedras? -->

## Regras & Perigos
<!-- O que um viajante imprudente jamais deve fazer aqui? Leis locais e perigos naturais. -->
"#
        ),

        "item" => format!(
            r#"---
title: "{safe_title}"
type: item
origin: "Antiga civilização perdida"
rarity: "Único"
powers: "Canalizador arcano"
tags: []
---

## Propriedades & Aparência
<!-- Peso, matéria-prima, inscrições gravadas e o toque ao empunhá-lo. -->

## Mito & Origem
<!-- Quem o forjou e a que custo? Que juramento foi selado junto a ele? -->

## Portadores & Paradeiro
<!-- Por quais mãos já passou e onde repousa no presente? -->
"#
        ),

        "faction" => format!(
            r#"---
title: "{safe_title}"
type: faction
motto: "A luz guia o dever"
leader: "Conselho dos Sete"
influence: "Alta"
tags: []
---

## Filosofia & Doutrina
<!-- Qual é a verdade inegociável que une seus membros? Qual o objetivo supremo? -->

## Estrutura & Hierarquia
<!-- Como se sobe em suas fileiras? Ritos de passagem e punições para traidores. -->

## Alianças & Inimizades
<!-- Quem os apoia nas sombras e quem jurou sua derrocada? -->
"#
        ),

        _ => format!(
            r#"---
title: "{safe_title}"
type: lore
topic: "História e Mundo"
tags: []
---

## Conceito & Contexto
<!-- Explicação fundamental deste elemento no universo literário. -->

## Desdobramentos Narrativos
<!-- Como este elemento afeta a vida dos personagens e o rumo do enredo? -->
"#
        ),
    }
}

/// Codificador simples em Base64 para retornar a imagem sem bibliotecas externas pesadas.
fn simple_base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(CHARSET[((n >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            out.push(CHARSET[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(CHARSET[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

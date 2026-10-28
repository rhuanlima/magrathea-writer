//! Testes de integração para Worldbuilding, Fichas Guiadas, Enciclopédia e Galeria de Imagens (M4-01 a M4-05).

use magrathea_writer_lib::commands::{
    create_vault_internal, create_world_entity_internal, delete_entity_image_internal,
    get_entity_appearances_internal, import_entity_image_internal, list_entity_images_internal,
    load_entity_image_internal, load_scene_content_internal, save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use magrathea_writer_lib::worldbuilding::validate_image_magic_bytes;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_worldbuilding_lifecycle_appearances_and_encrypted_gallery() {
    let temp_dir =
        std::env::temp_dir().join(format!("magrathea_wb_test_{}", rand::random::<u64>()));
    fs::create_dir_all(&temp_dir).unwrap();

    let state = AppState::default();
    let initial_pass = "senha_mestra_ultra_segura_1234!";

    // 1. Cria cofre de teste
    let create_res = create_vault_internal(
        &state,
        &temp_dir.to_string_lossy(),
        "O Legado das Marés",
        initial_pass,
    )
    .expect("deve criar cofre");

    // 2. Criação de Personagem com template guiado (M4-01, M4-02)
    let character = create_world_entity_internal(
        &state,
        "character",
        "Jaci",
        vec!["protagonista".to_string(), "navegadora".to_string()],
    )
    .expect("deve criar personagem Jaci");

    assert_eq!(character.title, "Jaci");
    assert_eq!(character.kind, "character");

    let char_content = load_scene_content_internal(&state, &character.id)
        .expect("deve carregar conteudo gerado pelo template");
    assert!(char_content.contains("type: character"));
    assert!(char_content.contains("desire:"));
    assert!(char_content.contains("fear:"));
    assert!(char_content.contains("arc_beginning:"));

    // 3. Criação de Local com template (M4-01)
    let location = create_world_entity_internal(
        &state,
        "location",
        "Praia das Conchas",
        vec!["costa".to_string()],
    )
    .expect("deve criar local");

    let loc_content =
        load_scene_content_internal(&state, &location.id).expect("deve carregar conteudo do local");
    assert!(loc_content.contains("type: location"));
    assert!(loc_content.contains("atmosphere:"));

    // 4. Criação de Cena no manuscrito citando as entidades (M4-05)
    let scene_text = r#"---
title: Cena 1
type: scene
---
O dia amanheceu cinzento quando [[Jaci]] caminhou pensativa até [[Praia das Conchas|a praia sagrada]].
Sentia o vento salgado no rosto."#;

    save_scene_content_internal(&state, "cena_mar_1", scene_text)
        .expect("deve salvar cena com links");

    // 5. Rastreio de Aparições (M4-05)
    let jaci_appearances = get_entity_appearances_internal(&state, &character.id)
        .expect("deve rastrear aparições de Jaci");
    assert_eq!(jaci_appearances.len(), 1);
    assert_eq!(jaci_appearances[0].scene_id, "cena_mar_1");
    assert!(jaci_appearances[0].snippet.contains("[[Jaci]]"));

    let loc_appearances = get_entity_appearances_internal(&state, &location.id)
        .expect("deve rastrear aparições do local");
    assert_eq!(loc_appearances.len(), 1);
    assert_eq!(
        loc_appearances[0].alias,
        Some("a praia sagrada".to_string())
    );

    // 6. Validação de Magic Bytes de Imagens (M4-04)
    // 6.1. PNG legítimo
    let valid_png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];
    assert_eq!(
        validate_image_magic_bytes(&valid_png_bytes),
        Ok("image/png")
    );

    // 6.2. JPEG legítimo
    let valid_jpg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    assert_eq!(
        validate_image_magic_bytes(&valid_jpg_bytes),
        Ok("image/jpeg")
    );

    // 6.3. Executável malicioso disfarçado (ELF header ou MZ) deve falhar
    let fake_executable_bytes = vec![0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01]; // ELF
    assert!(validate_image_magic_bytes(&fake_executable_bytes).is_err());

    // 7. Importação e Cifragem de Imagem no Cofre (M4-04)
    let img_meta = import_entity_image_internal(
        &state,
        &character.id,
        "avatar_jaci.png",
        &valid_png_bytes,
        "2026-10-27T10:00:00Z",
    )
    .expect("deve importar imagem com sucesso");

    assert_eq!(img_meta.file_name, "avatar_jaci.png");
    assert_eq!(img_meta.mime_type, "image/png");

    // 8. Confirmação de segurança: o blob em disco está cifrado e opaco
    let vault_root = PathBuf::from(&create_res.vault_path);
    let img_blob_path = vault_root
        .join("images")
        .join(format!("{}.blob", img_meta.id));
    assert!(img_blob_path.exists());
    let raw_blob_on_disk = fs::read(&img_blob_path).unwrap();
    // Não pode conter os magic bytes crus em claro no início do arquivo
    assert_ne!(
        &raw_blob_on_disk[..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );

    // 9. Leitura e Decifragem de Imagem sob demanda
    let loaded_img =
        load_entity_image_internal(&state, &img_meta.id).expect("deve carregar e decifrar imagem");
    assert_eq!(loaded_img.meta.id, img_meta.id);
    assert!(!loaded_img.base64_data.is_empty());

    // 10. Listagem de Imagens da Entidade
    let images_list = list_entity_images_internal(&state, &character.id)
        .expect("deve listar imagens da entidade");
    assert_eq!(images_list.len(), 1);
    assert_eq!(images_list[0].id, img_meta.id);

    // 11. Exclusão de Imagem
    delete_entity_image_internal(&state, &character.id, &img_meta.id).expect("deve excluir imagem");
    assert!(!img_blob_path.exists());

    let after_delete =
        list_entity_images_internal(&state, &character.id).expect("deve listar apos exclusao");
    assert!(after_delete.is_empty());

    // Limpeza
    let _ = fs::remove_dir_all(&temp_dir);
}

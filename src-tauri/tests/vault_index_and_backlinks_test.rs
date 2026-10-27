//! Testes de integração para o Índice em Memória e Backlinks (M2-01).

use magrathea_writer_lib::commands::{
    close_vault_internal, create_manuscript_item_internal, create_vault_internal,
    get_document_backlinks_internal, get_entity_preview_internal, get_vault_index_internal,
    lock_vault_internal, open_vault_with_passphrase_internal, rename_manuscript_item_internal,
    save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

fn create_test_temp_dir(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "magrathea_index_test_{}_{}",
        test_name,
        rand::random::<u64>()
    ));
    fs::create_dir_all(&dir).expect("falha ao criar diretório temporário");
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_in_memory_index_and_backlinks_integration() {
    let state = AppState::default();
    let temp_root = create_test_temp_dir("index_backlinks");
    let passphrase = "SenhaDoAutorSegura!2026";

    // 1. Cria cofre
    let create_res = create_vault_internal(
        &state,
        temp_root.to_str().unwrap(),
        "O Mistério de Magrathea",
        passphrase,
    )
    .expect("criar cofre");

    // 2. Consulta o índice inicial (vazio antes de itens serem criados)
    let initial_overview =
        get_vault_index_internal(&state).expect("deve retornar overview inicial");
    assert_eq!(initial_overview.entities.len(), 0);
    assert_eq!(initial_overview.total_links_count, 0);

    // 3. Cria uma entidade "Jaci" (personagem) no manifesto
    let jaci_item = create_manuscript_item_internal(&state, "scene", None, "Jaci")
        .expect("deve criar item Jaci");

    // Salva a ficha de Jaci com frontmatter
    let jaci_content = "---\ntitle: Jaci\ntype: character\nsummary: Guardiã mística das correntes profundas.\n---\nJaci nasceu nas águas escuras do arquipélago.";
    save_scene_content_internal(&state, &jaci_item.id, jaci_content).expect("salvar Jaci");

    // 4. Cria e salva a Cena 1 citando Jaci
    let scene_item = create_manuscript_item_internal(&state, "scene", None, "Encontro no Cais")
        .expect("deve criar Cena 1");

    let scene_content = "Na noite fria, o marinheiro encontrou [[Jaci]] perto das docas.\nEles sussurraram sobre [[Atlântida]].";
    save_scene_content_internal(&state, &scene_item.id, scene_content).expect("salvar Cena 1");

    // 5. Verifica o índice global
    let overview = get_vault_index_internal(&state).expect("overview atualizado");
    assert_eq!(overview.total_links_count, 1); // link para Jaci resolvido
    assert_eq!(overview.unresolved_links_count, 1); // Atlântida ainda não existe
    assert!(overview
        .unresolved_targets
        .contains(&"atlântida".to_string()));

    // 6. Verifica backlinks de Jaci
    let jaci_backlinks =
        get_document_backlinks_internal(&state, &jaci_item.id).expect("backlinks de Jaci");
    assert_eq!(jaci_backlinks.len(), 1);
    assert_eq!(jaci_backlinks[0].source_title, "Encontro no Cais");
    assert_eq!(jaci_backlinks[0].target_query, "Jaci");
    assert!(jaci_backlinks[0].snippet.contains("marinheiro encontrou"));

    // 7. Verifica pré-visualização (hover preview)
    let preview = get_entity_preview_internal(&state, "jaci")
        .expect("preview de Jaci")
        .expect("deve encontrar entidade");
    assert_eq!(preview.title, "Jaci");
    assert_eq!(
        preview.summary.as_deref(),
        Some("Guardiã mística das correntes profundas.")
    );
    assert_eq!(preview.mention_count, 1);

    // 8. Renomeia Cena para "Encontro nas Docas" e valida sincronismo
    rename_manuscript_item_internal(&state, &scene_item.id, "Encontro nas Docas")
        .expect("renomear cena");
    let updated_preview = get_entity_preview_internal(&state, "encontro nas docas")
        .expect("busca pelo novo nome")
        .expect("deve existir");
    assert_eq!(updated_preview.title, "Encontro nas Docas");

    // 9. Bloqueio e verificação de zeroização do índice
    lock_vault_internal(&state).expect("bloquear cofre");
    let err_after_lock = get_vault_index_internal(&state);
    assert!(err_after_lock.is_err());

    // 10. Desbloqueio e reconstrução transparente do índice
    open_vault_with_passphrase_internal(&state, &create_res.vault_path, passphrase)
        .expect("desbloquear cofre");
    let restored_overview =
        get_vault_index_internal(&state).expect("índice reconstruído ao desbloquear");
    assert_eq!(restored_overview.total_links_count, 1);

    close_vault_internal(&state).expect("fechar");
    cleanup_test_dir(&temp_root);
}

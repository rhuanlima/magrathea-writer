//! Testes de integração para Renomeação com Refatoração Global e Rollback (M2-05).

use magrathea_writer_lib::commands::{
    apply_rename_refactor_internal, close_vault_internal, create_manuscript_item_internal,
    create_vault_internal, get_entity_preview_internal, load_scene_content_internal,
    preview_rename_entity_internal, rollback_rename_refactor_internal, save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

fn create_test_temp_dir(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "magrathea_refactor_test_{}_{}",
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
fn test_rename_refactor_preview_apply_and_rollback_lifecycle() {
    let state = AppState::default();
    let temp_root = create_test_temp_dir("refactor_lifecycle");
    let passphrase = "SenhaDoAutorSegura!2026";

    // 1. Cria cofre
    create_vault_internal(
        &state,
        temp_root.to_str().unwrap(),
        "As Crônicas de Magrathea",
        passphrase,
    )
    .expect("criar cofre");

    // 2. Cria a entidade "Jaci" (personagem)
    let jaci = create_manuscript_item_internal(&state, "scene", None, "Jaci")
        .expect("criar personagem Jaci");
    save_scene_content_internal(
        &state,
        &jaci.id,
        "---\ntitle: Jaci\ntype: character\n---\nBiografia de Jaci.",
    )
    .expect("salvar ficha Jaci");

    // 3. Cria Cena 1 citando Jaci de forma simples e com alias
    let scene1 =
        create_manuscript_item_internal(&state, "scene", None, "Cena 1").expect("criar Cena 1");
    save_scene_content_internal(
        &state,
        &scene1.id,
        "No cais, [[Jaci]] observou o nevoeiro.\nMais tarde, conversou com [[Jaci|A Guardiã]].",
    )
    .expect("salvar Cena 1");

    // 4. Cria Cena 2 também citando Jaci
    let scene2 =
        create_manuscript_item_internal(&state, "scene", None, "Cena 2").expect("criar Cena 2");
    save_scene_content_internal(
        &state,
        &scene2.id,
        "Todos no vilarejo conheciam a lenda de [[Jaci]].",
    )
    .expect("salvar Cena 2");

    // 5. Gera a pré-visualização da refatoração de "Jaci" para "Iara"
    let preview = preview_rename_entity_internal(&state, "Jaci", "Iara")
        .expect("gerar prévia de refatoração");

    assert_eq!(preview.old_name, "Jaci");
    assert_eq!(preview.new_name, "Iara");
    assert_eq!(preview.total_occurrences, 3); // 2 na Cena 1 + 1 na Cena 2
    assert_eq!(preview.affected_documents.len(), 2);

    let doc1_preview = preview
        .affected_documents
        .iter()
        .find(|d| d.item_id == scene1.id)
        .expect("cena 1 na prévia");
    assert_eq!(doc1_preview.occurrences.len(), 2);
    assert_eq!(doc1_preview.occurrences[0].raw_match, "[[Jaci]]");
    assert!(doc1_preview.occurrences[0].new_snippet.contains("[[Iara]]"));
    assert_eq!(doc1_preview.occurrences[1].raw_match, "[[Jaci|A Guardiã]]");
    assert!(doc1_preview.occurrences[1]
        .new_snippet
        .contains("[[Iara|A Guardiã]]"));

    // 6. Aplica a refatoração atômica em lote com snapshot
    let apply_res =
        apply_rename_refactor_internal(&state, "Jaci", "Iara").expect("aplicar refatoração");

    assert_eq!(apply_res.total_replacements, 3);
    assert_eq!(apply_res.modified_docs_count, 3); // Jaci (título/ficha) + Cena 1 + Cena 2
    assert!(!apply_res.snapshot_id.is_empty());

    // 7. Verifica os conteúdos gravados nos blobs (devem conter Iara)
    let content1 = load_scene_content_internal(&state, &scene1.id).expect("ler cena 1");
    assert!(content1.contains("[[Iara]]"));
    assert!(content1.contains("[[Iara|A Guardiã]]"));
    assert!(!content1.contains("[[Jaci]]"));

    let content2 = load_scene_content_internal(&state, &scene2.id).expect("ler cena 2");
    assert!(content2.contains("[[Iara]]"));
    assert!(!content2.contains("[[Jaci]]"));

    // Verifica que o índice agora reconhece Iara com 3 menções
    let iara_preview = get_entity_preview_internal(&state, "iara")
        .expect("preview de Iara")
        .expect("Iara deve existir");
    assert_eq!(iara_preview.title, "Iara");
    assert_eq!(iara_preview.mention_count, 3);

    // 8. Executa Rollback utilizando o snapshot_id
    rollback_rename_refactor_internal(&state, &apply_res.snapshot_id)
        .expect("rollback deve restaurar");

    // 9. Valida que o cofre foi 100% restaurado ao estado original
    let restored_content1 = load_scene_content_internal(&state, &scene1.id).expect("ler cena 1");
    assert!(restored_content1.contains("[[Jaci]]"));
    assert!(restored_content1.contains("[[Jaci|A Guardiã]]"));
    assert!(!restored_content1.contains("[[Iara]]"));

    let restored_jaci = get_entity_preview_internal(&state, "jaci")
        .expect("preview de Jaci restaurada")
        .expect("Jaci deve existir");
    assert_eq!(restored_jaci.title, "Jaci");
    assert_eq!(restored_jaci.mention_count, 3);

    close_vault_internal(&state).expect("fechar");
    cleanup_test_dir(&temp_root);
}

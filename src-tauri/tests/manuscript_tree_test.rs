use magrathea_writer_lib::commands::{
    create_manuscript_item_internal, create_vault_internal, empty_trash_internal,
    get_manuscript_tree_internal, move_item_to_trash_internal, rename_manuscript_item_internal,
    reorder_manuscript_items_internal, restore_item_from_trash_internal,
    update_scene_metadata_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

fn create_test_temp_dir(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let random_id: u64 = rand::random();
    dir.push(format!("magrathea_tree_test_{test_name}_{random_id}"));
    fs::create_dir_all(&dir).expect("falha ao criar diretório temporário de teste");
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    if dir.exists() {
        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn test_manuscript_tree_crud_reorder_and_encrypted_trash() {
    let base_dir = create_test_temp_dir("crud_reorder_trash");
    let title = "O Guia do Mochileiro".to_string();
    let passphrase = "FraseSuperSecreta!2026".to_string();

    let state = AppState::default();

    // 1. Cria cofre inicial
    let init_res = create_vault_internal(&state, base_dir.to_str().unwrap(), &title, &passphrase)
        .expect("vault creation failed");
    assert_eq!(init_res.project_title, title);

    // 2. Consulta árvore inicial (deve vir vazia)
    let initial_tree = get_manuscript_tree_internal(&state).expect("failed to get tree");
    assert!(initial_tree.items.is_empty());
    assert!(initial_tree.trash.is_empty());

    // 3. Cria nós na hierarquia: Parte 1 -> Capítulo 1 -> Cena 1 e Cena 2
    let part = create_manuscript_item_internal(&state, "part", None, "Parte 1: A Terra")
        .expect("create part failed");
    assert_eq!(part.kind, "part");
    assert_eq!(part.parent_id, None);
    assert_eq!(part.title, "Parte 1: A Terra");
    assert_eq!(part.order, 0);

    let chapter = create_manuscript_item_internal(
        &state,
        "chapter",
        Some(&part.id),
        "Capítulo 1: Quinta-Feira",
    )
    .expect("create chapter failed");
    assert_eq!(chapter.kind, "chapter");
    assert_eq!(chapter.parent_id, Some(part.id.clone()));
    assert_eq!(chapter.order, 0);

    let scene1 = create_manuscript_item_internal(
        &state,
        "scene",
        Some(&chapter.id),
        "Cena 1: A Casa de Arthur",
    )
    .expect("create scene1 failed");
    assert_eq!(scene1.kind, "scene");
    assert_eq!(scene1.parent_id, Some(chapter.id.clone()));
    assert_eq!(scene1.order, 0);
    assert_eq!(scene1.status, Some("draft".to_string()));
    assert!(scene1.blob_id.is_some());

    let scene2 =
        create_manuscript_item_internal(&state, "scene", Some(&chapter.id), "Cena 2: O Pub")
            .expect("create scene2 failed");
    assert_eq!(scene2.order, 1);

    // 4. Renomeia Cena 1
    rename_manuscript_item_internal(&state, &scene1.id, "Cena 1: Os Buldôzeres")
        .expect("rename scene failed");

    // 5. Atualiza metadados da Cena 1 (status e tags)
    update_scene_metadata_internal(
        &state,
        &scene1.id,
        Some("revised".to_string()),
        vec!["arthur".to_string(), "proser".to_string()],
    )
    .expect("update metadata failed");

    // 6. Reordena as cenas sob o Capítulo 1: Cena 2 antes de Cena 1
    reorder_manuscript_items_internal(
        &state,
        Some(&chapter.id),
        &[scene2.id.clone(), scene1.id.clone()],
    )
    .expect("reorder failed");

    let tree_after_reorder = get_manuscript_tree_internal(&state).expect("get tree failed");
    let reordered_s2 = tree_after_reorder
        .items
        .iter()
        .find(|i| i.id == scene2.id)
        .unwrap();
    let reordered_s1 = tree_after_reorder
        .items
        .iter()
        .find(|i| i.id == scene1.id)
        .unwrap();
    assert_eq!(reordered_s2.order, 0);
    assert_eq!(reordered_s1.order, 1);
    assert_eq!(reordered_s1.title, "Cena 1: Os Buldôzeres");
    assert_eq!(reordered_s1.status, Some("revised".to_string()));
    assert_eq!(
        reordered_s1.tags,
        vec!["arthur".to_string(), "proser".to_string()]
    );

    // 7. Mover Cena 2 para a lixeira interna
    move_item_to_trash_internal(&state, &scene2.id).expect("move to trash failed");

    let tree_with_trash = get_manuscript_tree_internal(&state).expect("get tree failed");
    assert!(!tree_with_trash.items.iter().any(|i| i.id == scene2.id));
    assert!(tree_with_trash.trash.iter().any(|i| i.id == scene2.id));

    // 8. Restaurar Cena 2 da lixeira
    restore_item_from_trash_internal(&state, &scene2.id).expect("restore failed");

    let tree_restored = get_manuscript_tree_internal(&state).expect("get tree failed");
    assert!(tree_restored.items.iter().any(|i| i.id == scene2.id));
    assert!(tree_restored.trash.is_empty());

    // 9. Mover Cena 2 para a lixeira novamente e esvaziar lixeira definitivamente
    let s2_blob_id = scene2.blob_id.clone().unwrap();
    move_item_to_trash_internal(&state, &scene2.id).expect("move to trash failed");
    empty_trash_internal(&state).expect("empty trash failed");

    let tree_after_empty = get_manuscript_tree_internal(&state).expect("get tree failed");
    assert!(tree_after_empty.trash.is_empty());

    // Verifica que o blob foi de fato apagado do disco
    let guard = state.active_vault.lock().unwrap();
    let vault = guard.as_ref().unwrap();
    let blob_path = vault
        .root_path()
        .join("blobs")
        .join(format!("{s2_blob_id}.blob"));
    assert!(
        !blob_path.exists(),
        "Blob da cena esvaziada não deveria mais existir no disco"
    );

    drop(guard);
    cleanup_test_dir(&base_dir);
}

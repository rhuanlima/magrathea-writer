//! Teste de integração para Milestone M5 (Versões, Busca e Backup).
//!
//! Cobre:
//! - M5-01: Criação de snapshots manuais e automáticos cifrados por cena.
//! - M5-02: Lista de snapshots, diff e restauração com snapshot de segurança.
//! - M5-03: Busca global rápida em memória com snippets e contagem de ocorrências.
//! - M5-04: Substituição global atômica com prévia e rollback completo.
//! - M5-05: Backup automático cifrado rotativo com detecção de mesmo disco.

use magrathea_writer_lib::commands::{
    apply_global_replace_internal, compare_snapshot_with_current_internal,
    create_manuscript_item_internal, create_scene_snapshot_internal, create_vault_backup_internal,
    create_vault_internal, get_scene_snapshot_content_internal, global_search_internal,
    list_scene_snapshots_internal, list_vault_backups_internal, preview_global_replace_internal,
    restore_scene_snapshot_internal, rollback_global_replace_internal, save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_snapshots_search_and_backup_lifecycle() {
    let temp_dir =
        std::env::temp_dir().join(format!("magrathea_m5_test_{:08x}", rand::random::<u32>()));
    fs::create_dir_all(&temp_dir).expect("deve criar temp dir");

    let state = AppState::default();
    let title = "O Guia das Galáxias";
    let passphrase = "senha-soberana-m5";

    // 1. Criação do cofre
    let init_res = create_vault_internal(&state, temp_dir.to_str().unwrap(), title, passphrase)
        .expect("deve criar cofre");
    let vault_path = PathBuf::from(&init_res.vault_path);

    // 2. Criação de duas cenas no manuscrito
    let scene1 = create_manuscript_item_internal(&state, "scene", None, "Capítulo 1: O Início")
        .expect("deve criar cena 1");

    let scene2 = create_manuscript_item_internal(&state, "scene", None, "Capítulo 2: O Meio")
        .expect("deve criar cena 2");

    // 3. Salva conteúdo inicial nas cenas
    let content_scene1_v1 = "Arthur Dent acordou cedo e percebeu que um trator amarelo estava em frente à sua casa.\nEle bebeu chá de camomila.";
    let content_scene2 =
        "Ford Prefect olhou para Arthur e disse: Não entre em pânico.\nEle pegou uma toalha limpa.";

    save_scene_content_internal(&state, &scene1.id, content_scene1_v1).expect("deve salvar cena 1");
    save_scene_content_internal(&state, &scene2.id, content_scene2).expect("deve salvar cena 2");

    // ========================================================================
    // TESTE M5-01 e M5-02: Snapshots, Diff e Restauração
    // ========================================================================

    // Cria snapshot manual da versão 1 da cena 1
    let snap_v1 = create_scene_snapshot_internal(
        &state,
        &scene1.id,
        "Primeiro rascunho com chá",
        "manual",
        Some(content_scene1_v1),
    )
    .expect("deve criar snapshot v1");

    assert_eq!(snap_v1.scene_id, scene1.id);
    assert_eq!(snap_v1.label, "Primeiro rascunho com chá");
    assert_eq!(snap_v1.trigger, "manual");

    // Atualiza cena 1 para versão 2
    let content_scene1_v2 = "Arthur Dent acordou cedo e viu um trator gigante na estrada.\nEle bebeu uma xícara de café quente.";
    save_scene_content_internal(&state, &scene1.id, content_scene1_v2)
        .expect("deve salvar cena 1 v2");

    // Lista snapshots
    let snaps_list =
        list_scene_snapshots_internal(&state, &scene1.id).expect("deve listar snapshots");
    assert_eq!(snaps_list.len(), 1);
    assert_eq!(snaps_list[0].id, snap_v1.id);

    // Carrega conteúdo decifrado do snapshot v1
    let loaded_snap_v1 = get_scene_snapshot_content_internal(&state, &scene1.id, &snap_v1.id)
        .expect("deve ler snapshot v1 decifrado");
    assert_eq!(loaded_snap_v1, content_scene1_v1);

    // Compara (diff) snapshot v1 com conteúdo atual (v2)
    let diff =
        compare_snapshot_with_current_internal(&state, &scene1.id, &snap_v1.id, content_scene1_v2)
            .expect("deve calcular diff com sucesso");

    assert_eq!(diff.snapshot_id, snap_v1.id);
    assert!(!diff.lines.is_empty());
    assert!(diff
        .lines
        .iter()
        .any(|l| l.kind == "added" || l.kind == "removed"));

    // Restaura snapshot v1 (deve gerar snapshot de segurança automático da v2)
    let restore_res =
        restore_scene_snapshot_internal(&state, &scene1.id, &snap_v1.id, content_scene1_v2)
            .expect("deve restaurar snapshot v1");

    assert_eq!(restore_res.restored_content, content_scene1_v1);
    assert_eq!(restore_res.safety_snapshot.trigger, "auto");

    // Lista snapshots após restauração: agora devem existir 2 snapshots (o original + o de segurança)
    let updated_snaps =
        list_scene_snapshots_internal(&state, &scene1.id).expect("deve listar 2 snapshots");
    assert_eq!(updated_snaps.len(), 2);

    // ========================================================================
    // TESTE M5-03: Busca Global em Memória
    // ========================================================================

    // Busca pela palavra "Arthur" (presente na cena 1 restaurada e na cena 2)
    let search_res =
        global_search_internal(&state, "Arthur", false).expect("deve executar busca global");
    assert_eq!(search_res.total_matches, 2);
    assert_eq!(search_res.documents.len(), 2);

    // Busca com case-sensitive por "pânico"
    let search_case =
        global_search_internal(&state, "pânico", true).expect("deve buscar case-sensitive");
    assert_eq!(search_case.total_matches, 1);
    assert_eq!(search_case.documents[0].item_id, scene2.id);

    // ========================================================================
    // TESTE M5-04: Substituição Global com Prévia e Rollback
    // ========================================================================

    // Prévia de substituição: "Arthur" -> "Dent"
    let preview = preview_global_replace_internal(&state, "Arthur", "Ford", false)
        .expect("deve gerar previa de substituicao");
    assert_eq!(preview.total_affected_docs, 2);
    assert_eq!(preview.total_occurrences, 2);

    // Aplica a substituição global
    let apply_res = apply_global_replace_internal(&state, "Arthur", "Ford", false)
        .expect("deve aplicar substituicao global");
    assert_eq!(apply_res.modified_docs_count, 2);
    assert_eq!(apply_res.total_occurrences_replaced, 2);
    assert!(!apply_res.rollback_snapshot_id.is_empty());

    // Nova busca por "Arthur" deve retornar 0
    let search_after =
        global_search_internal(&state, "Arthur", false).expect("deve buscar apos replace");
    assert_eq!(search_after.total_matches, 0);

    // Reverte via Rollback
    rollback_global_replace_internal(&state, &apply_res.rollback_snapshot_id)
        .expect("deve executar rollback");

    // Nova busca por "Arthur" deve retornar 2 novamente
    let search_rollback =
        global_search_internal(&state, "Arthur", false).expect("deve buscar apos rollback");
    assert_eq!(search_rollback.total_matches, 2);

    // ========================================================================
    // TESTE M5-05: Backup Cifrado Rotativo com Detecção de Mesmo Disco
    // ========================================================================

    let backup_dir = temp_dir.join("backups_destination");
    fs::create_dir_all(&backup_dir).expect("deve criar pasta de backups");

    // Cria 3 backups com limite de rotação de 2
    let b1 = create_vault_backup_internal(&state, backup_dir.to_str().unwrap(), 2)
        .expect("deve criar backup 1");
    let b2 = create_vault_backup_internal(&state, backup_dir.to_str().unwrap(), 2)
        .expect("deve criar backup 2");
    let b3 = create_vault_backup_internal(&state, backup_dir.to_str().unwrap(), 2)
        .expect("deve criar backup 3 com rotacao");

    assert!(b1.byte_size > 0);
    assert!(b2.byte_size > 0);
    assert!(b3.byte_size > 0);

    // Lista backups na pasta de destino: deve manter apenas 2 devido à rotação
    let backups_list = list_vault_backups_internal(&state, backup_dir.to_str().unwrap())
        .expect("deve listar backups");
    assert_eq!(backups_list.len(), 2);
    assert!(backups_list[0].file_name.ends_with(".magrathea_backup"));

    // Opacidade estrita do backup: verifica que nenhum texto plano vazou no arquivo copiado
    let backup_content = fs::read(
        backup_dir
            .join(&backups_list[0].file_name)
            .join("manifest.blob"),
    )
    .expect("deve ler manifesto no backup");
    let manifest_str = String::from_utf8_lossy(&backup_content);
    assert!(!manifest_str.contains("O Guia das Galáxias"));
    assert!(!manifest_str.contains("Arthur Dent"));

    // Limpeza
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&vault_path);
}

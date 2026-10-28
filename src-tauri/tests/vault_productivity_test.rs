use magrathea_writer_lib::commands::{
    change_vault_passphrase_internal, create_vault_internal, get_productivity_overview_internal,
    lock_vault_internal, open_vault_with_passphrase_internal, record_productivity_session_internal,
    save_scene_content_internal, update_productivity_targets_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_vault_productivity_lifecycle_and_encryption() {
    let temp_dir =
        std::env::temp_dir().join(format!("magrathea_prod_test_{}", rand::random::<u64>()));
    fs::create_dir_all(&temp_dir).unwrap();

    let state = AppState::default();
    let initial_pass = "senha_mestra_ultra_segura_1234!";

    // 1. Cria cofre de teste
    let create_res = create_vault_internal(
        &state,
        &temp_dir.to_string_lossy(),
        "Romance Produtivo",
        initial_pass,
    )
    .expect("deve criar cofre com sucesso");

    let today_date = "2026-10-27";

    // 2. Overview inicial quando productivity.blob ainda não existe
    let initial_overview = get_productivity_overview_internal(&state, today_date)
        .expect("deve retornar overview inicial padrao");
    assert_eq!(initial_overview.project_target_words, 50_000);
    assert_eq!(initial_overview.daily_target_words, 1_000);
    assert_eq!(initial_overview.today_words, 0);
    assert_eq!(initial_overview.today_seconds, 0);

    // 3. Atualização das metas
    let updated_overview = update_productivity_targets_internal(
        &state,
        75_000,
        Some("2026-12-31".to_string()),
        1_200,
        today_date,
    )
    .expect("deve atualizar metas");

    assert_eq!(updated_overview.project_target_words, 75_000);
    assert_eq!(
        updated_overview.project_deadline,
        Some("2026-12-31".to_string())
    );
    assert_eq!(updated_overview.daily_target_words, 1_200);

    // 4. Registro incremental de sessões de escrita
    let session1 = record_productivity_session_internal(&state, today_date, 450, 900)
        .expect("deve registrar primeira sessao");
    assert_eq!(session1.today_words, 450);
    assert_eq!(session1.today_seconds, 900);

    let session2 = record_productivity_session_internal(&state, today_date, 350, 600)
        .expect("deve registrar segunda sessao com acumulo");
    assert_eq!(session2.today_words, 800);
    assert_eq!(session2.today_seconds, 1500);

    // 5. Salva conteúdo de cena e verifica contagem total de palavras no manuscrito
    let scene_content = r#"---
title: Cena 1
type: scene
status: draft
---
O vento soprava frio sobre os campos verdejantes de Magrathea."#;

    save_scene_content_internal(&state, "cena_prod_1", scene_content).expect("deve salvar cena");

    let overview_with_manuscript = get_productivity_overview_internal(&state, today_date)
        .expect("deve retornar overview com palavras do manuscrito");
    assert_eq!(overview_with_manuscript.total_manuscript_words, 10);

    // 6. Verifica segurança no disco: productivity.blob deve estar cifrado
    let vault_root = PathBuf::from(&create_res.vault_path);
    let prod_blob_path = vault_root.join("productivity.blob");
    assert!(prod_blob_path.exists());
    let prod_blob_bytes = fs::read(&prod_blob_path).unwrap();
    let prod_blob_str = String::from_utf8_lossy(&prod_blob_bytes);
    assert!(
        !prod_blob_str.contains("75_000"),
        "dados de metas não podem estar em texto claro"
    );
    assert!(
        !prod_blob_str.contains("2026-12-31"),
        "prazo não pode estar em texto claro"
    );

    // 7. Bloqueio do cofre e verificação de proteção
    lock_vault_internal(&state).expect("deve bloquear cofre");
    let locked_attempt = get_productivity_overview_internal(&state, today_date);
    assert!(locked_attempt.is_err(), "deve falhar com cofre bloqueado");

    // 8. Desbloqueio e verificação de recuperação dos dados
    open_vault_with_passphrase_internal(&state, &create_res.vault_path, initial_pass)
        .expect("deve desbloquear com senha");
    let after_unlock = get_productivity_overview_internal(&state, today_date)
        .expect("deve ler dados apos desbloquear");
    assert_eq!(after_unlock.project_target_words, 75_000);
    assert_eq!(after_unlock.today_words, 800);

    // 9. Troca de frase-senha / recifragem integral do cofre
    let new_pass = "nova_senha_ultra_forte_com_simbolos_!@#";
    change_vault_passphrase_internal(&state, initial_pass, new_pass)
        .expect("deve trocar frase-senha e recifrar cofre com sucesso");

    // Verifica leitura após re-envelopamento
    let after_rekey = get_productivity_overview_internal(&state, today_date)
        .expect("deve ler dados com sucesso apos recifrar cofre");
    assert_eq!(after_rekey.project_target_words, 75_000);
    assert_eq!(after_rekey.today_words, 800);

    // Limpeza
    let _ = fs::remove_dir_all(&temp_dir);
}

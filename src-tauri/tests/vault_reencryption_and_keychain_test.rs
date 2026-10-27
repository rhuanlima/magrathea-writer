use magrathea_writer_lib::commands::{
    change_vault_passphrase_internal, close_vault_internal, create_manuscript_item_internal,
    create_vault_internal, get_vault_keyring_status_internal, load_scene_content_internal,
    open_vault_with_keyring_internal, open_vault_with_passphrase_internal,
    open_vault_with_recovery_key_internal, reencrypt_vault_internal, remove_from_keyring_internal,
    save_scene_content_internal, save_to_keyring_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::PathBuf;

fn create_test_temp_dir(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let random_id: u64 = rand::random();
    dir.push(format!("magrathea_reenc_test_{test_name}_{random_id}"));
    fs::create_dir_all(&dir).expect("falha ao criar diretório temporário de teste");
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    if dir.exists() {
        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn test_vault_passphrase_change_lifecycle() {
    let base_dir = create_test_temp_dir("change_passphrase");
    let state = AppState::default();

    let title = "Obra Segura".to_string();
    let initial_pass = "SenhaInicialForte!2026".to_string();
    let new_pass = "NovaSenhaUltraSecreta!2026".to_string();

    let init_res = create_vault_internal(&state, base_dir.to_str().unwrap(), &title, &initial_pass)
        .expect("vault creation failed");
    let recovery_token = init_res.recovery_key_token.clone();
    let vault_path = init_res.vault_path.clone();

    // 1. Cria uma cena com texto
    let scene = create_manuscript_item_internal(&state, "scene", None, "Cena Confidencial")
        .expect("create scene failed");
    save_scene_content_internal(
        &state,
        &scene.id,
        "Conteúdo literário de alta confidencialidade.",
    )
    .expect("save scene failed");

    // 2. Altera a senha mestra
    change_vault_passphrase_internal(&state, &initial_pass, &new_pass)
        .expect("change passphrase failed");

    // 3. Fecha o cofre e tenta abrir com a senha antiga (deve falhar)
    close_vault_internal(&state).expect("close failed");
    let old_attempt = open_vault_with_passphrase_internal(&state, &vault_path, &initial_pass);
    assert!(
        old_attempt.is_err(),
        "Senha antiga não deveria mais abrir o cofre"
    );

    // 4. Abre com a nova senha (deve suceder e manter o conteúdo acessível)
    let new_attempt = open_vault_with_passphrase_internal(&state, &vault_path, &new_pass)
        .expect("open with new pass failed");
    assert_eq!(new_attempt.project_title, Some(title.clone()));

    let loaded_content = load_scene_content_internal(&state, &scene.id).expect("load scene failed");
    assert_eq!(
        loaded_content,
        "Conteúdo literário de alta confidencialidade."
    );

    // 5. Fecha e valida que a chave de recuperação original continua válida (Slot 1 preservado)
    close_vault_internal(&state).expect("close failed");
    let rec_attempt = open_vault_with_recovery_key_internal(&state, &vault_path, &recovery_token)
        .expect("recovery key failed");
    assert_eq!(rec_attempt.project_title, Some(title));

    cleanup_test_dir(&base_dir);
}

#[test]
fn test_vault_reencryption_and_dek_rotation_lifecycle() {
    let base_dir = create_test_temp_dir("reencrypt_vault");
    let state = AppState::default();

    let title = "Obra Reencriptada".to_string();
    let passphrase = "FraseMestraOriginal!2026".to_string();

    let init_res = create_vault_internal(&state, base_dir.to_str().unwrap(), &title, &passphrase)
        .expect("vault creation failed");
    let old_recovery_token = init_res.recovery_key_token.clone();
    let vault_path = init_res.vault_path.clone();

    // 1. Cria cena e escreve dados
    let scene = create_manuscript_item_internal(&state, "scene", None, "Capítulo Zero")
        .expect("create scene failed");
    save_scene_content_internal(
        &state,
        &scene.id,
        "Texto secreto que será recifrado com nova DEK.",
    )
    .expect("save scene failed");

    // 2. Executa recifragem integral do cofre (rotaciona DEK)
    let new_recovery_token =
        reencrypt_vault_internal(&state, &passphrase).expect("reencryption failed");
    assert_ne!(old_recovery_token, new_recovery_token);

    // 3. Fecha o cofre e tenta abrir com a chave de recuperação antiga (deve falhar)
    close_vault_internal(&state).expect("close failed");
    let old_rec_attempt =
        open_vault_with_recovery_key_internal(&state, &vault_path, &old_recovery_token);
    assert!(
        old_rec_attempt.is_err(),
        "Chave de recuperação antiga não deveria mais funcionar"
    );

    // 4. Abre com a nova chave de recuperação (deve suceder)
    let new_rec_attempt =
        open_vault_with_recovery_key_internal(&state, &vault_path, &new_recovery_token)
            .expect("new recovery key should unlock");
    assert_eq!(new_rec_attempt.project_title, Some(title.clone()));

    let content = load_scene_content_internal(&state, &scene.id).expect("load scene failed");
    assert_eq!(content, "Texto secreto que será recifrado com nova DEK.");

    // 5. Fecha e abre com a senha mestra
    close_vault_internal(&state).expect("close failed");
    let pass_attempt = open_vault_with_passphrase_internal(&state, &vault_path, &passphrase)
        .expect("passphrase should unlock after rekey");
    assert_eq!(pass_attempt.project_title, Some(title));

    cleanup_test_dir(&base_dir);
}

#[test]
fn test_vault_keyring_integration() {
    let base_dir = create_test_temp_dir("keyring_integration");
    let state = AppState::default();

    let title = "Obra Chaveiro".to_string();
    let passphrase = "FraseSeguraKeyring!2026".to_string();

    let init_res = create_vault_internal(&state, base_dir.to_str().unwrap(), &title, &passphrase)
        .expect("vault creation failed");
    let vault_path = init_res.vault_path.clone();

    // 1. Inicialmente o status do chaveiro deve ser falso ou suportado
    let status_init = get_vault_keyring_status_internal(&state);
    if let Ok(false) = status_init {
        // 2. Salva credencial no chaveiro
        save_to_keyring_internal(&state, &passphrase).expect("save to keyring failed");
        assert!(get_vault_keyring_status_internal(&state).unwrap());

        // 3. Fecha o cofre e abre via chaveiro
        close_vault_internal(&state).expect("close failed");
        let unlocked = open_vault_with_keyring_internal(&state, &vault_path)
            .expect("open with keyring failed");
        assert_eq!(unlocked.project_title, Some(title));

        // 4. Remove do chaveiro
        remove_from_keyring_internal(&state).expect("remove from keyring failed");
        assert!(!get_vault_keyring_status_internal(&state).unwrap());
    }

    cleanup_test_dir(&base_dir);
}

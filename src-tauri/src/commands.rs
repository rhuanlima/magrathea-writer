//! Comandos IPC expostos para o frontend via ponte Tauri.

use crate::crypto::SecretKey;
use crate::state::AppState;
use crate::vault::VaultStorage;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Resposta retornada após a criação bem-sucedida de um cofre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateVaultResponse {
    /// Identificador UUID único do cofre.
    pub vault_id: String,
    /// Título do romance ou projeto literário.
    pub project_title: String,
    /// Chave de recuperação de emergência formatada em token para exibição única.
    pub recovery_key_token: String,
    /// Caminho absoluto do cofre no disco.
    pub vault_path: String,
}

/// Estado atual do cofre em memória.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultStatusResponse {
    /// Indica se existe um cofre carregado em memória.
    pub is_open: bool,
    /// Indica se o cofre está bloqueado.
    pub is_locked: bool,
    /// UUID do cofre ativo, se houver.
    pub vault_id: Option<String>,
    /// Título do projeto ativo, se houver.
    pub project_title: Option<String>,
    /// Caminho do cofre no disco, se houver.
    pub vault_path: Option<String>,
}

/// Retorna mensagem inicial de boas-vindas para verificação da ponte IPC.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Magrathea Writer.", name)
}

/// Obtém o diretório padrão sugerido para a criação de novos cofres.
#[tauri::command]
pub fn get_default_vault_dir() -> String {
    let mut default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(user_home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let docs = PathBuf::from(user_home).join("Documents");
        if docs.exists() && docs.is_dir() {
            default_dir = docs;
        }
    }
    default_dir.display().to_string()
}

/// Cria e inicializa um novo cofre `.magrathea` em disco e o ativa em memória.
#[tauri::command]
pub fn create_vault(
    state: tauri::State<'_, AppState>,
    dir_path: String,
    project_title: String,
    passphrase: String,
) -> Result<CreateVaultResponse, String> {
    create_vault_internal(&state, &dir_path, &project_title, &passphrase)
}

/// Lógica interna de criação de cofre desacoplada do container de injeção Tauri.
pub fn create_vault_internal(
    state: &AppState,
    dir_path: &str,
    project_title: &str,
    passphrase: &str,
) -> Result<CreateVaultResponse, String> {
    let trimmed_title = project_title.trim();
    if trimmed_title.is_empty() {
        return Err("O título do projeto não pode ser vazio.".to_string());
    }

    if passphrase.len() < 10 {
        return Err("A senha mestra deve conter no mínimo 10 caracteres.".to_string());
    }

    let raw_path = PathBuf::from(dir_path.trim());
    let target_vault_path = resolve_vault_path(&raw_path, trimmed_title);

    let recovery_key = SecretKey::generate();
    let recovery_token = recovery_key.to_formatted_token();

    let unlocked_vault = VaultStorage::create(
        &target_vault_path,
        trimmed_title,
        passphrase.as_bytes(),
        &recovery_key,
        None,
    )
    .map_err(|e| format!("Falha ao criar cofre: {e}"))?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let final_path = target_vault_path.display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(final_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(trimmed_title.to_string());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;
    *guard = Some(unlocked_vault);

    Ok(CreateVaultResponse {
        vault_id,
        project_title: trimmed_title.to_string(),
        recovery_key_token: recovery_token,
        vault_path: final_path,
    })
}

/// Obtém o estado e os metadados do cofre atualmente ativo ou bloqueado.
#[tauri::command]
pub fn get_vault_status(state: tauri::State<'_, AppState>) -> Result<VaultStatusResponse, String> {
    get_vault_status_internal(&state)
}

/// Lógica interna para obter o status atual do cofre em memória.
pub fn get_vault_status_internal(state: &AppState) -> Result<VaultStatusResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    match &*guard {
        Some(vault) => Ok(VaultStatusResponse {
            is_open: true,
            is_locked: vault.is_locked(),
            vault_id: Some(vault.header().vault_id.clone()),
            project_title: vault.manifest().ok().map(|m| m.project_title.clone()),
            vault_path: Some(vault.root_path().display().to_string()),
        }),
        None => {
            let last_path = state.last_vault_path.lock().ok().and_then(|p| p.clone());
            let last_title = state.last_project_title.lock().ok().and_then(|t| t.clone());
            Ok(VaultStatusResponse {
                is_open: false,
                is_locked: true,
                vault_id: None,
                project_title: last_title,
                vault_path: last_path,
            })
        }
    }
}

/// Bloqueia o cofre atual em memória, descartando e zeroizando a chave DEK.
#[tauri::command]
pub fn lock_vault(state: tauri::State<'_, AppState>) -> Result<(), String> {
    lock_vault_internal(&state)
}

/// Lógica interna para bloquear o cofre e zeroizar a DEK.
pub fn lock_vault_internal(state: &AppState) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Some(ref mut vault) = *guard {
        if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
            *last_path_guard = Some(vault.root_path().display().to_string());
        }
        if let Ok(manifest) = vault.manifest() {
            if let Ok(mut last_title_guard) = state.last_project_title.lock() {
                *last_title_guard = Some(manifest.project_title.clone());
            }
        }
        vault.lock();
    }
    *guard = None;

    Ok(())
}

/// Abre um cofre existente decifrando o Slot 0 com a frase-senha mestra.
#[tauri::command]
pub fn open_vault_with_passphrase(
    state: tauri::State<'_, AppState>,
    vault_path: String,
    passphrase: String,
) -> Result<VaultStatusResponse, String> {
    open_vault_with_passphrase_internal(&state, &vault_path, &passphrase)
}

/// Lógica interna para abertura de cofre com frase-senha.
pub fn open_vault_with_passphrase_internal(
    state: &AppState,
    vault_path: &str,
    passphrase: &str,
) -> Result<VaultStatusResponse, String> {
    let path = PathBuf::from(vault_path.trim());
    if !path.exists() {
        return Err("O caminho especificado para o cofre não foi encontrado.".to_string());
    }

    let unlocked_vault = VaultStorage::open_with_passphrase(&path, passphrase.as_bytes())
        .map_err(|_| "Senha incorreta ou cofre corrompido.".to_string())?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let project_title = unlocked_vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Sem Título".to_string());
    let canonical_path = unlocked_vault.root_path().display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(canonical_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(project_title.clone());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;
    *guard = Some(unlocked_vault);

    Ok(VaultStatusResponse {
        is_open: true,
        is_locked: false,
        vault_id: Some(vault_id),
        project_title: Some(project_title),
        vault_path: Some(canonical_path),
    })
}

/// Abre um cofre existente decifrando o Slot 1 com a chave de recuperação de emergência.
#[tauri::command]
pub fn open_vault_with_recovery_key(
    state: tauri::State<'_, AppState>,
    vault_path: String,
    recovery_key: String,
) -> Result<VaultStatusResponse, String> {
    open_vault_with_recovery_key_internal(&state, &vault_path, &recovery_key)
}

/// Lógica interna para abertura de cofre com chave de recuperação.
pub fn open_vault_with_recovery_key_internal(
    state: &AppState,
    vault_path: &str,
    recovery_key: &str,
) -> Result<VaultStatusResponse, String> {
    let path = PathBuf::from(vault_path.trim());
    if !path.exists() {
        return Err("O caminho especificado para o cofre não foi encontrado.".to_string());
    }

    let key = SecretKey::from_hex(recovery_key.trim())
        .map_err(|_| "Chave de recuperação inválida ou malformatada.".to_string())?;

    let unlocked_vault = VaultStorage::open_with_recovery_key(&path, &key)
        .map_err(|_| "Chave de recuperação incorreta ou cofre corrompido.".to_string())?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let project_title = unlocked_vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Sem Título".to_string());
    let canonical_path = unlocked_vault.root_path().display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(canonical_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(project_title.clone());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;
    *guard = Some(unlocked_vault);

    Ok(VaultStatusResponse {
        is_open: true,
        is_locked: false,
        vault_id: Some(vault_id),
        project_title: Some(project_title),
        vault_path: Some(canonical_path),
    })
}

/// Fecha e desvincula o cofre atual, limpando metadados de sessão recente.
#[tauri::command]
pub fn close_vault(state: tauri::State<'_, AppState>) -> Result<(), String> {
    close_vault_internal(&state)
}

/// Lógica interna para fechar e desvincular o cofre.
pub fn close_vault_internal(state: &AppState) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Some(ref mut vault) = *guard {
        vault.lock();
    }
    *guard = None;

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = None;
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = None;
    }

    Ok(())
}

/// Constrói o caminho canônico do cofre com a extensão `.magrathea`.
fn resolve_vault_path(base_dir: &Path, title: &str) -> PathBuf {
    let path_str = base_dir.display().to_string();
    if path_str.ends_with(".magrathea") {
        return base_dir.to_path_buf();
    }

    // Sanitiza o nome do título para compor o nome de pasta
    let sanitized_name: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let folder_name = format!("{sanitized_name}.magrathea");
    base_dir.join(folder_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_message() {
        let result = greet("Author");
        assert_eq!(result, "Hello, Author! Welcome to Magrathea Writer.");
    }

    #[test]
    fn test_resolve_vault_path() {
        let base = PathBuf::from("/tmp/projects");
        let resolved = resolve_vault_path(&base, "Meu Romance 1");
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/projects/Meu_Romance_1.magrathea")
        );

        let direct = PathBuf::from("/tmp/projects/Obra.magrathea");
        let resolved_direct = resolve_vault_path(&direct, "Qualquer Titulo");
        assert_eq!(resolved_direct, direct);
    }

    #[test]
    fn test_get_default_vault_dir_not_empty() {
        let dir = get_default_vault_dir();
        assert!(!dir.is_empty());
    }

    #[test]
    fn test_vault_commands_lifecycle_and_zeroization() {
        let state = AppState::default();
        let unique_suffix: u64 = rand::random();
        let temp_dir = std::env::temp_dir().join(format!("test_cmd_lifecycle_{unique_suffix}"));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let fixture_key_str = "correto-cavalo-bateria-grampo-romance";

        // 1. Criação do cofre
        let create_res = create_vault_internal(
            &state,
            &temp_dir.display().to_string(),
            "O Guia Galactico",
            fixture_key_str,
        )
        .expect("deve criar cofre");

        assert_eq!(create_res.project_title, "O Guia Galactico");
        assert!(!create_res.recovery_key_token.is_empty());

        // 2. Status após criação (aberto)
        let status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(status.is_open);
        assert!(!status.is_locked);
        assert_eq!(status.project_title.as_deref(), Some("O Guia Galactico"));

        // 3. Bloqueio com descarte de sessão e zeroização em memória
        lock_vault_internal(&state).expect("deve bloquear");

        let locked_status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(!locked_status.is_open);
        assert!(locked_status.is_locked);
        assert_eq!(
            locked_status.vault_path.as_deref(),
            Some(create_res.vault_path.as_str())
        );
        assert_eq!(
            locked_status.project_title.as_deref(),
            Some("O Guia Galactico")
        );

        // 4. Tentativa de desbloqueio com senha errada (deve falhar com segurança)
        let wrong_res = open_vault_with_passphrase_internal(
            &state,
            &create_res.vault_path,
            "senha-errada-incorreta",
        );
        assert!(wrong_res.is_err());

        // 5. Desbloqueio com frase-senha correta
        let unlock_res =
            open_vault_with_passphrase_internal(&state, &create_res.vault_path, fixture_key_str)
                .expect("deve desbloquear com frase-senha");

        assert!(unlock_res.is_open);
        assert!(!unlock_res.is_locked);

        // 6. Novo bloqueio
        lock_vault_internal(&state).expect("deve bloquear");

        // 7. Desbloqueio com chave de recuperação de emergência
        let recovery_res = open_vault_with_recovery_key_internal(
            &state,
            &create_res.vault_path,
            &create_res.recovery_key_token,
        )
        .expect("deve desbloquear com chave de recuperacao");

        assert!(recovery_res.is_open);
        assert!(!recovery_res.is_locked);

        // 8. Fechamento total do cofre (close_vault)
        close_vault_internal(&state).expect("deve fechar cofre");
        let closed_status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(!closed_status.is_open);
        assert!(closed_status.is_locked);
        assert!(closed_status.vault_path.is_none());
        assert!(closed_status.project_title.is_none());

        // Limpeza
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

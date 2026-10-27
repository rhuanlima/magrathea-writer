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

/// Obtém o estado e os metadados do cofre atualmente ativo em memória.
#[tauri::command]
pub fn get_vault_status(state: tauri::State<'_, AppState>) -> Result<VaultStatusResponse, String> {
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
        None => Ok(VaultStatusResponse {
            is_open: false,
            is_locked: true,
            vault_id: None,
            project_title: None,
            vault_path: None,
        }),
    }
}

/// Bloqueia o cofre atual em memória, descartando e zeroizando a chave DEK.
#[tauri::command]
pub fn lock_vault(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Some(ref mut vault) = *guard {
        vault.lock();
    }
    *guard = None;

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
}

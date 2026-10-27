//! Estado global da aplicação compartilhado no Tauri.

use crate::vault::UnlockedVault;
use std::sync::Mutex;

/// Estado compartilhado da aplicação mantido pelo runtime do Tauri.
pub struct AppState {
    /// Instância ativa do cofre aberto em memória.
    pub active_vault: Mutex<Option<UnlockedVault>>,
    /// Caminho do último cofre utilizado (mantido ao bloquear para permitir desbloqueio rápido).
    pub last_vault_path: Mutex<Option<String>>,
    /// Título do último romance utilizado.
    pub last_project_title: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_vault: Mutex::new(None),
            last_vault_path: Mutex::new(None),
            last_project_title: Mutex::new(None),
        }
    }
}

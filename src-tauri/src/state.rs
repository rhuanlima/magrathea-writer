//! Estado global da aplicação compartilhado no Tauri.

use crate::vault::UnlockedVault;
use std::sync::Mutex;

/// Estado compartilhado da aplicação mantido pelo runtime do Tauri.
pub struct AppState {
    /// Instância ativa do cofre aberto em memória.
    pub active_vault: Mutex<Option<UnlockedVault>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_vault: Mutex::new(None),
        }
    }
}

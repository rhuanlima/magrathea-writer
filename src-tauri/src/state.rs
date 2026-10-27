//! Estado global da aplicação compartilhado no Tauri.

use crate::index::VaultIndex;
use crate::refactor::RefactorSnapshot;
use crate::vault::UnlockedVault;
use std::collections::HashMap;
use std::sync::Mutex;

/// Estado compartilhado da aplicação mantido pelo runtime do Tauri.
pub struct AppState {
    /// Instância ativa do cofre aberto em memória.
    pub active_vault: Mutex<Option<UnlockedVault>>,
    /// Índice em memória de entidades e referências bidirecionais (zerado ao bloquear).
    pub index: Mutex<Option<VaultIndex>>,
    /// Snapshots recentes de refatoração para desfazer/rollback (zerados ao bloquear).
    pub refactor_snapshots: Mutex<HashMap<String, RefactorSnapshot>>,
    /// Caminho do último cofre utilizado (mantido ao bloquear para permitir desbloqueio rápido).
    pub last_vault_path: Mutex<Option<String>>,
    /// Título do último romance utilizado.
    pub last_project_title: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_vault: Mutex::new(None),
            index: Mutex::new(None),
            refactor_snapshots: Mutex::new(HashMap::new()),
            last_vault_path: Mutex::new(None),
            last_project_title: Mutex::new(None),
        }
    }
}

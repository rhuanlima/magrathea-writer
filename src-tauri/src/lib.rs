//! Core library module for Magrathea Writer desktop application.

pub mod commands;
pub mod crypto;
pub mod state;
pub mod vault;

/// Initializes and executes the Tauri desktop application.
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_default_vault_dir,
            commands::create_vault,
            commands::get_vault_status,
            commands::lock_vault,
            commands::open_vault_with_passphrase,
            commands::open_vault_with_recovery_key,
            commands::close_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running magrathea writer application");
}

//! Core library module for Magrathea Writer desktop application.

pub mod commands;
pub mod crypto;
pub mod document;
pub mod index;
pub mod keychain;
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
            commands::save_scene_content,
            commands::load_scene_content,
            commands::get_manuscript_tree,
            commands::create_manuscript_item,
            commands::rename_manuscript_item,
            commands::update_scene_metadata,
            commands::move_item_to_trash,
            commands::restore_item_from_trash,
            commands::empty_trash,
            commands::reorder_manuscript_items,
            commands::change_vault_passphrase,
            commands::reencrypt_vault,
            commands::get_vault_keyring_status,
            commands::save_to_keyring,
            commands::remove_from_keyring,
            commands::open_vault_with_keyring,
            commands::get_vault_index,
            commands::get_document_backlinks,
            commands::get_entity_preview,
            commands::rebuild_vault_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running magrathea writer application");
}

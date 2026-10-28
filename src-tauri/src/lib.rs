//! Core library module for Magrathea Writer desktop application.

pub mod backup;
pub mod commands;
pub mod compiler;
pub mod crypto;
pub mod document;
pub mod index;
pub mod keychain;
pub mod productivity;
pub mod refactor;
pub mod search_replace;
pub mod snapshots;
pub mod state;
pub mod timeline;
pub mod vault;
pub mod worldbuilding;

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
            commands::is_vault_passwordless,
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
            commands::preview_rename_entity,
            commands::apply_rename_refactor,
            commands::rollback_rename_refactor,
            commands::get_productivity_overview,
            commands::update_productivity_targets,
            commands::record_productivity_session,
            commands::get_entity_template,
            commands::create_world_entity,
            commands::import_entity_image,
            commands::load_entity_image,
            commands::delete_entity_image,
            commands::list_entity_images,
            commands::get_entity_appearances,
            commands::create_scene_snapshot,
            commands::list_scene_snapshots,
            commands::get_scene_snapshot_content,
            commands::compare_snapshot_with_current,
            commands::restore_scene_snapshot,
            commands::global_search,
            commands::preview_global_replace,
            commands::apply_global_replace,
            commands::rollback_global_replace,
            commands::create_vault_backup,
            commands::list_vault_backups,
            commands::load_compiler_settings,
            commands::save_compiler_settings,
            commands::compile_manuscript,
            commands::export_obsidian_markdown,
            commands::export_docx,
            commands::export_epub,
            commands::export_pdf_html,
            commands::export_txt_dossier,
            commands::get_vault_timeline,
            commands::pick_vault_path,
            commands::pick_directory,
            commands::pick_save_export_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running magrathea writer application");
}

//! Core library module for Magrathea Writer desktop application.

pub mod backup;
pub mod canvas;
pub mod commands;
pub mod compiler;
pub mod crypto;
pub mod document;
pub mod graph;
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

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::Emitter;

/// Configura e constrói o menu nativo da barra do sistema operacional (macOS Menu Bar / Windows).
pub fn setup_native_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // Menu do App (macOS)
    let app_menu = Submenu::with_items(
        handle,
        "Magrathea Writer",
        true,
        &[
            &PredefinedMenuItem::about(handle, Some("Sobre o Magrathea Writer"), None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::hide(handle, Some("Ocultar Magrathea Writer"))?,
            &PredefinedMenuItem::hide_others(handle, Some("Ocultar Outros"))?,
            &PredefinedMenuItem::show_all(handle, Some("Mostrar Todos"))?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, Some("Encerrar Magrathea Writer"))?,
        ],
    )?;

    // Menu Arquivo
    let file_menu = Submenu::with_items(
        handle,
        "Arquivo",
        true,
        &[
            &MenuItem::with_id(
                handle,
                "menu_new_world",
                "Construir Novo Mundo...",
                true,
                Some("CmdOrCtrl+N"),
            )?,
            &MenuItem::with_id(
                handle,
                "menu_open_world",
                "Explorar Mundo Existente...",
                true,
                Some("CmdOrCtrl+O"),
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_backup",
                "Fazer Backup Cifrado...",
                true,
                Some("CmdOrCtrl+Shift+B"),
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_lock_world",
                "Trancar Mundo",
                true,
                Some("CmdOrCtrl+L"),
            )?,
        ],
    )?;

    // Menu Editar
    let edit_menu = Submenu::with_items(
        handle,
        "Editar",
        true,
        &[
            &PredefinedMenuItem::undo(handle, Some("Desfazer"))?,
            &PredefinedMenuItem::redo(handle, Some("Refazer"))?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::cut(handle, Some("Recortar"))?,
            &PredefinedMenuItem::copy(handle, Some("Copiar"))?,
            &PredefinedMenuItem::paste(handle, Some("Colar"))?,
            &PredefinedMenuItem::select_all(handle, Some("Selecionar Tudo"))?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_find",
                "Busca Global no Projeto...",
                true,
                Some("CmdOrCtrl+Shift+F"),
            )?,
        ],
    )?;

    // Menu Exibir
    let view_menu = Submenu::with_items(
        handle,
        "Exibir",
        true,
        &[
            &MenuItem::with_id(
                handle,
                "menu_toggle_sidebar",
                "Alternar Barra Lateral",
                true,
                Some("CmdOrCtrl+\\"),
            )?,
            &MenuItem::with_id(
                handle,
                "menu_toggle_inspector",
                "Alternar Inspetor de Fichas",
                true,
                Some("CmdOrCtrl+I"),
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_zen_mode",
                "Modo Foco Zen Total",
                true,
                Some("F11"),
            )?,
            &MenuItem::with_id(
                handle,
                "menu_toggle_theme",
                "Alternar Modo Claro / Escuro",
                true,
                None::<&str>,
            )?,
        ],
    )?;

    // Menu Ferramentas Narrativas
    let tools_menu = Submenu::with_items(
        handle,
        "Ferramentas",
        true,
        &[
            &MenuItem::with_id(
                handle,
                "menu_timeline",
                "Linha do Tempo Literária...",
                true,
                Some("CmdOrCtrl+Shift+T"),
            )?,
            &MenuItem::with_id(
                handle,
                "menu_graph",
                "Grafo Narrativo & Canvas...",
                true,
                Some("CmdOrCtrl+Shift+G"),
            )?,
            &MenuItem::with_id(
                handle,
                "menu_productivity",
                "Metas & Produtividade...",
                true,
                Some("CmdOrCtrl+Shift+P"),
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_security",
                "Segurança do Mundo...",
                true,
                None::<&str>,
            )?,
        ],
    )?;

    // Menu Exportação Editorial
    let export_menu = Submenu::with_items(
        handle,
        "Exportar",
        true,
        &[
            &MenuItem::with_id(
                handle,
                "menu_export_panel",
                "Painel de Publicação Editorial...",
                true,
                Some("CmdOrCtrl+Shift+E"),
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                "menu_export_docx",
                "Exportar para Word (.docx)...",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                "menu_export_epub",
                "Exportar para Livro Digital (.epub)...",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                "menu_export_pdf",
                "Exportar para Leitura / PDF (.html)...",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                "menu_export_obsidian",
                "Exportar para Obsidian (.md)...",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                "menu_export_txt",
                "Exportar Texto Puro + Dossiê...",
                true,
                None::<&str>,
            )?,
        ],
    )?;

    let menu = Menu::with_items(
        handle,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &tools_menu,
            &export_menu,
        ],
    )?;

    app.set_menu(menu)?;

    // Quando o usuário aciona um item de menu, despacha o evento correspondente para o frontend
    let app_handle = app.handle().clone();
    app.on_menu_event(move |_app, event| {
        let event_id = event.id().as_ref();
        let _ = app_handle.emit(event_id, ());
    });

    Ok(())
}

/// Initializes and executes the Tauri desktop application.
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .setup(|app| {
            setup_native_menu(app)?;
            Ok(())
        })
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
            commands::get_vault_graph,
            commands::get_local_graph,
            commands::load_canvas,
            commands::save_canvas,
            commands::list_canvases,
            commands::delete_canvas,
        ])
        .run(tauri::generate_context!())
        .expect("error while running magrathea writer application");
}

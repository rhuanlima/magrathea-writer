//! Core library module for Magrathea Writer desktop application.

pub mod commands;

/// Initializes and executes the Tauri desktop application.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::greet])
        .run(tauri::generate_context!())
        .expect("error while running magrathea writer application");
}

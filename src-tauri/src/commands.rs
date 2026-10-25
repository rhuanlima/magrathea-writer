//! IPC commands exposed to the frontend webview.

/// Greets a user or module with a welcome message.
///
/// This serves as an initial smoke-test command for the Tauri IPC bridge.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Magrathea Writer.", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_message() {
        let result = greet("Author");
        assert_eq!(result, "Hello, Author! Welcome to Magrathea Writer.");
    }
}

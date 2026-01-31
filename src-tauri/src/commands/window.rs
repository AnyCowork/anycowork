use tauri::AppHandle;
#[cfg(debug_assertions)]
use tauri::Manager;

/// Toggle devtools for the main window
#[tauri::command]
pub async fn toggle_devtools(_app: AppHandle) -> Result<(), String> {
    #[cfg(debug_assertions)]
    {
        if let Some(window) = _app.webview_windows().values().next() {
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
        }
    }
    Ok(())
}

/// Check if running in development mode
#[tauri::command]
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

/// Get the current working directory as an absolute path
#[tauri::command]
pub fn get_current_working_directory() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get current directory: {}", e))
}

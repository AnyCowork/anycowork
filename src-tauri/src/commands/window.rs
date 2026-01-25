use tauri::AppHandle;

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

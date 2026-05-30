use std::path::PathBuf;
use tauri::Manager;

/// Detects whether the app is running in portable mode.
/// Portable mode is triggered by placing a file named `portable` (no extension)
/// in the same directory as the executable.
pub fn is_portable() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("portable").exists();
        }
    }
    false
}

/// Returns the appropriate data directory for the app.
/// In portable mode, this is a `data` folder next to the executable.
/// Otherwise, it falls back to the standard OS app-data directory.
pub fn data_dir(app: &impl Manager<tauri::Wry>) -> PathBuf {
    if is_portable() {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("data");
            }
        }
    }
    app.path().app_data_dir().expect("failed to get app data dir")
}

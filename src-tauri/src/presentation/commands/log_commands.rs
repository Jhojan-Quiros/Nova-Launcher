use std::path::PathBuf;
use tauri::State;
use crate::domain::errors::LauncherError;
use crate::infrastructure::logging::LogEntry;
use crate::presentation::state::AppState;

#[tauri::command]
pub fn get_logs(state: State<'_, AppState>) -> Vec<LogEntry> {
    state.log_buffer.get_logs()
}

#[tauri::command]
pub fn clear_logs(state: State<'_, AppState>) {
    state.log_buffer.clear();
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), LauncherError> {
    let p = PathBuf::from(&path);
    if p.exists() {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(&p).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(&p).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(&p).spawn();
        }
        Ok(())
    } else {
        Err(LauncherError::not_found(format!("Directory {:?} does not exist", p)))
    }
}
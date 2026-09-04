use tauri::State;
use crate::domain::entities::JavaRuntime;
use crate::domain::errors::LauncherError;
use crate::presentation::state::AppState;

#[tauri::command]
pub async fn detect_java(
    state: State<'_, AppState>,
) -> Result<Vec<JavaRuntime>, LauncherError> {
    state.detect_java_uc.execute().await
}

#[tauri::command]
pub async fn probe_java(
    state: State<'_, AppState>,
    path: String,
) -> Result<JavaRuntime, LauncherError> {
    state.detect_java_uc.probe(&path).await
}
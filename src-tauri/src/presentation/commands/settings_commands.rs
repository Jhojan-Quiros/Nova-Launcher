use tauri::State;
use crate::application::dto::UpdateSettingsDto;
use crate::domain::entities::AppSettings;
use crate::domain::errors::LauncherError;
use crate::presentation::state::AppState;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, LauncherError> {
    state.get_settings_uc.execute().await
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    dto: UpdateSettingsDto,
) -> Result<AppSettings, LauncherError> {
    state.update_settings_uc.execute(dto).await
}
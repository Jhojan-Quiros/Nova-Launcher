use tauri::State;
use crate::application::dto::{VersionFilterDto, VersionListResponseDto};
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::forge_installer::ForgeVersionOption;
use crate::presentation::state::AppState;

#[tauri::command]
pub async fn get_minecraft_versions(
    state: State<'_, AppState>,
    filter: Option<VersionFilterDto>,
) -> Result<VersionListResponseDto, LauncherError> {
    let filter = filter.unwrap_or_default();
    state.fetch_versions_uc.execute(filter).await
}

#[tauri::command]
pub async fn install_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), LauncherError> {
    state.install_instance_uc.execute(&instance_id).await
}

#[tauri::command]
pub async fn get_forge_versions(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<Vec<ForgeVersionOption>, LauncherError> {
    state.get_forge_versions_uc.execute(&mc_version).await
}

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), LauncherError> {
    state.launch_instance_uc.execute(&instance_id).await
}
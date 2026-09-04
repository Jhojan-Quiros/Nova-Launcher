use tauri::State;
use crate::application::dto::{CreateInstanceDto, UpdateInstanceDto};
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;
use crate::presentation::state::AppState;

#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    dto: CreateInstanceDto,
) -> Result<Instance, LauncherError> {
    state.create_instance_uc.execute(dto).await
}

#[tauri::command]
pub async fn list_instances(
    state: State<'_, AppState>,
) -> Result<Vec<Instance>, LauncherError> {
    state.list_instances_uc.execute().await
}

#[tauri::command]
pub async fn get_instance(
    state: State<'_, AppState>,
    id: String,
) -> Result<Instance, LauncherError> {
    state.get_instance_uc.execute(&id).await
}

#[tauri::command]
pub async fn update_instance(
    state: State<'_, AppState>,
    id: String,
    dto: UpdateInstanceDto,
) -> Result<Instance, LauncherError> {
    state.update_instance_uc.execute(&id, dto).await
}

#[tauri::command]
pub async fn delete_instance(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, LauncherError> {
    state.delete_instance_uc.execute(&id).await
}
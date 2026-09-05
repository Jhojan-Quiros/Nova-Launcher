use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::application::modpacks::dto::{CatalogItemWithStatusDto, InstallModpackDto};
use crate::domain::modpacks::*;
use crate::infrastructure::modpacks::ModpackProgressCallback;
use crate::presentation::state::AppState;

fn create_progress_emitter(app_handle: &AppHandle) -> ModpackProgressCallback {
    let app_handle = app_handle.clone();
    let last_emit_ms = Arc::new(std::sync::atomic::AtomicU64::new(0));
    Arc::new(move |progress: ModpackDownloadJobProgress| {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let last = last_emit_ms.load(std::sync::atomic::Ordering::Relaxed);
        if progress.percentage >= 100.0 || now_ms.saturating_sub(last) >= 80 {
            last_emit_ms.store(now_ms, std::sync::atomic::Ordering::Relaxed);
            let _ = app_handle.emit("modpack-update-progress", &progress);
        }
    })
}

#[tauri::command]
pub async fn get_modpack_catalog(
    state: State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<Vec<CatalogItemWithStatusDto>, ModpackError> {
    state
        .get_available_modpacks_uc
        .execute(force_refresh.unwrap_or(false))
        .await
}

#[tauri::command]
pub async fn get_modpack_details(
    state: State<'_, AppState>,
    manifest_url: String,
) -> Result<(ModpackMainManifest, ModpackVersionManifest), ModpackError> {
    state.get_modpack_details_uc.execute(&manifest_url).await
}

#[tauri::command]
pub async fn check_modpack_update(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Option<String>, ModpackError> {
    state.check_modpack_updates_uc.execute(&instance_id).await
}

#[tauri::command]
pub async fn check_all_modpack_updates(
    state: State<'_, AppState>,
) -> Result<usize, ModpackError> {
    state.check_all_installed_modpack_updates_uc.execute().await
}

#[tauri::command]
pub async fn prepare_modpack_update(
    state: State<'_, AppState>,
    instance_id: String,
    target_version: Option<String>,
) -> Result<UpdatePlan, ModpackError> {
    state
        .prepare_modpack_update_uc
        .execute(&instance_id, target_version.as_deref())
        .await
}

#[tauri::command]
pub async fn install_modpack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    dto: InstallModpackDto,
) -> Result<InstalledModpack, ModpackError> {
    let progress_cb = create_progress_emitter(&app_handle);
    state.install_modpack_uc.execute(dto, Some(progress_cb)).await
}

#[tauri::command]
pub async fn update_modpack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    target_version: Option<String>,
) -> Result<InstalledModpack, ModpackError> {
    let progress_cb = create_progress_emitter(&app_handle);
    state
        .update_modpack_uc
        .execute(&instance_id, target_version.as_deref(), Some(progress_cb))
        .await
}

#[tauri::command]
pub async fn verify_modpack(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<VerificationResult, ModpackError> {
    state.verify_modpack_uc.execute(&instance_id).await
}

#[tauri::command]
pub async fn repair_modpack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<VerificationResult, ModpackError> {
    let progress_cb = create_progress_emitter(&app_handle);
    state
        .repair_modpack_uc
        .execute(&instance_id, Some(progress_cb))
        .await
}

#[tauri::command]
pub async fn cancel_modpack_operation(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, ModpackError> {
    Ok(state.cancel_modpack_operation_uc.execute(&instance_id).await)
}

#[tauri::command]
pub async fn get_modpack_update_history(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ModpackUpdateHistoryRecord>, ModpackError> {
    state
        .get_modpack_update_history_uc
        .execute(&instance_id)
        .await
}

#[tauri::command]
pub async fn get_installed_modpack(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Option<InstalledModpack>, ModpackError> {
    state
        .installed_modpack_repo
        .get_by_instance_id(&instance_id)
        .await
}

#[tauri::command]
pub async fn list_installed_modpacks(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledModpack>, ModpackError> {
    state.installed_modpack_repo.list_all().await
}

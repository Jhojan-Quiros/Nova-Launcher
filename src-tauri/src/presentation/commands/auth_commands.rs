use tauri::State;
use crate::domain::entities::OfflineProfile;
use crate::presentation::state::AppState;

#[tauri::command]
pub async fn get_active_offline_profile(
    state: State<'_, AppState>,
) -> Result<OfflineProfile, String> {
    state
        .get_active_offline_profile_uc
        .execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_offline_profiles(
    state: State<'_, AppState>,
) -> Result<Vec<OfflineProfile>, String> {
    state
        .list_offline_profiles_uc
        .execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_offline_profile(
    username: String,
    state: State<'_, AppState>,
) -> Result<OfflineProfile, String> {
    state
        .create_offline_profile_uc
        .execute(username)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn select_offline_profile(
    username: String,
    state: State<'_, AppState>,
) -> Result<OfflineProfile, String> {
    state
        .select_offline_profile_uc
        .execute(username)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_offline_profile(
    username: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .delete_offline_profile_uc
        .execute(username)
        .await
        .map_err(|e| e.to_string())
}

use tauri::State;
use crate::application::dto::MicrosoftAccountInfoDto;
use crate::domain::entities::{MinecraftAccount, OfflineProfile};
use crate::infrastructure::auth::microsoft_auth::DeviceCodeInfo;
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

#[tauri::command]
pub async fn begin_microsoft_login(
    state: State<'_, AppState>,
) -> Result<DeviceCodeInfo, String> {
    state
        .begin_microsoft_login_uc
        .execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_microsoft_login(
    device_code: String,
    interval: u64,
    expires_in: u64,
    state: State<'_, AppState>,
) -> Result<MinecraftAccount, String> {
    state
        .complete_microsoft_login_uc
        .execute(device_code, interval, expires_in)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_microsoft_account(
    state: State<'_, AppState>,
) -> Result<Option<MicrosoftAccountInfoDto>, String> {
    state
        .get_microsoft_account_uc
        .execute()
        .await
        .map(|opt| opt.map(MicrosoftAccountInfoDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sign_out_microsoft(state: State<'_, AppState>) -> Result<(), String> {
    state
        .sign_out_microsoft_uc
        .execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activate_microsoft_account(state: State<'_, AppState>) -> Result<(), String> {
    state
        .activate_microsoft_account_uc
        .execute()
        .await
        .map_err(|e| e.to_string())
}

/// Opens a URL in the user's default browser (used for the Microsoft device-code
/// sign-in link). Reuses the same per-OS shell mechanism as `open_folder`.
#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Refusing to open a non-http(s) URL".to_string());
    }

    #[cfg(target_os = "windows")]
    let spawn_result = std::process::Command::new("explorer").arg(&url).spawn();
    #[cfg(target_os = "macos")]
    let spawn_result = std::process::Command::new("open").arg(&url).spawn();
    #[cfg(target_os = "linux")]
    let spawn_result = std::process::Command::new("xdg-open").arg(&url).spawn();

    spawn_result.map(|_| ()).map_err(|e| format!("Failed to open browser: {}", e))
}

use serde::{Deserialize, Serialize};
use super::account::AccountType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub launcher_dir: String,
    pub close_on_launch: bool,
    pub default_min_ram: u32,
    pub default_max_ram: u32,
    pub default_java_path: Option<String>,
    pub theme: String,
    pub blur_intensity: u32,
    pub max_concurrent_downloads: usize,
    pub window_width: u32,
    pub window_height: u32,
    pub modpack_catalog_url: Option<String>,
    pub check_updates_on_startup: bool,
    pub auto_check_interval_minutes: u32,
    pub allow_automatic_updates: bool,
    pub verify_files_before_launch: bool,
    pub desktop_notifications: bool,
    pub strict_modpack_mode_default: bool,
    /// Which account system to launch instances with: the local offline profile,
    /// or the signed-in Microsoft/Xbox account.
    pub active_auth_mode: AccountType,
    /// Azure AD "Application (client) ID" the user registers themselves at
    /// portal.azure.com to enable Microsoft sign-in (public client, device code flow).
    pub microsoft_client_id: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launcher_dir: "".to_string(),
            close_on_launch: false,
            default_min_ram: 2048,
            default_max_ram: 4096,
            default_java_path: None,
            theme: "dark".to_string(),
            blur_intensity: 100,
            max_concurrent_downloads: 8,
            window_width: 1180,
            window_height: 780,
            modpack_catalog_url: Some("https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/catalog.json".to_string()),
            check_updates_on_startup: true,
            auto_check_interval_minutes: 15,
            allow_automatic_updates: false,
            verify_files_before_launch: true,
            desktop_notifications: true,
            strict_modpack_mode_default: false,
            active_auth_mode: AccountType::Offline,
            microsoft_client_id: None,
        }
    }
}

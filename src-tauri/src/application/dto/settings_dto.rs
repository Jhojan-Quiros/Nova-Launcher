use serde::Deserialize;
use crate::domain::entities::AppSettings;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsDto {
    pub launcher_dir: Option<String>,
    pub close_on_launch: Option<bool>,
    pub default_min_ram: Option<u32>,
    pub default_max_ram: Option<u32>,
    pub default_java_path: Option<String>,
    pub theme: Option<String>,
    pub blur_intensity: Option<u32>,
    pub max_concurrent_downloads: Option<usize>,
    pub microsoft_client_id: Option<String>,
}

impl UpdateSettingsDto {
    pub fn apply_to(&self, target: &mut AppSettings) {
        if let Some(dir) = &self.launcher_dir {
            target.launcher_dir = dir.clone();
        }
        if let Some(val) = self.close_on_launch {
            target.close_on_launch = val;
        }
        if let Some(val) = self.default_min_ram {
            target.default_min_ram = val;
        }
        if let Some(val) = self.default_max_ram {
            target.default_max_ram = val;
        }
        if let Some(val) = &self.default_java_path {
            target.default_java_path = if val.is_empty() { None } else { Some(val.clone()) };
        }
        if let Some(t) = &self.theme {
            target.theme = t.clone();
        }
        if let Some(b) = self.blur_intensity {
            target.blur_intensity = b;
        }
        if let Some(m) = self.max_concurrent_downloads {
            target.max_concurrent_downloads = m;
        }
        if let Some(val) = &self.microsoft_client_id {
            target.microsoft_client_id = if val.trim().is_empty() { None } else { Some(val.trim().to_string()) };
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        }
    }
}

use serde::{Deserialize, Serialize};
use crate::domain::entities::minecraft_version::MinecraftVersion;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionFilterDto {
    pub show_releases: bool,
    pub show_snapshots: bool,
    pub show_old_beta: bool,
    pub show_old_alpha: bool,
}

impl Default for VersionFilterDto {
    fn default() -> Self {
        Self {
            show_releases: true,
            show_snapshots: false,
            show_old_beta: false,
            show_old_alpha: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResponseDto {
    pub latest_release: String,
    pub latest_snapshot: String,
    pub versions: Vec<MinecraftVersion>,
}

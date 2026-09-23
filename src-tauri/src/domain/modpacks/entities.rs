use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::entities::ModLoader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModpackStatus {
    NotInstalled,
    Installing,
    Installed,
    UpdateAvailable,
    Updating,
    Repairing,
    Corrupted,
    Failed,
}

impl ToString for ModpackStatus {
    fn to_string(&self) -> String {
        match self {
            ModpackStatus::NotInstalled => "not_installed".to_string(),
            ModpackStatus::Installing => "installing".to_string(),
            ModpackStatus::Installed => "installed".to_string(),
            ModpackStatus::UpdateAvailable => "update_available".to_string(),
            ModpackStatus::Updating => "updating".to_string(),
            ModpackStatus::Repairing => "repairing".to_string(),
            ModpackStatus::Corrupted => "corrupted".to_string(),
            ModpackStatus::Failed => "failed".to_string(),
        }
    }
}

impl From<&str> for ModpackStatus {
    fn from(s: &str) -> Self {
        match s {
            "installing" => ModpackStatus::Installing,
            "installed" => ModpackStatus::Installed,
            "update_available" => ModpackStatus::UpdateAvailable,
            "updating" => ModpackStatus::Updating,
            "repairing" => ModpackStatus::Repairing,
            "corrupted" => ModpackStatus::Corrupted,
            "failed" => ModpackStatus::Failed,
            _ => ModpackStatus::NotInstalled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModpack {
    pub id: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub minecraft_version: String,
    pub loader: ModLoader,
    #[serde(default)]
    pub loader_version: Option<String>,
    pub latest_version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    pub manifest_url: String,
    #[serde(default = "default_channel")]
    pub release_channel: Option<String>,
}

fn default_channel() -> Option<String> {
    Some("stable".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModpack {
    pub instance_id: String,
    pub modpack_id: String,
    pub name: String,
    pub installed_version: String,
    pub latest_known_version: Option<String>,
    pub manifest_url: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: ModpackStatus,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackCatalog {
    pub schema_version: u32,
    #[serde(default, alias = "updatedAt")]
    pub generated_at: Option<String>,
    pub modpacks: Vec<RemoteModpack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackMainManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub latest_version: String,
    pub minecraft_version: String,
    pub loader: String,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    pub manifest_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFileEntry {
    pub path: String,
    pub file_name: String,
    #[serde(default)]
    pub category: String,
    pub size: u64,
    pub sha256: String,
    pub url: String,
    #[serde(default)]
    pub mod_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VersionStats {
    pub total_files: usize,
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    #[serde(default)]
    pub unchanged: usize,
    pub total_size: u64,
    pub download_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackVersionManifest {
    pub schema_version: u32,
    pub pack_id: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub changelog: Vec<String>,
    pub stats: Option<VersionStats>,
    pub files: Vec<ManifestFileEntry>,
    #[serde(default)]
    pub removed_files: Vec<String>,
}

impl ModpackVersionManifest {
    /// Normalizes relative paths in file entries and removed_files.
    /// If flat filenames like "AI-Improvements.jar" are provided without a subfolder,
    /// they are automatically routed into the expected Minecraft directory (mods/, resourcepacks/).
    pub fn normalize_paths(&mut self) {
        for file in &mut self.files {
            let trimmed = file.path.trim().replace('\\', "/");
            if !trimmed.contains('/') {
                if trimmed.ends_with(".jar") {
                    file.path = format!("mods/{}", trimmed);
                } else if trimmed.ends_with(".zip") {
                    file.path = format!("resourcepacks/{}", trimmed);
                } else {
                    file.path = trimmed;
                }
            } else {
                file.path = trimmed;
            }
        }
        for file in &mut self.removed_files {
            let trimmed = file.trim().replace('\\', "/");
            if !trimmed.contains('/') {
                if trimmed.ends_with(".jar") {
                    *file = format!("mods/{}", trimmed);
                } else if trimmed.ends_with(".zip") {
                    *file = format!("resourcepacks/{}", trimmed);
                } else {
                    *file = trimmed;
                }
            } else {
                *file = trimmed;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlan {
    pub pack_id: String,
    pub from_version: Option<String>,
    pub to_version: String,
    pub downloads: Vec<ManifestFileEntry>,
    pub deletions: Vec<String>,
    pub unmodified_count: usize,
    pub total_download_size: u64,
    pub total_files: usize,
    pub changelog: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub total_checked: usize,
    pub valid: usize,
    pub missing: Vec<String>,
    pub modified: Vec<String>,
    pub unmanaged_user_files: Vec<String>,
    pub repair_size: u64,
    pub files_to_repair: Vec<ManifestFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackUpdateHistoryRecord {
    pub id: String,
    pub instance_id: String,
    pub pack_id: String,
    pub from_version: String,
    pub to_version: String,
    pub status: String,
    pub download_size: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackDownloadJobProgress {
    pub instance_id: String,
    pub pack_id: String,
    pub version: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub current_file: String,
    pub files_completed: usize,
    pub total_files: usize,
    pub stage: String,
}

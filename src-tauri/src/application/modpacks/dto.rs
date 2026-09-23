use serde::{Deserialize, Serialize};
use crate::domain::modpacks::{InstalledModpack, ModpackStatus, RemoteModpack};
use crate::domain::value_objects::ram_config::RamConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackDto {
    pub catalog_pack: RemoteModpack,
    pub instance_name: Option<String>,
    pub custom_ram: Option<RamConfig>,
    pub strict_mode: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemWithStatusDto {
    #[serde(rename = "modpack", alias = "remote")]
    pub remote: RemoteModpack,
    pub installed: Option<InstalledModpack>,
    pub status: ModpackStatus,
    pub update_available: bool,
    pub latest_version: String,
    pub installed_version: Option<String>,
    pub associated_instance_id: Option<String>,
}

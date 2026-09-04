use serde::{Deserialize, Serialize};
use crate::domain::entities::{Instance, ModLoader};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceDto {
    pub name: String,
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub min_ram: Option<u32>,
    pub max_ram: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceDto {
    pub name: Option<String>,
    pub java_path: Option<String>,
    pub min_ram: Option<u32>,
    pub max_ram: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceResponseDto {
    pub instance: Instance,
}

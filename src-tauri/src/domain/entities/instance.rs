use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::mod_loader::ModLoader;
use crate::domain::value_objects::ram_config::RamConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Idle,
    Installing,
    Ready,
    Running,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub game_directory: String,
    pub java_path: Option<String>,
    pub java_version: Option<u32>,
    pub ram: RamConfig,
    pub icon: Option<String>,
    pub status: InstanceStatus,
    pub total_play_time_seconds: u64,
    pub last_played_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Instance {
    pub fn new(
        id: String,
        name: String,
        minecraft_version: String,
        loader: ModLoader,
        loader_version: Option<String>,
        game_directory: String,
        ram: RamConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            minecraft_version,
            loader,
            loader_version,
            game_directory,
            java_path: None,
            java_version: None,
            ram,
            icon: None,
            status: InstanceStatus::Idle,
            total_play_time_seconds: 0,
            last_played_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_status(&mut self, status: InstanceStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn record_play_session(&mut self, session_duration_seconds: u64) {
        self.total_play_time_seconds += session_duration_seconds;
        self.last_played_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

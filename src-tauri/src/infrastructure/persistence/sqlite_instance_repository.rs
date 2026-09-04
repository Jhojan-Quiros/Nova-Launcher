use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use crate::domain::entities::{Instance, InstanceStatus, ModLoader};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;
use crate::domain::value_objects::ram_config::RamConfig;
use super::database::DbConn;

pub struct SqliteInstanceRepository {
    pool: DbConn,
}

impl SqliteInstanceRepository {
    pub fn new(pool: DbConn) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InstanceRepository for SqliteInstanceRepository {
    async fn find_all(&self) -> Result<Vec<Instance>, LauncherError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, minecraft_version, loader, loader_version, game_directory,
                    java_path, java_version, min_ram, max_ram, icon, status, status_details,
                    total_play_time_seconds, last_played_at, created_at, updated_at
             FROM instances
             ORDER BY updated_at DESC"
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let loader_str: String = row.get(3)?;
            let loader = match loader_str.as_str() {
                "fabric" => ModLoader::Fabric,
                "forge" => ModLoader::Forge,
                "neoforge" => ModLoader::NeoForge,
                _ => ModLoader::Vanilla,
            };

            let status_str: String = row.get(11)?;
            let status_details: Option<String> = row.get(12)?;
            let status = match status_str.as_str() {
                "installing" => InstanceStatus::Installing,
                "ready" => InstanceStatus::Ready,
                "running" => InstanceStatus::Running,
                "error" => InstanceStatus::Error(status_details.unwrap_or_else(|| "Unknown error".into())),
                _ => InstanceStatus::Idle,
            };

            let min_ram: u32 = row.get(8)?;
            let max_ram: u32 = row.get(9)?;

            let last_played_at: Option<String> = row.get(14)?;
            let created_at_str: String = row.get(15)?;
            let updated_at_str: String = row.get(16)?;

            Ok(Instance {
                id: row.get(0)?,
                name: row.get(1)?,
                minecraft_version: row.get(2)?,
                loader,
                loader_version: row.get(4)?,
                game_directory: row.get(5)?,
                java_path: row.get(6)?,
                java_version: row.get(7)?,
                ram: RamConfig { min_mb: min_ram, max_mb: max_ram },
                icon: row.get(10)?,
                status,
                total_play_time_seconds: row.get(13)?,
                last_played_at: last_played_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        }).map_err(|e| LauncherError::database(e.to_string()))?;

        let mut instances = Vec::new();
        for r in rows {
            instances.push(r.map_err(|e| LauncherError::database(e.to_string()))?);
        }

        Ok(instances)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Instance>, LauncherError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, minecraft_version, loader, loader_version, game_directory,
                    java_path, java_version, min_ram, max_ram, icon, status, status_details,
                    total_play_time_seconds, last_played_at, created_at, updated_at
             FROM instances
             WHERE id = ?1"
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        let result = stmt.query_row(params![id], |row| {
            let loader_str: String = row.get(3)?;
            let loader = match loader_str.as_str() {
                "fabric" => ModLoader::Fabric,
                "forge" => ModLoader::Forge,
                "neoforge" => ModLoader::NeoForge,
                _ => ModLoader::Vanilla,
            };

            let status_str: String = row.get(11)?;
            let status_details: Option<String> = row.get(12)?;
            let status = match status_str.as_str() {
                "installing" => InstanceStatus::Installing,
                "ready" => InstanceStatus::Ready,
                "running" => InstanceStatus::Running,
                "error" => InstanceStatus::Error(status_details.unwrap_or_else(|| "Unknown error".into())),
                _ => InstanceStatus::Idle,
            };

            let min_ram: u32 = row.get(8)?;
            let max_ram: u32 = row.get(9)?;
            let last_played_at: Option<String> = row.get(14)?;
            let created_at_str: String = row.get(15)?;
            let updated_at_str: String = row.get(16)?;

            Ok(Instance {
                id: row.get(0)?,
                name: row.get(1)?,
                minecraft_version: row.get(2)?,
                loader,
                loader_version: row.get(4)?,
                game_directory: row.get(5)?,
                java_path: row.get(6)?,
                java_version: row.get(7)?,
                ram: RamConfig { min_mb: min_ram, max_mb: max_ram },
                icon: row.get(10)?,
                status,
                total_play_time_seconds: row.get(13)?,
                last_played_at: last_played_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        }).optional().map_err(|e| LauncherError::database(e.to_string()))?;

        Ok(result)
    }

    async fn save(&self, instance: &Instance) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;

        let loader_str = match instance.loader {
            ModLoader::Vanilla => "vanilla",
            ModLoader::Fabric => "fabric",
            ModLoader::Forge => "forge",
            ModLoader::NeoForge => "neoforge",
        };

        let (status_str, status_details) = match &instance.status {
            InstanceStatus::Idle => ("idle", None),
            InstanceStatus::Installing => ("installing", None),
            InstanceStatus::Ready => ("ready", None),
            InstanceStatus::Running => ("running", None),
            InstanceStatus::Error(msg) => ("error", Some(msg.clone())),
        };

        let last_played = instance.last_played_at.map(|d| d.to_rfc3339());
        let created_at = instance.created_at.to_rfc3339();
        let updated_at = instance.updated_at.to_rfc3339();

        conn.execute(
            "INSERT INTO instances (
                id, name, minecraft_version, loader, loader_version, game_directory,
                java_path, java_version, min_ram, max_ram, icon, status, status_details,
                total_play_time_seconds, last_played_at, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                minecraft_version = excluded.minecraft_version,
                loader = excluded.loader,
                loader_version = excluded.loader_version,
                game_directory = excluded.game_directory,
                java_path = excluded.java_path,
                java_version = excluded.java_version,
                min_ram = excluded.min_ram,
                max_ram = excluded.max_ram,
                icon = excluded.icon,
                status = excluded.status,
                status_details = excluded.status_details,
                total_play_time_seconds = excluded.total_play_time_seconds,
                last_played_at = excluded.last_played_at,
                updated_at = excluded.updated_at",
            params![
                instance.id,
                instance.name,
                instance.minecraft_version,
                loader_str,
                instance.loader_version,
                instance.game_directory,
                instance.java_path,
                instance.java_version,
                instance.ram.min_mb,
                instance.ram.max_mb,
                instance.icon,
                status_str,
                status_details,
                instance.total_play_time_seconds,
                last_played,
                created_at,
                updated_at
            ],
        ).map_err(|e| LauncherError::database(format!("Failed to save instance: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool, LauncherError> {
        let conn = self.pool.lock().await;
        let rows_affected = conn.execute("DELETE FROM instances WHERE id = ?1", params![id])
            .map_err(|e| LauncherError::database(e.to_string()))?;
        Ok(rows_affected > 0)
    }

    async fn exists(&self, id: &str) -> Result<bool, LauncherError> {
        let conn = self.pool.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(1) FROM instances WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ).map_err(|e| LauncherError::database(e.to_string()))?;
        Ok(count > 0)
    }
}
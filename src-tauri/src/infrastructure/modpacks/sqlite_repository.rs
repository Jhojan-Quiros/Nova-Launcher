use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::domain::modpacks::{
    InstalledModpack, InstalledModpackRepository, ModpackError,
    ModpackStatus, ModpackUpdateHistoryRecord, ModpackVersionManifest,
};
use crate::infrastructure::persistence::DbConn;

pub struct SqliteInstalledModpackRepository {
    pool: DbConn,
}

impl SqliteInstalledModpackRepository {
    pub fn new(pool: DbConn) -> Self {
        Self { pool }
    }

    fn map_row_to_installed(row: &Row) -> rusqlite::Result<InstalledModpack> {
        let installed_at_str: String = row.get(8)?;
        let updated_at_str: String = row.get(9)?;
        let last_verified_at_str: Option<String> = row.get(10)?;
        let status_str: String = row.get(11)?;
        let strict_mode_int: i32 = row.get(12)?;

        Ok(InstalledModpack {
            instance_id: row.get(0)?,
            modpack_id: row.get(1)?,
            name: row.get(2)?,
            installed_version: row.get(3)?,
            latest_known_version: row.get(4)?,
            manifest_url: row.get(5)?,
            icon_url: row.get(6)?,
            banner_url: row.get(7)?,
            installed_at: DateTime::parse_from_rfc3339(&installed_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_verified_at: last_verified_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.with_timezone(&Utc))
                    .ok()
            }),
            status: ModpackStatus::from(status_str.as_str()),
            strict_mode: strict_mode_int != 0,
        })
    }

    fn map_row_to_history(row: &Row) -> rusqlite::Result<ModpackUpdateHistoryRecord> {
        let started_at_str: String = row.get(7)?;
        let completed_at_str: Option<String> = row.get(8)?;

        Ok(ModpackUpdateHistoryRecord {
            id: row.get(0)?,
            instance_id: row.get(1)?,
            pack_id: row.get(2)?,
            from_version: row.get(3)?,
            to_version: row.get(4)?,
            status: row.get(5)?,
            download_size: row.get::<_, i64>(6)? as u64,
            started_at: DateTime::parse_from_rfc3339(&started_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            completed_at: completed_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.with_timezone(&Utc))
                    .ok()
            }),
            error_code: row.get(9)?,
        })
    }
}

#[async_trait]
impl InstalledModpackRepository for SqliteInstalledModpackRepository {
    async fn save(&self, installed: &InstalledModpack) -> Result<(), ModpackError> {
        let conn = self.pool.lock().await;
        conn.execute(
            "INSERT INTO installed_modpacks (
                instance_id, modpack_id, name, installed_version, latest_known_version,
                manifest_url, icon_url, banner_url, installed_at, updated_at,
                last_verified_at, status, strict_mode
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(instance_id) DO UPDATE SET
                modpack_id = excluded.modpack_id,
                name = excluded.name,
                installed_version = excluded.installed_version,
                latest_known_version = excluded.latest_known_version,
                manifest_url = excluded.manifest_url,
                icon_url = excluded.icon_url,
                banner_url = excluded.banner_url,
                updated_at = excluded.updated_at,
                last_verified_at = excluded.last_verified_at,
                status = excluded.status,
                strict_mode = excluded.strict_mode",
            params![
                installed.instance_id,
                installed.modpack_id,
                installed.name,
                installed.installed_version,
                installed.latest_known_version,
                installed.manifest_url,
                installed.icon_url,
                installed.banner_url,
                installed.installed_at.to_rfc3339(),
                installed.updated_at.to_rfc3339(),
                installed.last_verified_at.map(|d| d.to_rfc3339()),
                installed.status.to_string(),
                if installed.strict_mode { 1 } else { 0 }
            ],
        ).map_err(|e| ModpackError::DatabaseError(format!("Failed to save installed modpack: {}", e)))?;

        Ok(())
    }

    async fn get_by_instance_id(&self, instance_id: &str) -> Result<Option<InstalledModpack>, ModpackError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT instance_id, modpack_id, name, installed_version, latest_known_version,
                    manifest_url, icon_url, banner_url, installed_at, updated_at,
                    last_verified_at, status, strict_mode
             FROM installed_modpacks WHERE instance_id = ?1",
        ).map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let res = stmt.query_row(params![instance_id], Self::map_row_to_installed)
            .optional()
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        Ok(res)
    }

    async fn get_by_modpack_id(&self, modpack_id: &str) -> Result<Option<InstalledModpack>, ModpackError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT instance_id, modpack_id, name, installed_version, latest_known_version,
                    manifest_url, icon_url, banner_url, installed_at, updated_at,
                    last_verified_at, status, strict_mode
             FROM installed_modpacks WHERE modpack_id = ?1 LIMIT 1",
        ).map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let res = stmt.query_row(params![modpack_id], Self::map_row_to_installed)
            .optional()
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        Ok(res)
    }

    async fn list_all(&self) -> Result<Vec<InstalledModpack>, ModpackError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT instance_id, modpack_id, name, installed_version, latest_known_version,
                    manifest_url, icon_url, banner_url, installed_at, updated_at,
                    last_verified_at, status, strict_mode
             FROM installed_modpacks ORDER BY updated_at DESC",
        ).map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map([], Self::map_row_to_installed)
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ModpackError::DatabaseError(e.to_string()))?);
        }

        Ok(list)
    }

    async fn delete(&self, instance_id: &str) -> Result<(), ModpackError> {
        let conn = self.pool.lock().await;
        conn.execute("DELETE FROM installed_modpacks WHERE instance_id = ?1", params![instance_id])
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn save_version_cache(&self, version: &ModpackVersionManifest) -> Result<(), ModpackError> {
        let conn = self.pool.lock().await;
        let manifest_json = serde_json::to_string(version)
            .map_err(|e| ModpackError::DatabaseError(format!("Failed to serialize version manifest: {}", e)))?;
        let changelog_json = serde_json::to_string(&version.changelog).unwrap_or_else(|_| "[]".to_string());
        let stats_json = serde_json::to_string(&version.stats).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO modpack_versions (
                pack_id, version, minecraft_version, loader, loader_version,
                changelog_json, stats_json, manifest_json, cached_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(pack_id, version) DO UPDATE SET
                manifest_json = excluded.manifest_json,
                cached_at = excluded.cached_at",
            params![
                version.pack_id,
                version.version,
                version.minecraft_version,
                version.loader,
                version.loader_version,
                changelog_json,
                stats_json,
                manifest_json,
                Utc::now().to_rfc3339()
            ],
        ).map_err(|e| ModpackError::DatabaseError(format!("Failed to cache modpack version: {}", e)))?;

        Ok(())
    }

    async fn get_cached_version(&self, pack_id: &str, version: &str) -> Result<Option<ModpackVersionManifest>, ModpackError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT manifest_json FROM modpack_versions WHERE pack_id = ?1 AND version = ?2",
        ).map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let json_str: Option<String> = stmt.query_row(params![pack_id, version], |r| r.get(0))
            .optional()
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        match json_str {
            Some(s) => serde_json::from_str::<ModpackVersionManifest>(&s)
                .map(Some)
                .map_err(|e| ModpackError::DatabaseError(format!("Corrupted cached version manifest: {}", e))),
            None => Ok(None),
        }
    }

    async fn record_history(&self, record: &ModpackUpdateHistoryRecord) -> Result<(), ModpackError> {
        let conn = self.pool.lock().await;
        conn.execute(
            "INSERT INTO modpack_update_history (
                id, instance_id, pack_id, from_version, to_version, status,
                download_size, started_at, completed_at, error_code
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.id,
                record.instance_id,
                record.pack_id,
                record.from_version,
                record.to_version,
                record.status,
                record.download_size as i64,
                record.started_at.to_rfc3339(),
                record.completed_at.map(|d| d.to_rfc3339()),
                record.error_code
            ],
        ).map_err(|e| ModpackError::DatabaseError(format!("Failed to record update history: {}", e)))?;

        Ok(())
    }

    async fn get_history(&self, instance_id: &str) -> Result<Vec<ModpackUpdateHistoryRecord>, ModpackError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, instance_id, pack_id, from_version, to_version, status,
                    download_size, started_at, completed_at, error_code
             FROM modpack_update_history WHERE instance_id = ?1 ORDER BY started_at DESC",
        ).map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map(params![instance_id], Self::map_row_to_history)
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ModpackError::DatabaseError(e.to_string()))?);
        }

        Ok(list)
    }
}

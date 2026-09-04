use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use crate::domain::entities::OfflineProfile;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::OfflineProfileRepository;
use super::database::DbConn;

pub struct SqliteOfflineProfileRepository {
    pool: DbConn,
}

impl SqliteOfflineProfileRepository {
    pub fn new(pool: DbConn) -> Self {
        Self { pool }
    }

    fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<OfflineProfile> {
        Ok(OfflineProfile {
            username: row.get(0)?,
            generated_local_uuid: row.get(1)?,
            created_at: row.get(2)?,
            last_used_at: row.get(3)?,
        })
    }
}

#[async_trait]
impl OfflineProfileRepository for SqliteOfflineProfileRepository {
    async fn get_active(&self) -> Result<OfflineProfile, LauncherError> {
        let conn = self.pool.lock().await;

        // 1. Try to find the currently active profile
        let mut stmt = conn
            .prepare("SELECT username, generated_local_uuid, created_at, last_used_at FROM offline_profiles WHERE is_active = 1 LIMIT 1")
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let active: Option<OfflineProfile> = stmt
            .query_row([], Self::row_to_profile)
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))?;

        if let Some(profile) = active {
            return Ok(profile);
        }

        // 2. Fallback: If any profile exists, make the latest one active
        let mut fallback_stmt = conn
            .prepare("SELECT username, generated_local_uuid, created_at, last_used_at FROM offline_profiles ORDER BY last_used_at DESC LIMIT 1")
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let fallback: Option<OfflineProfile> = fallback_stmt
            .query_row([], Self::row_to_profile)
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))?;

        if let Some(profile) = fallback {
            conn.execute(
                "UPDATE offline_profiles SET is_active = 1 WHERE username = ?1",
                params![profile.username],
            ).map_err(|e| LauncherError::database(e.to_string()))?;
            return Ok(profile);
        }

        // 3. If database is completely empty, seed default "NovaPlayer"
        let default_profile = OfflineProfile::new("NovaPlayer");
        conn.execute(
            "INSERT INTO offline_profiles (username, generated_local_uuid, created_at, last_used_at, is_active)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![
                default_profile.username,
                default_profile.generated_local_uuid,
                default_profile.created_at,
                default_profile.last_used_at,
            ],
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        Ok(default_profile)
    }

    async fn set_active(&self, username: &str) -> Result<OfflineProfile, LauncherError> {
        let conn = self.pool.lock().await;

        let mut stmt = conn
            .prepare("SELECT username, generated_local_uuid, created_at, last_used_at FROM offline_profiles WHERE username = ?1")
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let profile = stmt
            .query_row(params![username], Self::row_to_profile)
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))?
            .ok_or_else(|| LauncherError::not_found(format!("Offline profile '{}' not found", username)))?;

        let now = Utc::now().to_rfc3339();
        conn.execute("UPDATE offline_profiles SET is_active = 0", [])
            .map_err(|e| LauncherError::database(e.to_string()))?;

        conn.execute(
            "UPDATE offline_profiles SET is_active = 1, last_used_at = ?1 WHERE username = ?2",
            params![now, username],
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        Ok(OfflineProfile {
            username: profile.username,
            generated_local_uuid: profile.generated_local_uuid,
            created_at: profile.created_at,
            last_used_at: now,
        })
    }

    async fn list(&self) -> Result<Vec<OfflineProfile>, LauncherError> {
        // Ensure default profile is seeded if empty
        drop(self.get_active().await?);

        let conn = self.pool.lock().await;
        let mut stmt = conn
            .prepare("SELECT username, generated_local_uuid, created_at, last_used_at FROM offline_profiles ORDER BY is_active DESC, last_used_at DESC")
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let rows = stmt
            .query_map([], Self::row_to_profile)
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let mut profiles = Vec::new();
        for r in rows {
            profiles.push(r.map_err(|e| LauncherError::database(e.to_string()))?);
        }

        Ok(profiles)
    }

    async fn save(&self, profile: &OfflineProfile, is_active: bool) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;

        if is_active {
            conn.execute("UPDATE offline_profiles SET is_active = 0", [])
                .map_err(|e| LauncherError::database(e.to_string()))?;
        }

        conn.execute(
            "INSERT INTO offline_profiles (username, generated_local_uuid, created_at, last_used_at, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(username) DO UPDATE SET
                generated_local_uuid = excluded.generated_local_uuid,
                last_used_at = excluded.last_used_at,
                is_active = CASE WHEN ?5 = 1 THEN 1 ELSE offline_profiles.is_active END",
            params![
                profile.username,
                profile.generated_local_uuid,
                profile.created_at,
                profile.last_used_at,
                if is_active { 1 } else { 0 },
            ],
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, username: &str) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM offline_profiles", [], |r| r.get(0))
            .map_err(|e| LauncherError::database(e.to_string()))?;

        if count <= 1 {
            return Err(LauncherError::validation("Cannot delete the only remaining offline profile"));
        }

        let was_active: bool = conn
            .query_row(
                "SELECT is_active FROM offline_profiles WHERE username = ?1",
                params![username],
                |r| r.get::<_, i64>(0).map(|v| v == 1),
            )
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))?
            .unwrap_or(false);

        conn.execute("DELETE FROM offline_profiles WHERE username = ?1", params![username])
            .map_err(|e| LauncherError::database(e.to_string()))?;

        if was_active {
            conn.execute(
                "UPDATE offline_profiles SET is_active = 1 WHERE username = (
                    SELECT username FROM offline_profiles ORDER BY last_used_at DESC LIMIT 1
                )",
                [],
            ).map_err(|e| LauncherError::database(e.to_string()))?;
        }

        Ok(())
    }

    async fn touch_last_used(&self, username: &str) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE offline_profiles SET last_used_at = ?1 WHERE username = ?2",
            params![now, username],
        ).map_err(|e| LauncherError::database(e.to_string()))?;
        Ok(())
    }
}

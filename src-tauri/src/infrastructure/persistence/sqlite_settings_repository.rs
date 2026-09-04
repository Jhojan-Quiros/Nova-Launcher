use async_trait::async_trait;
use rusqlite::{params, OptionalExtension};
use crate::domain::entities::AppSettings;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::SettingsRepository;
use super::database::DbConn;

pub struct SqliteSettingsRepository {
    pool: DbConn,
}

impl SqliteSettingsRepository {
    pub fn new(pool: DbConn) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn get(&self) -> Result<AppSettings, LauncherError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'app_config'")
            .map_err(|e| LauncherError::database(e.to_string()))?;

        let val: Option<String> = stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))?;

        match val {
            Some(json_str) => serde_json::from_str(&json_str)
                .map_err(|e| LauncherError::database(format!("Corrupted settings JSON: {}", e))),
            None => Ok(AppSettings::default()),
        }
    }

    async fn save(&self, settings: &AppSettings) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;
        let json_str = serde_json::to_string(settings)
            .map_err(|e| LauncherError::database(format!("Failed to serialize settings: {}", e)))?;

        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('app_config', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json_str],
        ).map_err(|e| LauncherError::database(format!("Failed to save settings: {}", e)))?;

        Ok(())
    }
}
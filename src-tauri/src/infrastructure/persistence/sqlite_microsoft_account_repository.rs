use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use crate::domain::entities::StoredMicrosoftAccount;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::MicrosoftAccountRepository;
use super::database::DbConn;

const ROW_ID: &str = "default";

pub struct SqliteMicrosoftAccountRepository {
    pool: DbConn,
}

impl SqliteMicrosoftAccountRepository {
    pub fn new(pool: DbConn) -> Self {
        Self { pool }
    }

    fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<StoredMicrosoftAccount> {
        let expires_str: String = row.get(3)?;
        let created_str: String = row.get(5)?;
        let updated_str: String = row.get(6)?;
        Ok(StoredMicrosoftAccount {
            username: row.get(0)?,
            uuid: row.get(1)?,
            minecraft_access_token: row.get(2)?,
            minecraft_token_expires_at: DateTime::parse_from_rfc3339(&expires_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            ms_refresh_token: row.get(4)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

#[async_trait]
impl MicrosoftAccountRepository for SqliteMicrosoftAccountRepository {
    async fn get(&self) -> Result<Option<StoredMicrosoftAccount>, LauncherError> {
        let conn = self.pool.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT username, uuid, minecraft_access_token, minecraft_token_expires_at, ms_refresh_token, created_at, updated_at
                 FROM microsoft_account WHERE id = ?1",
            )
            .map_err(|e| LauncherError::database(e.to_string()))?;

        stmt.query_row(params![ROW_ID], Self::row_to_account)
            .optional()
            .map_err(|e| LauncherError::database(e.to_string()))
    }

    async fn save(&self, account: &StoredMicrosoftAccount) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;
        conn.execute(
            "INSERT INTO microsoft_account (id, username, uuid, minecraft_access_token, minecraft_token_expires_at, ms_refresh_token, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                username = excluded.username,
                uuid = excluded.uuid,
                minecraft_access_token = excluded.minecraft_access_token,
                minecraft_token_expires_at = excluded.minecraft_token_expires_at,
                ms_refresh_token = excluded.ms_refresh_token,
                updated_at = excluded.updated_at",
            params![
                ROW_ID,
                account.username,
                account.uuid,
                account.minecraft_access_token,
                account.minecraft_token_expires_at.to_rfc3339(),
                account.ms_refresh_token,
                account.created_at.to_rfc3339(),
                account.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| LauncherError::database(e.to_string()))?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), LauncherError> {
        let conn = self.pool.lock().await;
        conn.execute("DELETE FROM microsoft_account WHERE id = ?1", params![ROW_ID])
            .map_err(|e| LauncherError::database(e.to_string()))?;
        Ok(())
    }
}

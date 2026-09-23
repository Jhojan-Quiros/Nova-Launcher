use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A signed-in Microsoft/Xbox account, persisted so the user doesn't have to
/// re-authenticate through the browser every launch. Tokens are stored as-is in
/// the local SQLite database, with no additional encryption layer - consistent
/// with how the rest of the launcher's local data is stored today.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredMicrosoftAccount {
    pub username: String,
    pub uuid: String,
    pub minecraft_access_token: String,
    pub minecraft_token_expires_at: DateTime<Utc>,
    pub ms_refresh_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::domain::entities::StoredMicrosoftAccount;

/// Public-facing view of a stored Microsoft account - deliberately omits the
/// access/refresh tokens, the frontend never needs to see those.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftAccountInfoDto {
    pub username: String,
    pub uuid: String,
    pub token_expires_at: DateTime<Utc>,
}

impl From<StoredMicrosoftAccount> for MicrosoftAccountInfoDto {
    fn from(value: StoredMicrosoftAccount) -> Self {
        Self {
            username: value.username,
            uuid: value.uuid,
            token_expires_at: value.minecraft_token_expires_at,
        }
    }
}

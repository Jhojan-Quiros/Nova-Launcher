use chrono::Utc;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineProfile {
    pub username: String,
    pub generated_local_uuid: String,
    pub created_at: String,
    pub last_used_at: String,
}

impl OfflineProfile {
    pub fn new(username: &str) -> Self {
        let clean_username = username.trim();
        let now = Utc::now().to_rfc3339();
        Self {
            username: clean_username.to_string(),
            generated_local_uuid: Self::generate_deterministic_uuid(clean_username),
            created_at: now.clone(),
            last_used_at: now,
        }
    }

    /// Official Minecraft Offline Player UUID Generation
    /// Matches Java's: UUID.nameUUIDFromBytes(("OfflinePlayer:" + username).getBytes(StandardCharsets.UTF_8))
    pub fn generate_deterministic_uuid(username: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(format!("OfflinePlayer:{}", username.trim()).as_bytes());
        let mut bytes = hasher.finalize();

        // Set version to 3 (MD5-based UUID)
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        // Set variant to IETF / RFC 4122
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        let raw_bytes: [u8; 16] = bytes.into();
        Uuid::from_bytes(raw_bytes).to_string()
    }
}

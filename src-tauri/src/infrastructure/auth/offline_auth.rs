use async_trait::async_trait;
use uuid::Uuid;
use crate::application::ports::AuthenticationProviderPort;
use crate::domain::entities::{AccountType, MinecraftAccount};
use crate::domain::errors::LauncherError;

pub struct DevOfflineAuthenticationProvider {
    username: String,
}

impl DevOfflineAuthenticationProvider {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }
}

impl Default for DevOfflineAuthenticationProvider {
    fn default() -> Self {
        Self {
            username: "NovaPlayer".to_string(),
        }
    }
}

#[async_trait]
impl AuthenticationProviderPort for DevOfflineAuthenticationProvider {
    async fn get_active_account(&self) -> Result<MinecraftAccount, LauncherError> {
        let uuid = Uuid::new_v4().to_string();
        Ok(MinecraftAccount {
            id: format!("offline-{}", self.username.to_lowercase()),
            username: self.username.clone(),
            uuid,
            access_token: "0".to_string(),
            account_type: AccountType::Offline,
        })
    }
}

// Architectural placeholder for future Microsoft OAuth2 + Xbox Live Auth
pub struct MicrosoftAuthenticationProvider;
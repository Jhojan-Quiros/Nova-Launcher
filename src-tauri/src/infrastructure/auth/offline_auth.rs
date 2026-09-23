use std::sync::Arc;
use async_trait::async_trait;
use crate::application::ports::AuthenticationProviderPort;
use crate::domain::entities::{AccountType, MinecraftAccount, OfflineProfile};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::OfflineProfileRepository;

pub struct OfflineAuthenticationProvider {
    profile_repo: Arc<dyn OfflineProfileRepository>,
}

impl OfflineAuthenticationProvider {
    pub fn new(profile_repo: Arc<dyn OfflineProfileRepository>) -> Self {
        Self { profile_repo }
    }
}

#[async_trait]
impl AuthenticationProviderPort for OfflineAuthenticationProvider {
    async fn get_active_account(&self) -> Result<MinecraftAccount, LauncherError> {
        let profile = self.profile_repo.get_active().await?;
        let _ = self.profile_repo.touch_last_used(&profile.username).await;

        Ok(MinecraftAccount {
            id: format!("offline-{}", profile.username.to_lowercase()),
            username: profile.username,
            uuid: profile.generated_local_uuid,
            access_token: "0".to_string(),
            account_type: AccountType::Offline,
        })
    }
}

/// Backward compatibility provider for tests and standalone instances
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
        let uuid = OfflineProfile::generate_deterministic_uuid(&self.username);
        Ok(MinecraftAccount {
            id: format!("offline-{}", self.username.to_lowercase()),
            username: self.username.clone(),
            uuid,
            access_token: "0".to_string(),
            account_type: AccountType::Offline,
        })
    }
}
use async_trait::async_trait;
use crate::domain::entities::MinecraftAccount;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait AuthenticationProviderPort: Send + Sync {
    async fn get_active_account(&self) -> Result<MinecraftAccount, LauncherError>;
}
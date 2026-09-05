use async_trait::async_trait;
use crate::domain::entities::StoredMicrosoftAccount;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait MicrosoftAccountRepository: Send + Sync {
    async fn get(&self) -> Result<Option<StoredMicrosoftAccount>, LauncherError>;
    async fn save(&self, account: &StoredMicrosoftAccount) -> Result<(), LauncherError>;
    async fn clear(&self) -> Result<(), LauncherError>;
}

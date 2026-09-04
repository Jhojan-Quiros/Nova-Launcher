use async_trait::async_trait;
use crate::domain::entities::OfflineProfile;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait OfflineProfileRepository: Send + Sync {
    async fn get_active(&self) -> Result<OfflineProfile, LauncherError>;
    async fn set_active(&self, username: &str) -> Result<OfflineProfile, LauncherError>;
    async fn list(&self) -> Result<Vec<OfflineProfile>, LauncherError>;
    async fn save(&self, profile: &OfflineProfile, is_active: bool) -> Result<(), LauncherError>;
    async fn delete(&self, username: &str) -> Result<(), LauncherError>;
    async fn touch_last_used(&self, username: &str) -> Result<(), LauncherError>;
}

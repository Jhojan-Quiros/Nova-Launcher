use async_trait::async_trait;
use crate::domain::entities::settings::AppSettings;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get(&self) -> Result<AppSettings, LauncherError>;
    async fn save(&self, settings: &AppSettings) -> Result<(), LauncherError>;
}

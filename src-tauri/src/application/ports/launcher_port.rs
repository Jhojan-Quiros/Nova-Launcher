use async_trait::async_trait;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait MinecraftLauncherPort: Send + Sync {
    async fn launch(&self, instance: &Instance) -> Result<(), LauncherError>;
}
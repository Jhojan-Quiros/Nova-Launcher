use async_trait::async_trait;
use crate::domain::entities::{Instance, ModLoader};
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait ModLoaderInstallerPort: Send + Sync {
    fn loader_type(&self) -> ModLoader;
    async fn install(&self, instance: &Instance) -> Result<(), LauncherError>;
}
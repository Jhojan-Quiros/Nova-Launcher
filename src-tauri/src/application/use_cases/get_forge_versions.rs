use std::sync::Arc;
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::forge_installer::{ForgeInstaller, ForgeVersionOption};

pub struct GetForgeVersionsUseCase {
    forge_installer: Arc<ForgeInstaller>,
}

impl GetForgeVersionsUseCase {
    pub fn new(forge_installer: Arc<ForgeInstaller>) -> Self {
        Self { forge_installer }
    }

    pub async fn execute(&self, mc_version: &str) -> Result<Vec<ForgeVersionOption>, LauncherError> {
        self.forge_installer.fetch_available_versions(mc_version).await
    }
}

use std::sync::Arc;
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::installer::MinecraftInstaller;

pub struct InstallInstanceUseCase {
    installer: Arc<MinecraftInstaller>,
}

impl InstallInstanceUseCase {
    pub fn new(installer: Arc<MinecraftInstaller>) -> Self {
        Self { installer }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<(), LauncherError> {
        self.installer.install(instance_id).await
    }
}
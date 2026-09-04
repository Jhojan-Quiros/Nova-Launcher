use std::sync::Arc;
use crate::application::ports::MinecraftLauncherPort;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;

pub struct LaunchInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
    launcher: Arc<dyn MinecraftLauncherPort>,
}

impl LaunchInstanceUseCase {
    pub fn new(instance_repo: Arc<dyn InstanceRepository>, launcher: Arc<dyn MinecraftLauncherPort>) -> Self {
        Self {
            instance_repo,
            launcher,
        }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<(), LauncherError> {
        let instance = self.instance_repo.find_by_id(instance_id).await?
            .ok_or_else(|| LauncherError::not_found(format!("Instance with id {} not found", instance_id)))?;

        self.launcher.launch(&instance).await
    }
}
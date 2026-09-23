use std::collections::HashMap;
use std::sync::Arc;
use crate::application::ports::ModLoaderInstallerPort;
use crate::domain::entities::{InstanceStatus, ModLoader};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;

pub struct InstallInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
    installers: HashMap<ModLoader, Arc<dyn ModLoaderInstallerPort>>,
}

impl InstallInstanceUseCase {
    pub fn new(
        instance_repo: Arc<dyn InstanceRepository>,
        installers: HashMap<ModLoader, Arc<dyn ModLoaderInstallerPort>>,
    ) -> Self {
        Self {
            instance_repo,
            installers,
        }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<(), LauncherError> {
        let mut instance = self.instance_repo.find_by_id(instance_id).await?
            .ok_or_else(|| LauncherError::not_found(format!("Instance {} not found", instance_id)))?;

        let installer = self.installers.get(&instance.loader).cloned().ok_or_else(|| {
            LauncherError::validation(format!(
                "The '{}' mod loader is not supported yet",
                instance.loader
            ))
        })?;

        instance.set_status(InstanceStatus::Installing);
        self.instance_repo.save(&instance).await?;

        match installer.install(&instance).await {
            Ok(()) => {
                instance.set_status(InstanceStatus::Ready);
                let _ = self.instance_repo.save(&instance).await;
                Ok(())
            }
            Err(e) => {
                instance.set_status(InstanceStatus::Error(e.to_string()));
                let _ = self.instance_repo.save(&instance).await;
                Err(e)
            }
        }
    }
}

use std::sync::Arc;
use std::fs;
use crate::domain::entities::InstanceStatus;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;
use crate::shared::config::LauncherPaths;

pub struct DeleteInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
    paths: LauncherPaths,
}

impl DeleteInstanceUseCase {
    pub fn new(instance_repo: Arc<dyn InstanceRepository>, paths: LauncherPaths) -> Self {
        Self { instance_repo, paths }
    }

    pub async fn execute(&self, id: &str) -> Result<bool, LauncherError> {
        let instance = self.instance_repo.find_by_id(id).await?;
        let instance = match instance {
            Some(i) => i,
            None => return Ok(false),
        };

        if instance.status == InstanceStatus::Running {
            return Err(LauncherError::validation("Cannot delete an instance that is currently running"));
        }

        // Delete files from disk
        let inst_dir = self.paths.instance_dir(id);
        if inst_dir.exists() {
            fs::remove_dir_all(&inst_dir).map_err(|e| {
                LauncherError::filesystem(format!("Failed to delete instance files: {}", e))
            })?;
        }

        self.instance_repo.delete(id).await
    }
}
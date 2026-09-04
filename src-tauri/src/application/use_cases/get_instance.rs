use std::sync::Arc;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;

pub struct GetInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
}

impl GetInstanceUseCase {
    pub fn new(instance_repo: Arc<dyn InstanceRepository>) -> Self {
        Self { instance_repo }
    }

    pub async fn execute(&self, id: &str) -> Result<Instance, LauncherError> {
        self.instance_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| LauncherError::not_found(format!("Instance with id '{}' was not found", id)))
    }
}
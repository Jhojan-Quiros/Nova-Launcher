use std::sync::Arc;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;

pub struct ListInstancesUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
}

impl ListInstancesUseCase {
    pub fn new(instance_repo: Arc<dyn InstanceRepository>) -> Self {
        Self { instance_repo }
    }

    pub async fn execute(&self) -> Result<Vec<Instance>, LauncherError> {
        self.instance_repo.find_all().await
    }
}
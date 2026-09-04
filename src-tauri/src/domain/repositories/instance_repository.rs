use async_trait::async_trait;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait InstanceRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Instance>, LauncherError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Instance>, LauncherError>;
    async fn save(&self, instance: &Instance) -> Result<(), LauncherError>;
    async fn delete(&self, id: &str) -> Result<bool, LauncherError>;
    async fn exists(&self, id: &str) -> Result<bool, LauncherError>;
}

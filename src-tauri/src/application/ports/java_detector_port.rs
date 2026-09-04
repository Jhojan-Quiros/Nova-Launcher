use async_trait::async_trait;
use crate::domain::entities::JavaRuntime;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait JavaDetectorPort: Send + Sync {
    async fn detect_installed_runtimes(&self) -> Result<Vec<JavaRuntime>, LauncherError>;
    async fn probe_executable(&self, path: &str) -> Result<JavaRuntime, LauncherError>;
}
use std::sync::Arc;
use crate::application::ports::JavaDetectorPort;
use crate::domain::entities::JavaRuntime;
use crate::domain::errors::LauncherError;

pub struct DetectJavaUseCase {
    detector: Arc<dyn JavaDetectorPort>,
}

impl DetectJavaUseCase {
    pub fn new(detector: Arc<dyn JavaDetectorPort>) -> Self {
        Self { detector }
    }

    pub async fn execute(&self) -> Result<Vec<JavaRuntime>, LauncherError> {
        self.detector.detect_installed_runtimes().await
    }

    pub async fn probe(&self, path: &str) -> Result<JavaRuntime, LauncherError> {
        self.detector.probe_executable(path).await
    }
}
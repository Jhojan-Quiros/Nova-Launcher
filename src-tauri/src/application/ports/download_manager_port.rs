use async_trait::async_trait;
use std::path::PathBuf;
use crate::domain::errors::LauncherError;

#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub url: String,
    pub destination: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[async_trait]
pub trait DownloadManagerPort: Send + Sync {
    async fn download_batch(
        &self,
        instance_id: Option<String>,
        items: Vec<DownloadItem>,
    ) -> Result<(), LauncherError>;
}
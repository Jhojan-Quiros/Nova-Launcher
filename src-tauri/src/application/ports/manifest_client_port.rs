use async_trait::async_trait;
use crate::domain::entities::minecraft_version::MinecraftVersion;
use crate::domain::errors::LauncherError;

#[async_trait]
pub trait ManifestClientPort: Send + Sync {
    async fn fetch_versions(&self) -> Result<(String, String, Vec<MinecraftVersion>), LauncherError>;
    async fn fetch_version_json(&self, version_id: &str, url: &str) -> Result<serde_json::Value, LauncherError>;
}
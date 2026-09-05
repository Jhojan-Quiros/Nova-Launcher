use async_trait::async_trait;
use super::entities::*;
use super::errors::ModpackError;

#[async_trait]
pub trait RemoteModpackRepository: Send + Sync {
    async fn get_catalog(&self, catalog_url: &str, force_refresh: bool) -> Result<ModpackCatalog, ModpackError>;
    async fn get_main_manifest(&self, manifest_url: &str, force_refresh: bool) -> Result<ModpackMainManifest, ModpackError>;
    async fn get_version_manifest(&self, version_url: &str) -> Result<ModpackVersionManifest, ModpackError>;
}

#[async_trait]
pub trait InstalledModpackRepository: Send + Sync {
    async fn save(&self, installed: &InstalledModpack) -> Result<(), ModpackError>;
    async fn get_by_instance_id(&self, instance_id: &str) -> Result<Option<InstalledModpack>, ModpackError>;
    async fn get_by_modpack_id(&self, modpack_id: &str) -> Result<Option<InstalledModpack>, ModpackError>;
    async fn list_all(&self) -> Result<Vec<InstalledModpack>, ModpackError>;
    async fn delete(&self, instance_id: &str) -> Result<(), ModpackError>;

    // Version caching
    async fn save_version_cache(&self, version: &ModpackVersionManifest) -> Result<(), ModpackError>;
    async fn get_cached_version(&self, pack_id: &str, version: &str) -> Result<Option<ModpackVersionManifest>, ModpackError>;

    // History
    async fn record_history(&self, record: &ModpackUpdateHistoryRecord) -> Result<(), ModpackError>;
    async fn get_history(&self, instance_id: &str) -> Result<Vec<ModpackUpdateHistoryRecord>, ModpackError>;
}

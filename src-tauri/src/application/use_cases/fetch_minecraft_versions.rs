use std::sync::Arc;
use crate::application::dto::{VersionFilterDto, VersionListResponseDto};
use crate::application::ports::ManifestClientPort;
use crate::domain::entities::minecraft_version::VersionType;
use crate::domain::errors::LauncherError;

pub struct FetchMinecraftVersionsUseCase {
    manifest_client: Arc<dyn ManifestClientPort>,
}

impl FetchMinecraftVersionsUseCase {
    pub fn new(manifest_client: Arc<dyn ManifestClientPort>) -> Self {
        Self { manifest_client }
    }

    pub async fn execute(&self, filter: VersionFilterDto) -> Result<VersionListResponseDto, LauncherError> {
        let (latest_release, latest_snapshot, versions) = self.manifest_client.fetch_versions().await?;

        let filtered = versions
            .into_iter()
            .filter(|v| match v.version_type {
                VersionType::Release => filter.show_releases,
                VersionType::Snapshot => filter.show_snapshots,
                VersionType::OldBeta => filter.show_old_beta,
                VersionType::OldAlpha => filter.show_old_alpha,
            })
            .collect();

        Ok(VersionListResponseDto {
            latest_release,
            latest_snapshot,
            versions: filtered,
        })
    }
}
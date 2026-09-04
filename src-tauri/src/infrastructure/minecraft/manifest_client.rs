use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use crate::application::ports::ManifestClientPort;
use crate::domain::entities::minecraft_version::{MinecraftVersion, VersionType};
use crate::domain::errors::LauncherError;
use crate::shared::config::LauncherPaths;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Deserialize)]
struct MojangManifestLatest {
    release: String,
    snapshot: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MojangVersionEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    sha1: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MojangManifest {
    latest: MojangManifestLatest,
    versions: Vec<MojangVersionEntry>,
}

pub struct MinecraftVersionManifestClient {
    client: Client,
    paths: LauncherPaths,
}

impl MinecraftVersionManifestClient {
    pub fn new(paths: LauncherPaths) -> Self {
        Self {
            client: Client::builder()
                .user_agent("NovaLauncher/1.0 (contact@novalauncher.local)")
                .build()
                .unwrap_or_default(),
            paths,
        }
    }

    fn cached_manifest_path(&self) -> PathBuf {
        self.paths.versions_dir().join("version_manifest_v2.json")
    }

    fn parse_manifest(&self, json_content: &str) -> Result<(String, String, Vec<MinecraftVersion>), LauncherError> {
        let manifest: MojangManifest = serde_json::from_str(json_content)
            .map_err(|e| LauncherError::minecraft(format!("Failed to parse Mojang version manifest: {}", e), None))?;

        let mut versions = Vec::with_capacity(manifest.versions.len());
        for v in manifest.versions {
            let v_type = match v.version_type.as_str() {
                "release" => VersionType::Release,
                "snapshot" => VersionType::Snapshot,
                "old_beta" => VersionType::OldBeta,
                "old_alpha" => VersionType::OldAlpha,
                _ => continue,
            };

            let release_time = DateTime::parse_from_rfc3339(&v.release_time)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            versions.push(MinecraftVersion {
                id: v.id,
                version_type: v_type,
                url: v.url,
                release_time,
                sha1: v.sha1,
            });
        }

        Ok((manifest.latest.release, manifest.latest.snapshot, versions))
    }
}

#[async_trait]
impl ManifestClientPort for MinecraftVersionManifestClient {
    async fn fetch_versions(&self) -> Result<(String, String, Vec<MinecraftVersion>), LauncherError> {
        let cache_file = self.cached_manifest_path();

        // Attempt network fetch
        match self.client.get(MANIFEST_URL).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(text) = resp.text().await {
                        // Cache locally
                        let _ = fs::create_dir_all(self.paths.versions_dir());
                        let _ = fs::write(&cache_file, &text);
                        return self.parse_manifest(&text);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch version manifest from Mojang: {}. Checking cache...", e);
            }
        }

        // Fallback to local cached copy
        if cache_file.exists() {
            let cached = fs::read_to_string(&cache_file).map_err(|e| {
                LauncherError::filesystem(format!("Failed to read cached version manifest: {}", e))
            })?;
            return self.parse_manifest(&cached);
        }

        Err(LauncherError::network("Unable to fetch Minecraft versions and no cached manifest found"))
    }

    async fn fetch_version_json(&self, version_id: &str, url: &str) -> Result<serde_json::Value, LauncherError> {
        let version_dir = self.paths.versions_dir().join(version_id);
        let version_json_file = version_dir.join(format!("{}.json", version_id));

        if version_json_file.exists() {
            let content = fs::read_to_string(&version_json_file).map_err(|e| {
                LauncherError::filesystem(format!("Failed to read version JSON: {}", e))
            })?;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                return Ok(val);
            }
        }

        let resp = self.client.get(url).send().await.map_err(|e| {
            LauncherError::network(format!("Failed to download version details for {}: {}", version_id, e))
        })?;

        if !resp.status().is_success() {
            return Err(LauncherError::network(format!("Mojang server returned status {} for version {}", resp.status(), version_id)));
        }

        let text = resp.text().await.map_err(|e| {
            LauncherError::network(format!("Failed to read version json body: {}", e))
        })?;

        fs::create_dir_all(&version_dir).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create version directory: {}", e))
        })?;

        let _ = fs::write(&version_json_file, &text);

        let val: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            LauncherError::minecraft(format!("Invalid version JSON for {}: {}", version_id, e), None)
        })?;

        Ok(val)
    }
}
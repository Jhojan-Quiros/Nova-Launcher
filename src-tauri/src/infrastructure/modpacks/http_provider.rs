use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::domain::modpacks::{
    ManifestSchemaValidator, ModpackCatalog, ModpackError, ModpackMainManifest,
    ModpackVersionManifest, RemoteModpackRepository,
};

struct CacheEntry<T> {
    data: T,
    cached_at: Instant,
}

pub struct HttpModpackProvider {
    client: Client,
    catalog_cache: Arc<RwLock<HashMap<String, CacheEntry<ModpackCatalog>>>>,
    manifest_cache: Arc<RwLock<HashMap<String, CacheEntry<ModpackMainManifest>>>>,
    cache_ttl: Duration,
}

impl HttpModpackProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("NovaLauncher/1.0")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            catalog_cache: Arc::new(RwLock::new(HashMap::new())),
            manifest_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes cache TTL
        }
    }

    async fn fetch_json_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, ModpackError> {
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            attempts += 1;
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let err_msg = format!("HTTP error {} fetching {}", status, url);
                        if attempts >= max_attempts || status.as_u16() == 404 {
                            return Err(ModpackError::ManifestError {
                                code: format!("HTTP_{}", status.as_u16()),
                                message: err_msg,
                                details: Some(serde_json::json!({ "url": url, "statusCode": status.as_u16() })),
                            });
                        }
                    } else {
                        match response.json::<T>().await {
                            Ok(parsed) => return Ok(parsed),
                            Err(e) => {
                                return Err(ModpackError::ManifestError {
                                    code: "JSON_PARSE_ERROR".to_string(),
                                    message: format!("Failed to parse JSON from {}: {}", url, e),
                                    details: Some(serde_json::json!({ "url": url })),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(ModpackError::DownloadError {
                            code: "NETWORK_ERROR".to_string(),
                            message: format!("Network request failed for {}: {}", url, e),
                            details: Some(serde_json::json!({ "url": url })),
                        });
                    }
                }
            }

            // Exponential backoff
            tokio::time::sleep(Duration::from_millis(300 * attempts as u64)).await;
        }
    }
}

#[async_trait]
impl RemoteModpackRepository for HttpModpackProvider {
    async fn get_catalog(
        &self,
        catalog_url: &str,
        force_refresh: bool,
    ) -> Result<ModpackCatalog, ModpackError> {
        if !force_refresh {
            let cache = self.catalog_cache.read().await;
            if let Some(entry) = cache.get(catalog_url) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    return Ok(entry.data.clone());
                }
            }
        }

        let catalog: ModpackCatalog = self.fetch_json_with_retry(catalog_url).await?;
        ManifestSchemaValidator::validate_catalog(&catalog)?;

        let mut cache = self.catalog_cache.write().await;
        cache.insert(
            catalog_url.to_string(),
            CacheEntry {
                data: catalog.clone(),
                cached_at: Instant::now(),
            },
        );

        Ok(catalog)
    }

    async fn get_main_manifest(
        &self,
        manifest_url: &str,
        force_refresh: bool,
    ) -> Result<ModpackMainManifest, ModpackError> {
        if !force_refresh {
            let cache = self.manifest_cache.read().await;
            if let Some(entry) = cache.get(manifest_url) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    return Ok(entry.data.clone());
                }
            }
        }

        let manifest: ModpackMainManifest = self.fetch_json_with_retry(manifest_url).await?;
        ManifestSchemaValidator::validate_main_manifest(&manifest)?;

        let mut cache = self.manifest_cache.write().await;
        cache.insert(
            manifest_url.to_string(),
            CacheEntry {
                data: manifest.clone(),
                cached_at: Instant::now(),
            },
        );

        Ok(manifest)
    }

    async fn get_version_manifest(
        &self,
        version_url: &str,
    ) -> Result<ModpackVersionManifest, ModpackError> {
        // Version manifests are immutable by version number, but we do not cache in RAM to ensure fresh diff during updates
        let mut manifest: ModpackVersionManifest = self.fetch_json_with_retry(version_url).await?;
        manifest.normalize_paths();
        ManifestSchemaValidator::validate_version_manifest(&manifest)?;
        Ok(manifest)
    }
}

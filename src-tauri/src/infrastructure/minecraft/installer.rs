use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use crate::application::ports::{DownloadItem, DownloadManagerPort, ManifestClientPort, ModLoaderInstallerPort};
use crate::domain::entities::{Instance, ModLoader};
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::rule_evaluator::{ArgumentRuleEvaluator, PlatformEnvironment};
use crate::infrastructure::process::supervisor::LogListener;
use crate::shared::config::LauncherPaths;
use crate::shared::utils::ZipExtractor;

pub struct MinecraftInstaller {
    manifest_client: Arc<dyn ManifestClientPort>,
    download_manager: Arc<dyn DownloadManagerPort>,
    paths: LauncherPaths,
    log_listener: Option<LogListener>,
}

impl MinecraftInstaller {
    pub fn new(
        manifest_client: Arc<dyn ManifestClientPort>,
        download_manager: Arc<dyn DownloadManagerPort>,
        paths: LauncherPaths,
        log_listener: Option<LogListener>,
    ) -> Self {
        Self {
            manifest_client,
            download_manager,
            paths,
            log_listener,
        }
    }

    fn log(&self, instance_id: &str, level: &str, message: &str) {
        tracing::info!("[{}] {}", instance_id, message);
        if let Some(listener) = &self.log_listener {
            listener(instance_id, level, message);
        }
    }

    /// Downloads and installs the plain vanilla client (jar, libraries, assets) for
    /// an instance's `minecraft_version`. Used directly for vanilla instances, and as
    /// the base install step for mod loaders (Forge, etc.) that build on top of it.
    /// Does not manage `instance.status` - the caller (an install use case) does that.
    pub async fn install_base(&self, instance: &Instance) -> Result<(), LauncherError> {
        self.log(&instance.id, "INFO", &format!("Preparing Minecraft {} (vanilla base)...", instance.minecraft_version));
        let result = self.install_base_inner(instance).await;
        match &result {
            Ok(()) => self.log(&instance.id, "INFO", "Vanilla base ready"),
            Err(e) => self.log(&instance.id, "ERROR", &format!("Vanilla install failed: {}", e)),
        }
        result
    }

    async fn install_base_inner(&self, instance: &Instance) -> Result<(), LauncherError> {
        let version_id = &instance.minecraft_version;
        let version_dir = self.paths.versions_dir().join(version_id);
        let version_json_path = version_dir.join(format!("{}.json", version_id));

        // 1. Get or fetch version JSON
        let version_json: Value = if version_json_path.exists() {
            let content = fs::read_to_string(&version_json_path).map_err(|e| {
                LauncherError::filesystem(format!("Failed to read version JSON: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                LauncherError::minecraft(format!("Invalid version JSON: {}", e), None)
            })?
        } else {
            // Locate URL from manifest
            let (_, _, versions) = self.manifest_client.fetch_versions().await?;
            let version_entry = versions.iter().find(|v| v.id == *version_id).ok_or_else(|| {
                LauncherError::not_found(format!("Minecraft version {} not found in manifest", version_id))
            })?;
            self.manifest_client.fetch_version_json(version_id, &version_entry.url).await?
        };

        let mut download_items: Vec<DownloadItem> = Vec::new();
        let env = PlatformEnvironment::current();

        // 2. Client jar download
        let client_jar_path = version_dir.join(format!("{}.jar", version_id));
        if let Some(client_dl) = version_json.pointer("/downloads/client") {
            let url = client_dl.get("url").and_then(|u| u.as_str()).unwrap_or_default();
            let sha1 = client_dl.get("sha1").and_then(|s| s.as_str()).map(|s| s.to_string());
            let size = client_dl.get("size").and_then(|s| s.as_u64());

            if !url.is_empty() {
                download_items.push(DownloadItem {
                    url: url.to_string(),
                    destination: client_jar_path.clone(),
                    sha1,
                    size,
                });
            }
        }

        // 3. Libraries downloads & natives extraction tracking
        let mut native_jars: Vec<PathBuf> = Vec::new();

        if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
            for lib in libraries {
                let rules = lib.get("rules").and_then(|r| r.as_array());
                if !ArgumentRuleEvaluator::is_allowed(rules, &env) {
                    continue;
                }

                // Standard artifact
                if let Some(artifact) = lib.pointer("/downloads/artifact") {
                    if let (Some(url), Some(path_str)) = (
                        artifact.get("url").and_then(|u| u.as_str()),
                        artifact.get("path").and_then(|p| p.as_str()),
                    ) {
                        let dest = self.paths.libraries_dir().join(path_str);
                        let sha1 = artifact.get("sha1").and_then(|s| s.as_str()).map(|s| s.to_string());
                        let size = artifact.get("size").and_then(|s| s.as_u64());

                        download_items.push(DownloadItem {
                            url: url.to_string(),
                            destination: dest.clone(),
                            sha1,
                            size,
                        });

                        // Check if library is a native jar (e.g. natives-windows)
                        if path_str.contains("natives") {
                            native_jars.push(dest);
                        }
                    }
                }

                // Classifiers for older versions
                if let Some(classifiers) = lib.pointer("/downloads/classifiers") {
                    let os_key = format!("natives-{}", env.os_name);
                    if let Some(native_artifact) = classifiers.get(&os_key) {
                        if let (Some(url), Some(path_str)) = (
                            native_artifact.get("url").and_then(|u| u.as_str()),
                            native_artifact.get("path").and_then(|p| p.as_str()),
                        ) {
                            let dest = self.paths.libraries_dir().join(path_str);
                            let sha1 = native_artifact.get("sha1").and_then(|s| s.as_str()).map(|s| s.to_string());
                            let size = native_artifact.get("size").and_then(|s| s.as_u64());

                            download_items.push(DownloadItem {
                                url: url.to_string(),
                                destination: dest.clone(),
                                sha1,
                                size,
                            });
                            native_jars.push(dest);
                        }
                    }
                }
            }
        }

        // 4. Asset Index & Asset downloads
        if let Some(asset_index) = version_json.get("assetIndex") {
            let asset_index_id = asset_index.get("id").and_then(|i| i.as_str()).unwrap_or(version_id);
            let index_url = asset_index.get("url").and_then(|u| u.as_str()).unwrap_or_default();
            let index_sha1 = asset_index.get("sha1").and_then(|s| s.as_str()).map(|s| s.to_string());
            let index_dest = self.paths.asset_indexes_dir().join(format!("{}.json", asset_index_id));

            if !index_url.is_empty() {
                // Download asset index first if not cached
                self.download_manager.download_batch(
                    Some(instance.id.clone()),
                    vec![DownloadItem {
                        url: index_url.to_string(),
                        destination: index_dest.clone(),
                        sha1: index_sha1,
                        size: asset_index.get("size").and_then(|s| s.as_u64()),
                    }],
                ).await?;

                // Parse assets and add to download queue
                if index_dest.exists() {
                    if let Ok(index_content) = fs::read_to_string(&index_dest) {
                        if let Ok(index_json) = serde_json::from_str::<Value>(&index_content) {
                            if let Some(objects) = index_json.get("objects").and_then(|o| o.as_object()) {
                                for (_asset_name, obj) in objects {
                                    if let Some(hash) = obj.get("hash").and_then(|h| h.as_str()) {
                                        if hash.len() >= 2 {
                                            let prefix = &hash[..2];
                                            let asset_url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
                                            let asset_dest = self.paths.asset_objects_dir().join(prefix).join(hash);
                                            let size = obj.get("size").and_then(|s| s.as_u64());

                                            download_items.push(DownloadItem {
                                                url: asset_url,
                                                destination: asset_dest,
                                                sha1: Some(hash.to_string()),
                                                size,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 5. Execute batch download with concurrency limiter & progress tracking
        self.log(&instance.id, "INFO", &format!("Downloading {} assets, libraries, and the client jar...", download_items.len()));
        self.download_manager.download_batch(Some(instance.id.clone()), download_items).await?;

        // 6. Extract natives into instance directory
        let natives_dir = self.paths.instance_natives_dir(&instance.id);
        for native_jar in native_jars {
            if native_jar.exists() {
                let _ = ZipExtractor::extract_natives(&native_jar, &natives_dir);
            }
        }

        tracing::info!("Installation of instance {} completed successfully", instance.id);
        Ok(())
    }
}

pub struct VanillaInstaller {
    installer: Arc<MinecraftInstaller>,
}

impl VanillaInstaller {
    pub fn new(installer: Arc<MinecraftInstaller>) -> Self {
        Self { installer }
    }
}

#[async_trait]
impl ModLoaderInstallerPort for VanillaInstaller {
    fn loader_type(&self) -> ModLoader {
        ModLoader::Vanilla
    }

    async fn install(&self, instance: &Instance) -> Result<(), LauncherError> {
        self.installer.install_base(instance).await
    }
}
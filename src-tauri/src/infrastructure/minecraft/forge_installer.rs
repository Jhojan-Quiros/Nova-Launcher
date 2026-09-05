use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zip::ZipArchive;
use crate::application::ports::{DownloadItem, DownloadManagerPort, ModLoaderInstallerPort};
use crate::application::ports::JavaDetectorPort;
use crate::domain::entities::{composite_version_id, Instance, ModLoader};
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::installer::MinecraftInstaller;
use crate::infrastructure::minecraft::rule_evaluator::{ArgumentRuleEvaluator, PlatformEnvironment};
use crate::infrastructure::process::supervisor::LogListener;
use crate::shared::config::LauncherPaths;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersionOption {
    pub version: String,
    pub label: String, // "recommended" | "latest"
}

struct LibraryNeed {
    relative_path: String,
    destination: PathBuf,
}

/// Installs Forge for an instance by downloading the official Forge installer jar and
/// replicating what it does under the hood: extract its embedded `install_profile.json`
/// and `version.json`, download the libraries it references, run its "processors"
/// (small java tools that patch the vanilla jar / produce the Forge client library),
/// then merge the resulting version JSON on top of the vanilla one so the launcher can
/// use it directly.
pub struct ForgeInstaller {
    http_client: Client,
    download_manager: Arc<dyn DownloadManagerPort>,
    java_detector: Arc<dyn JavaDetectorPort>,
    vanilla_installer: Arc<MinecraftInstaller>,
    paths: LauncherPaths,
    log_listener: Option<LogListener>,
}

impl ForgeInstaller {
    pub fn new(
        download_manager: Arc<dyn DownloadManagerPort>,
        java_detector: Arc<dyn JavaDetectorPort>,
        vanilla_installer: Arc<MinecraftInstaller>,
        paths: LauncherPaths,
        log_listener: Option<LogListener>,
    ) -> Self {
        Self {
            http_client: Client::builder()
                .user_agent("NovaLauncher/1.0")
                .build()
                .unwrap_or_default(),
            download_manager,
            java_detector,
            vanilla_installer,
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

    /// Fetches the recommended/latest Forge build numbers published for a Minecraft version.
    pub async fn fetch_available_versions(&self, mc_version: &str) -> Result<Vec<ForgeVersionOption>, LauncherError> {
        let url = format!("{}/net/minecraftforge/forge/promotions_slim.json", FORGE_MAVEN);
        let resp = self.http_client.get(&url).send().await.map_err(|e| {
            LauncherError::network(format!("Failed to reach Forge maven: {}", e))
        })?;
        if !resp.status().is_success() {
            return Err(LauncherError::network(format!("Forge maven returned status {}", resp.status())));
        }
        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid Forge promotions response: {}", e))
        })?;

        let mut out = Vec::new();
        if let Some(promos) = body.get("promos").and_then(|p| p.as_object()) {
            let recommended_key = format!("{}-recommended", mc_version);
            let latest_key = format!("{}-latest", mc_version);
            if let Some(v) = promos.get(&recommended_key).and_then(|v| v.as_str()) {
                out.push(ForgeVersionOption { version: v.to_string(), label: "recommended".to_string() });
            }
            if let Some(v) = promos.get(&latest_key).and_then(|v| v.as_str()) {
                if !out.iter().any(|o| o.version == v) {
                    out.push(ForgeVersionOption { version: v.to_string(), label: "latest".to_string() });
                }
            }
        }

        if out.is_empty() {
            return Err(LauncherError::not_found(format!(
                "No Forge builds published for Minecraft {}",
                mc_version
            )));
        }

        Ok(out)
    }

    async fn resolve_java_executable(&self, instance: &Instance) -> Result<PathBuf, LauncherError> {
        if let Some(path) = &instance.java_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }
        let runtimes = self.java_detector.detect_installed_runtimes().await?;
        let best = runtimes
            .iter()
            .filter(|r| r.is_valid)
            .max_by_key(|r| r.major_version);
        match best {
            Some(r) => Ok(PathBuf::from(&r.path)),
            None => Err(LauncherError::java(
                "No valid Java installation found to run the Forge installer. Please install Java or configure it in Settings.",
            )),
        }
    }

    async fn download_file(&self, url: &str, dest: &Path) -> Result<(), LauncherError> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }
        let resp = self.http_client.get(url).send().await.map_err(|e| {
            LauncherError::network(format!("Failed to download {}: {}", url, e))
        })?;
        if !resp.status().is_success() {
            return Err(LauncherError::network(format!("Server returned status {} for {}", resp.status(), url)));
        }
        let bytes = resp.bytes().await.map_err(|e| {
            LauncherError::network(format!("Failed to read response body from {}: {}", url, e))
        })?;
        let mut file = File::create(dest).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create {:?}: {}", dest, e))
        })?;
        file.write_all(&bytes).map_err(|e| {
            LauncherError::filesystem(format!("Failed to write {:?}: {}", dest, e))
        })?;
        Ok(())
    }
}

#[async_trait]
impl ModLoaderInstallerPort for ForgeInstaller {
    fn loader_type(&self) -> ModLoader {
        ModLoader::Forge
    }

    async fn install(&self, instance: &Instance) -> Result<(), LauncherError> {
        self.log(&instance.id, "INFO", &format!(
            "Installing Forge {} for Minecraft {}...",
            instance.loader_version.as_deref().unwrap_or("?"),
            instance.minecraft_version
        ));
        let result = self.install_impl(instance).await;
        match &result {
            Ok(()) => self.log(&instance.id, "INFO", "Forge installed successfully"),
            Err(e) => self.log(&instance.id, "ERROR", &format!("Forge install failed: {}", e)),
        }
        result
    }
}

impl ForgeInstaller {
    async fn install_impl(&self, instance: &Instance) -> Result<(), LauncherError> {
        // Forge builds on top of the vanilla client (jar, assets, base libraries).
        self.vanilla_installer.install_base(instance).await?;

        let mc_version = instance.minecraft_version.clone();
        let forge_version = instance.loader_version.clone().ok_or_else(|| {
            LauncherError::validation("This instance has no Forge version configured")
        })?;

        let composite_id = composite_version_id(&mc_version, ModLoader::Forge, &forge_version);
        let composite_dir = self.paths.versions_dir().join(&composite_id);
        let composite_json_path = composite_dir.join(format!("{}.json", composite_id));

        if composite_json_path.exists() {
            return Ok(());
        }
        fs::create_dir_all(&composite_dir).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create {:?}: {}", composite_dir, e))
        })?;

        let full_version = format!("{}-{}", mc_version, forge_version);
        let installer_path = self
            .paths
            .root()
            .join("cache")
            .join("forge")
            .join(format!("forge-{}-installer.jar", full_version));

        if !installer_path.exists() {
            let installer_url = format!(
                "{}/net/minecraftforge/forge/{full}/forge-{full}-installer.jar",
                FORGE_MAVEN,
                full = full_version
            );
            self.log(&instance.id, "INFO", &format!("Downloading Forge installer from {}...", installer_url));
            self.download_file(&installer_url, &installer_path).await?;
        }

        let installer_bytes = fs::read(&installer_path).map_err(|e| {
            LauncherError::filesystem(format!("Failed to read Forge installer jar: {}", e))
        })?;

        let install_profile = Self::read_zip_json(&installer_bytes, "install_profile.json")?;

        if install_profile.get("processors").is_none() {
            return Err(LauncherError::minecraft(
                format!(
                    "Forge {} uses a legacy installer format that isn't supported yet. Try a newer Forge build.",
                    full_version
                ),
                None,
            ));
        }

        let version_json_entry = install_profile
            .get("json")
            .and_then(|j| j.as_str())
            .unwrap_or("/version.json")
            .trim_start_matches('/')
            .to_string();
        let forge_version_json = Self::read_zip_json(&installer_bytes, &version_json_entry)?;

        // Collect libraries to download from both the installer profile and the Forge
        // version JSON. Entries with no download URL are either bundled inside the
        // installer jar (under "maven/...") or produced later by a processor.
        let env = PlatformEnvironment::current();
        let mut download_items: Vec<DownloadItem> = Vec::new();
        let mut bundled_needed: Vec<LibraryNeed> = Vec::new();

        if let Some(libs) = install_profile.get("libraries").and_then(|l| l.as_array()) {
            Self::collect_libraries(libs, &self.paths, &env, &mut download_items, &mut bundled_needed);
        }
        if let Some(libs) = forge_version_json.get("libraries").and_then(|l| l.as_array()) {
            Self::collect_libraries(libs, &self.paths, &env, &mut download_items, &mut bundled_needed);
        }

        self.log(&instance.id, "INFO", &format!("Downloading {} Forge libraries...", download_items.len()));
        self.download_manager.download_batch(Some(instance.id.clone()), download_items).await?;

        for need in bundled_needed {
            if need.destination.exists() {
                continue;
            }
            let entry_name = format!("maven/{}", need.relative_path);
            if let Err(e) = Self::extract_zip_entry_to_file(&installer_bytes, &entry_name, &need.destination) {
                tracing::debug!(
                    "Library {} not bundled in installer and not downloadable ({}); assuming a processor will produce it",
                    need.relative_path, e
                );
            }
        }

        // Build the placeholder map used to substitute processor arguments.
        let client_jar = self
            .paths
            .versions_dir()
            .join(&mc_version)
            .join(format!("{}.jar", mc_version));

        let data_map = Self::build_data_map(
            &install_profile,
            &installer_bytes,
            &composite_dir,
            &self.paths,
            &mc_version,
            &client_jar,
            &installer_path,
        )?;

        let java_exe = self.resolve_java_executable(instance).await?;
        self.log(&instance.id, "INFO", &format!("Using Java: {}", java_exe.display()));

        if let Some(processors) = install_profile.get("processors").and_then(|p| p.as_array()) {
            let total = processors.len();
            for (idx, processor) in processors.iter().enumerate() {
                self.log(&instance.id, "INFO", &format!("Running Forge install step {}/{}...", idx + 1, total));
                Self::run_processor(processor, &data_map, &self.paths, &java_exe, idx).await.map_err(|e| {
                    self.log(&instance.id, "ERROR", &format!("Install step {}/{} failed: {}", idx + 1, total, e));
                    if let Some(details) = e.to_payload().details {
                        for line in details.lines().filter(|l| !l.trim().is_empty()) {
                            self.log(&instance.id, "ERROR", line);
                        }
                    }
                    e
                })?;
            }
        }

        // Merge the Forge version JSON on top of vanilla and persist it as the
        // composite version used by the launcher for this instance.
        let vanilla_json_path = self
            .paths
            .versions_dir()
            .join(&mc_version)
            .join(format!("{}.json", mc_version));
        let vanilla_json_content = fs::read_to_string(&vanilla_json_path).map_err(|e| {
            LauncherError::filesystem(format!("Failed to read vanilla version JSON: {}", e))
        })?;
        let vanilla_json: Value = serde_json::from_str(&vanilla_json_content).map_err(|e| {
            LauncherError::minecraft(format!("Invalid vanilla version JSON: {}", e), None)
        })?;

        let merged = Self::merge_version_json(&vanilla_json, &forge_version_json);
        let merged_content = serde_json::to_string_pretty(&merged).map_err(|e| {
            LauncherError::internal(format!("Failed to serialize merged version JSON: {}", e))
        })?;
        fs::write(&composite_json_path, merged_content).map_err(|e| {
            LauncherError::filesystem(format!("Failed to write {:?}: {}", composite_json_path, e))
        })?;

        tracing::info!("Forge {} installed successfully for instance {}", full_version, instance.id);
        Ok(())
    }
}

impl ForgeInstaller {
    fn read_zip_json(bytes: &[u8], entry_name: &str) -> Result<Value, LauncherError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
            LauncherError::internal(format!("Invalid Forge installer archive: {}", e))
        })?;
        let mut entry = archive.by_name(entry_name).map_err(|e| {
            LauncherError::minecraft(format!("Forge installer is missing '{}': {}", entry_name, e), None)
        })?;
        let mut content = String::new();
        entry.read_to_string(&mut content).map_err(|e| {
            LauncherError::filesystem(format!("Failed to read '{}' from installer: {}", entry_name, e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            LauncherError::minecraft(format!("Invalid JSON in '{}': {}", entry_name, e), None)
        })
    }

    fn extract_zip_entry_to_file(bytes: &[u8], entry_name: &str, dest: &Path) -> Result<(), LauncherError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
            LauncherError::internal(format!("Invalid Forge installer archive: {}", e))
        })?;
        let mut entry = archive.by_name(entry_name).map_err(|_| {
            LauncherError::minecraft(format!("Entry '{}' not found in installer", entry_name), None)
        })?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }
        let mut out_file = File::create(dest).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create {:?}: {}", dest, e))
        })?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| {
            LauncherError::filesystem(format!("Failed to extract '{}': {}", entry_name, e))
        })?;
        Ok(())
    }

    /// Converts a maven coordinate ("group:artifact:version[:classifier][@ext]") into
    /// its relative path under a maven-style libraries directory.
    fn maven_coord_to_relative_path(coord: &str) -> Result<String, LauncherError> {
        let (coord, ext) = match coord.split_once('@') {
            Some((c, e)) => (c, e),
            None => (coord, "jar"),
        };
        let parts: Vec<&str> = coord.split(':').collect();
        if parts.len() < 3 {
            return Err(LauncherError::minecraft(format!("Invalid maven coordinate: {}", coord), None));
        }
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let file_name = match parts.get(3) {
            Some(classifier) => format!("{}-{}-{}.{}", artifact, version, classifier, ext),
            None => format!("{}-{}.{}", artifact, version, ext),
        };
        Ok(format!("{}/{}/{}/{}", group, artifact, version, file_name))
    }

    fn collect_libraries(
        libraries: &[Value],
        paths: &LauncherPaths,
        env: &PlatformEnvironment,
        download_items: &mut Vec<DownloadItem>,
        bundled_needed: &mut Vec<LibraryNeed>,
    ) {
        for lib in libraries {
            let rules = lib.get("rules").and_then(|r| r.as_array());
            if !ArgumentRuleEvaluator::is_allowed(rules, env) {
                continue;
            }
            let Some(name) = lib.get("name").and_then(|n| n.as_str()) else { continue };
            let Ok(rel) = Self::maven_coord_to_relative_path(name) else { continue };
            let dest = paths.libraries_dir().join(&rel);

            let artifact = lib.pointer("/downloads/artifact");
            let url = artifact.and_then(|a| a.get("url")).and_then(|u| u.as_str()).unwrap_or("");

            if !url.is_empty() {
                let sha1 = artifact.and_then(|a| a.get("sha1")).and_then(|s| s.as_str()).map(String::from);
                let size = artifact.and_then(|a| a.get("size")).and_then(|s| s.as_u64());
                download_items.push(DownloadItem {
                    url: url.to_string(),
                    destination: dest,
                    sha1,
                    size,
                });
            } else if !dest.exists() {
                bundled_needed.push(LibraryNeed { relative_path: rel, destination: dest });
            }
        }
    }

    fn manifest_main_class(jar_path: &Path) -> Result<String, LauncherError> {
        let file = File::open(jar_path).map_err(|e| {
            LauncherError::filesystem(format!("Failed to open {:?}: {}", jar_path, e))
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| {
            LauncherError::internal(format!("Invalid jar {:?}: {}", jar_path, e))
        })?;
        let mut manifest_content = String::new();
        {
            let mut entry = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
                LauncherError::minecraft(format!("{:?} has no manifest", jar_path), None)
            })?;
            entry.read_to_string(&mut manifest_content).map_err(|e| {
                LauncherError::filesystem(format!("Failed to read manifest of {:?}: {}", jar_path, e))
            })?;
        }

        // Unwrap manifest line continuations: a line starting with a single space
        // continues the previous logical line (per the JAR manifest spec).
        let mut logical_lines: Vec<String> = Vec::new();
        for raw_line in manifest_content.lines() {
            if let Some(cont) = raw_line.strip_prefix(' ') {
                if let Some(last) = logical_lines.last_mut() {
                    last.push_str(cont);
                    continue;
                }
            }
            logical_lines.push(raw_line.to_string());
        }

        logical_lines
            .iter()
            .find_map(|line| line.strip_prefix("Main-Class:").map(|v| v.trim().to_string()))
            .ok_or_else(|| LauncherError::minecraft(format!("No Main-Class in manifest of {:?}", jar_path), None))
    }

    fn build_data_map(
        install_profile: &Value,
        installer_bytes: &[u8],
        extract_dir: &Path,
        paths: &LauncherPaths,
        mc_version: &str,
        client_jar: &Path,
        installer_path: &Path,
    ) -> Result<HashMap<String, String>, LauncherError> {
        let mut map = HashMap::new();
        map.insert("SIDE".to_string(), "client".to_string());
        map.insert("MINECRAFT_JAR".to_string(), client_jar.to_string_lossy().to_string());
        map.insert("MINECRAFT_VERSION".to_string(), mc_version.to_string());
        map.insert("ROOT".to_string(), extract_dir.to_string_lossy().to_string());
        map.insert("INSTALLER".to_string(), installer_path.to_string_lossy().to_string());
        map.insert("LIBRARY_DIR".to_string(), paths.libraries_dir().to_string_lossy().to_string());

        if let Some(data) = install_profile.get("data").and_then(|d| d.as_object()) {
            for (key, entry) in data {
                let Some(raw) = entry.get("client").and_then(|v| v.as_str()) else { continue };

                let resolved = if raw.starts_with('[') && raw.ends_with(']') {
                    let coord = &raw[1..raw.len() - 1];
                    let rel = Self::maven_coord_to_relative_path(coord)?;
                    paths.libraries_dir().join(rel).to_string_lossy().to_string()
                } else if let Some(inner_path) = raw.strip_prefix('/') {
                    let dest = extract_dir.join("data").join(inner_path);
                    Self::extract_zip_entry_to_file(installer_bytes, inner_path, &dest)?;
                    dest.to_string_lossy().to_string()
                } else {
                    raw.to_string()
                };

                map.insert(key.clone(), resolved);
            }
        }

        Ok(map)
    }

    fn substitute_arg(arg: &str, data: &HashMap<String, String>, paths: &LauncherPaths) -> Result<String, LauncherError> {
        if arg.len() > 2 && arg.starts_with('{') && arg.ends_with('}') {
            let key = &arg[1..arg.len() - 1];
            return data.get(key).cloned().ok_or_else(|| {
                LauncherError::minecraft(format!("Unknown processor placeholder: {{{}}}", key), None)
            });
        }
        if arg.len() > 2 && arg.starts_with('[') && arg.ends_with(']') {
            let coord = &arg[1..arg.len() - 1];
            let rel = Self::maven_coord_to_relative_path(coord)?;
            return Ok(paths.libraries_dir().join(rel).to_string_lossy().to_string());
        }
        Ok(arg.to_string())
    }

    async fn run_processor(
        processor: &Value,
        data: &HashMap<String, String>,
        paths: &LauncherPaths,
        java_exe: &Path,
        index: usize,
    ) -> Result<(), LauncherError> {
        if let Some(sides) = processor.get("sides").and_then(|s| s.as_array()) {
            let allowed = sides.iter().any(|s| s.as_str() == Some("client"));
            if !allowed {
                return Ok(());
            }
        }

        let jar_coord = processor.get("jar").and_then(|j| j.as_str()).ok_or_else(|| {
            LauncherError::minecraft(format!("Forge processor #{} has no jar", index), None)
        })?;
        let jar_rel = Self::maven_coord_to_relative_path(jar_coord)?;
        let jar_path = paths.libraries_dir().join(&jar_rel);
        if !jar_path.exists() {
            return Err(LauncherError::minecraft(
                format!("Forge processor jar not found: {:?}", jar_path),
                None,
            ));
        }

        let main_class = Self::manifest_main_class(&jar_path)?;

        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        let mut classpath_parts = vec![jar_path.to_string_lossy().to_string()];
        if let Some(cp) = processor.get("classpath").and_then(|c| c.as_array()) {
            for entry in cp {
                if let Some(coord) = entry.as_str() {
                    let rel = Self::maven_coord_to_relative_path(coord)?;
                    classpath_parts.push(paths.libraries_dir().join(rel).to_string_lossy().to_string());
                }
            }
        }
        let classpath = classpath_parts.join(sep);

        let mut args = Vec::new();
        if let Some(raw_args) = processor.get("args").and_then(|a| a.as_array()) {
            for a in raw_args {
                if let Some(s) = a.as_str() {
                    args.push(Self::substitute_arg(s, data, paths)?);
                }
            }
        }

        tracing::info!("Running Forge processor #{}: {}", index, main_class);

        let output = tokio::process::Command::new(java_exe)
            .arg("-cp")
            .arg(&classpath)
            .arg(&main_class)
            .args(&args)
            .output()
            .await
            .map_err(|e| LauncherError::process(format!("Failed to run Forge processor {}: {}", main_class, e)))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LauncherError::minecraft(
                format!("Forge processor {} failed (exit {:?})", main_class, output.status.code()),
                Some(format!("{}\n{}", stdout, stderr)),
            ));
        }

        Ok(())
    }

    /// Merges a mod loader's version JSON on top of the vanilla one it targets:
    /// the loader's mainClass wins, its game/jvm arguments are appended after the
    /// vanilla ones, and its libraries take priority over same group:artifact entries
    /// inherited from vanilla.
    fn merge_version_json(vanilla: &Value, child: &Value) -> Value {
        let mut merged = vanilla.clone();
        let Some(merged_obj) = merged.as_object_mut() else { return merged };

        if let Some(main_class) = child.get("mainClass") {
            merged_obj.insert("mainClass".to_string(), main_class.clone());
        }

        if let Some(java_version) = child.get("javaVersion") {
            merged_obj.insert("javaVersion".to_string(), java_version.clone());
        }

        if let Some(legacy_args) = child.get("minecraftArguments") {
            merged_obj.insert("minecraftArguments".to_string(), legacy_args.clone());
        }

        if let Some(child_args) = child.get("arguments") {
            let mut game = merged_obj
                .get("arguments")
                .and_then(|a| a.get("game"))
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![]));
            let mut jvm = merged_obj
                .get("arguments")
                .and_then(|a| a.get("jvm"))
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![]));

            if let (Some(arr), Some(extra)) = (game.as_array_mut(), child_args.get("game").and_then(|g| g.as_array())) {
                arr.extend(extra.clone());
            }
            if let (Some(arr), Some(extra)) = (jvm.as_array_mut(), child_args.get("jvm").and_then(|j| j.as_array())) {
                arr.extend(extra.clone());
            }

            merged_obj.insert("arguments".to_string(), serde_json::json!({ "game": game, "jvm": jvm }));
        }

        let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut libraries: Vec<Value> = Vec::new();

        if let Some(child_libs) = child.get("libraries").and_then(|l| l.as_array()) {
            for lib in child_libs {
                if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    seen_keys.insert(Self::group_artifact_key(name));
                }
                libraries.push(lib.clone());
            }
        }
        if let Some(parent_libs) = merged_obj.get("libraries").and_then(|l| l.as_array()) {
            for lib in parent_libs {
                if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    if seen_keys.contains(&Self::group_artifact_key(name)) {
                        continue;
                    }
                }
                libraries.push(lib.clone());
            }
        }
        merged_obj.insert("libraries".to_string(), Value::Array(libraries));

        merged
    }

    fn group_artifact_key(maven_name: &str) -> String {
        let parts: Vec<&str> = maven_name.splitn(3, ':').collect();
        format!("{}:{}", parts.first().unwrap_or(&""), parts.get(1).unwrap_or(&""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maven_coord_resolves_simple_artifact() {
        let rel = ForgeInstaller::maven_coord_to_relative_path("net.minecraftforge:installertools:1.3.0").unwrap();
        assert_eq!(rel, "net/minecraftforge/installertools/1.3.0/installertools-1.3.0.jar");
    }

    #[test]
    fn maven_coord_resolves_classifier_and_extension() {
        let rel = ForgeInstaller::maven_coord_to_relative_path(
            "de.oceanlabs.mcp:mcp_config:1.20.1-20230612.114412:mappings@txt",
        )
        .unwrap();
        assert_eq!(
            rel,
            "de/oceanlabs/mcp/mcp_config/1.20.1-20230612.114412/mcp_config-1.20.1-20230612.114412-mappings.txt"
        );
    }

    #[test]
    fn maven_coord_rejects_invalid_input() {
        assert!(ForgeInstaller::maven_coord_to_relative_path("not-a-coordinate").is_err());
    }

    #[test]
    fn group_artifact_key_ignores_version() {
        assert_eq!(
            ForgeInstaller::group_artifact_key("net.minecraftforge:forge:1.20.1-47.2.20"),
            "net.minecraftforge:forge"
        );
    }

    #[test]
    fn merge_prefers_child_main_class_and_appends_arguments() {
        let vanilla = json!({
            "mainClass": "net.minecraft.client.main.Main",
            "arguments": {
                "game": ["--username", "${auth_player_name}"],
                "jvm": ["-Djava.library.path=${natives_directory}"]
            },
            "libraries": [
                { "name": "com.mojang:brigadier:1.0.18" }
            ]
        });
        let child = json!({
            "mainClass": "cpw.mods.bootstraplauncher.BootstrapLauncher",
            "arguments": {
                "game": ["--launchTarget", "forgeclient"],
                "jvm": ["-p", "${library_directory}"]
            },
            "libraries": [
                { "name": "net.minecraftforge:forge:1.20.1-47.2.20" },
                { "name": "com.mojang:brigadier:1.0.18" }
            ]
        });

        let merged = ForgeInstaller::merge_version_json(&vanilla, &child);

        assert_eq!(merged["mainClass"], "cpw.mods.bootstraplauncher.BootstrapLauncher");
        assert_eq!(
            merged["arguments"]["game"],
            json!(["--username", "${auth_player_name}", "--launchTarget", "forgeclient"])
        );
        assert_eq!(
            merged["arguments"]["jvm"],
            json!(["-Djava.library.path=${natives_directory}", "-p", "${library_directory}"])
        );

        // brigadier appears once (child's copy wins, vanilla's duplicate is dropped)
        let libs = merged["libraries"].as_array().unwrap();
        let brigadier_count = libs.iter().filter(|l| l["name"] == "com.mojang:brigadier:1.0.18").count();
        assert_eq!(brigadier_count, 1);
        assert!(libs.iter().any(|l| l["name"] == "net.minecraftforge:forge:1.20.1-47.2.20"));
    }
}

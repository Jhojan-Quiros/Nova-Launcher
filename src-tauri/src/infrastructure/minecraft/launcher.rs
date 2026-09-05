use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use crate::application::ports::{AuthenticationProviderPort, JavaDetectorPort, MinecraftLauncherPort};
use crate::domain::entities::{composite_version_id, Instance, InstanceStatus, ModLoader};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::{InstanceRepository, SettingsRepository};
use crate::infrastructure::minecraft::argument_builder::ArgumentBuilder;
use crate::infrastructure::minecraft::rule_evaluator::PlatformEnvironment;
use crate::infrastructure::process::supervisor::{LogListener, ProcessSupervisor};
use crate::shared::config::LauncherPaths;

pub struct MinecraftLauncherService {
    instance_repo: Arc<dyn InstanceRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    auth_provider: Arc<dyn AuthenticationProviderPort>,
    java_detector: Arc<dyn JavaDetectorPort>,
    paths: LauncherPaths,
    log_listener: Option<LogListener>,
}

impl MinecraftLauncherService {
    pub fn new(
        instance_repo: Arc<dyn InstanceRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        auth_provider: Arc<dyn AuthenticationProviderPort>,
        java_detector: Arc<dyn JavaDetectorPort>,
        paths: LauncherPaths,
        log_listener: Option<LogListener>,
    ) -> Self {
        Self {
            instance_repo,
            settings_repo,
            auth_provider,
            java_detector,
            paths,
            log_listener,
        }
    }

    /// Picks the Java executable to launch with. `required_major` is the Java major
    /// version the version JSON asks for (e.g. 17 for Minecraft 1.20.1/Forge) - old
    /// Forge/Mixin builds fail with cryptic "Unsupported class file major version"
    /// crashes when run on a much newer JDK, so auto-detection must match it rather
    /// than just grabbing the first or newest Java found on the system.
    async fn resolve_java_executable(&self, instance: &Instance, required_major: Option<u32>) -> Result<PathBuf, LauncherError> {
        // 1. Instance override - trust the user's explicit choice.
        if let Some(path) = &instance.java_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        // 2. Global settings override - same.
        let settings = self.settings_repo.get().await?;
        if let Some(path) = &settings.default_java_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        // 3. Auto-detected runtimes: prefer one matching the required major version.
        let runtimes = self.java_detector.detect_installed_runtimes().await?;
        let valid: Vec<_> = runtimes.iter().filter(|r| r.is_valid).collect();

        if let Some(required) = required_major {
            if let Some(matching) = valid.iter().find(|r| r.major_version == required) {
                return Ok(PathBuf::from(&matching.path));
            }
            if !valid.is_empty() {
                let found: Vec<String> = valid.iter().map(|r| format!("Java {}", r.major_version)).collect();
                return Err(LauncherError::java(format!(
                    "This instance needs Java {}, but only {} was found on this system. Install Java {} (e.g. Eclipse Temurin {}) and select it in Settings or in this instance's settings.",
                    required, found.join(", "), required, required
                )));
            }
        } else if let Some(first) = valid.first() {
            return Ok(PathBuf::from(&first.path));
        }

        Err(LauncherError::java(
            "No valid Java installation found. Please configure a Java executable in Settings or install Java.",
        ))
    }

    fn log(&self, instance_id: &str, level: &str, message: &str) {
        tracing::info!("[{}] {}", instance_id, message);
        if let Some(listener) = &self.log_listener {
            listener(instance_id, level, message);
        }
    }
}

#[async_trait]
impl MinecraftLauncherPort for MinecraftLauncherService {
    async fn launch(&self, instance: &Instance) -> Result<(), LauncherError> {
        self.log(&instance.id, "INFO", "Preparing to launch...");
        let result = self.launch_impl(instance).await;
        if let Err(e) = &result {
            self.log(&instance.id, "ERROR", &format!("Launch failed: {}", e));
        }
        result
    }
}

impl MinecraftLauncherService {
    async fn launch_impl(&self, instance: &Instance) -> Result<(), LauncherError> {
        let mut inst = instance.clone();
        if inst.status == InstanceStatus::Running {
            return Err(LauncherError::validation("This instance is already running"));
        }

        // Read version JSON: vanilla instances launch straight off the Mojang version
        // JSON, while modded instances launch off the composite JSON produced at
        // install time (vanilla merged with the mod loader's own version JSON).
        let version_id = match inst.loader {
            ModLoader::Vanilla => inst.minecraft_version.clone(),
            ModLoader::Forge => {
                let loader_version = inst.loader_version.as_ref().ok_or_else(|| {
                    LauncherError::validation("This instance has no Forge version configured")
                })?;
                composite_version_id(&inst.minecraft_version, ModLoader::Forge, loader_version)
            }
            ModLoader::Fabric | ModLoader::NeoForge => {
                return Err(LauncherError::validation(format!(
                    "The '{}' mod loader is not supported yet",
                    inst.loader
                )));
            }
        };
        let version_json_file = self.paths.versions_dir().join(&version_id).join(format!("{}.json", version_id));
        if !version_json_file.exists() {
            return Err(LauncherError::not_found(format!(
                "Version JSON for {} not found. Please reinstall the instance.",
                version_id
            )));
        }

        let content = fs::read_to_string(&version_json_file).map_err(|e| {
            LauncherError::filesystem(format!("Failed to read version JSON: {}", e))
        })?;
        let version_json: Value = serde_json::from_str(&content).map_err(|e| {
            LauncherError::minecraft(format!("Invalid version JSON: {}", e), None)
        })?;

        let required_java_major = version_json
            .pointer("/javaVersion/majorVersion")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let java_exe = self.resolve_java_executable(&inst, required_java_major).await?;
        self.log(&inst.id, "INFO", &format!("Using Java: {}", java_exe.display()));

        let account = self.auth_provider.get_active_account().await?;
        let env = PlatformEnvironment::current();

        let launch_args = ArgumentBuilder::build(
            &version_json,
            &inst,
            &self.paths,
            &account,
            &env,
        )?;

        let mut full_command_args = Vec::new();
        full_command_args.extend(launch_args.jvm_args);
        full_command_args.push(launch_args.main_class);
        full_command_args.extend(launch_args.game_args);

        // Update status to Running
        inst.set_status(InstanceStatus::Running);
        self.instance_repo.save(&inst).await?;

        let game_dir = if std::path::Path::new(&inst.game_directory).is_absolute() {
            PathBuf::from(&inst.game_directory)
        } else {
            self.paths.instance_game_dir(&inst.id)
        };
        let start_time = Instant::now();
        let instance_id = inst.id.clone();
        let listener = self.log_listener.clone();
        let repo = self.instance_repo.clone();

        self.log(&inst.id, "INFO", &format!("Launching Java process in {:?}...", game_dir));
        tracing::debug!("Full launch command for {}: {:?} {:?}", instance_id, java_exe, full_command_args);

        // Spawn process asynchronously
        let spawn_listener = listener.clone();
        tokio::spawn(async move {
            let result = ProcessSupervisor::spawn_and_monitor(
                &java_exe,
                &full_command_args,
                &game_dir,
                &instance_id,
                listener,
            ).await;

            let duration = start_time.elapsed().as_secs();

            if let Ok(mut current) = repo.find_by_id(&instance_id).await {
                if let Some(ref mut c) = current {
                    c.record_play_session(duration);
                    match result {
                        Ok(code) => {
                            if code != 0 {
                                let msg = format!("Game exited with a non-zero code ({}). Check the log lines above for the actual error.", code);
                                tracing::warn!("{}", msg);
                                if let Some(l) = &spawn_listener {
                                    l(&instance_id, "WARN", &msg);
                                }
                            }
                            c.set_status(InstanceStatus::Ready);
                        }
                        Err(e) => {
                            tracing::error!("Game execution error: {}", e);
                            if let Some(l) = &spawn_listener {
                                l(&instance_id, "ERROR", &format!("Game execution error: {}", e));
                            }
                            c.set_status(InstanceStatus::Error(e.to_string()));
                        }
                    }
                    let _ = repo.save(c).await;
                }
            }
        });

        Ok(())
    }
}
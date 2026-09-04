use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use crate::application::ports::{AuthenticationProviderPort, JavaDetectorPort, MinecraftLauncherPort};
use crate::domain::entities::{Instance, InstanceStatus};
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

    async fn resolve_java_executable(&self, instance: &Instance) -> Result<PathBuf, LauncherError> {
        // 1. Instance override
        if let Some(path) = &instance.java_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        // 2. Global settings override
        let settings = self.settings_repo.get().await?;
        if let Some(path) = &settings.default_java_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        // 3. Auto-detected runtimes
        let runtimes = self.java_detector.detect_installed_runtimes().await?;
        if let Some(first) = runtimes.first() {
            return Ok(PathBuf::from(&first.path));
        }

        Err(LauncherError::java(
            "No valid Java installation found. Please configure a Java executable in Settings or install Java.",
        ))
    }
}

#[async_trait]
impl MinecraftLauncherPort for MinecraftLauncherService {
    async fn launch(&self, instance: &Instance) -> Result<(), LauncherError> {
        let mut inst = instance.clone();
        if inst.status == InstanceStatus::Running {
            return Err(LauncherError::validation("This instance is already running"));
        }

        let java_exe = self.resolve_java_executable(&inst).await?;

        // Read version JSON
        let version_id = &inst.minecraft_version;
        let version_json_file = self.paths.versions_dir().join(version_id).join(format!("{}.json", version_id));
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

        // Spawn process asynchronously
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
                                tracing::warn!("Game exited with code {}", code);
                            }
                            c.set_status(InstanceStatus::Ready);
                        }
                        Err(e) => {
                            tracing::error!("Game execution error: {}", e);
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
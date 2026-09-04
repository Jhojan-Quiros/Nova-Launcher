use std::sync::Arc;
use uuid::Uuid;
use crate::application::dto::CreateInstanceDto;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::{InstanceRepository, SettingsRepository};
use crate::domain::value_objects::ram_config::RamConfig;
use crate::shared::config::LauncherPaths;

pub struct CreateInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    paths: LauncherPaths,
}

impl CreateInstanceUseCase {
    pub fn new(
        instance_repo: Arc<dyn InstanceRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        paths: LauncherPaths,
    ) -> Self {
        Self {
            instance_repo,
            settings_repo,
            paths,
        }
    }

    pub async fn execute(&self, dto: CreateInstanceDto) -> Result<Instance, LauncherError> {
        let name = dto.name.trim();
        if name.is_empty() {
            return Err(LauncherError::validation("Instance name cannot be empty"));
        }
        if name.len() > 64 {
            return Err(LauncherError::validation("Instance name cannot exceed 64 characters"));
        }

        // Sanitize for folder name
        let sanitized: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let id = format!("{}-{}", sanitized.to_lowercase(), &Uuid::new_v4().to_string()[..8]);

        if self.instance_repo.exists(&id).await? {
            return Err(LauncherError::validation("An instance with this ID already exists"));
        }

        let settings = self.settings_repo.get().await?;
        let min_ram = dto.min_ram.unwrap_or(settings.default_min_ram);
        let max_ram = dto.max_ram.unwrap_or(settings.default_max_ram);
        let ram = RamConfig::new(min_ram, max_ram)?;

        // Ensure directories are initialized
        let game_dir = self.paths.init_instance_directory(&id)?;

        let instance = Instance::new(
            id.clone(),
            name.to_string(),
            dto.minecraft_version,
            dto.loader,
            dto.loader_version,
            game_dir.to_string_lossy().to_string(),
            ram,
        );

        // Save instance.json metadata file inside the instance directory
        let meta_file = self.paths.instance_dir(&id).join("instance.json");
        let meta_json = serde_json::to_string_pretty(&instance)
            .map_err(|e| LauncherError::internal(e.to_string()))?;
        std::fs::write(&meta_file, meta_json).map_err(|e| {
            LauncherError::filesystem(format!("Failed to write instance.json: {}", e))
        })?;

        self.instance_repo.save(&instance).await?;

        Ok(instance)
    }
}
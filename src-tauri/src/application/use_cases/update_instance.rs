use std::sync::Arc;
use crate::application::dto::UpdateInstanceDto;
use crate::domain::entities::Instance;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::InstanceRepository;
use crate::domain::value_objects::ram_config::RamConfig;
use crate::shared::config::LauncherPaths;

pub struct UpdateInstanceUseCase {
    instance_repo: Arc<dyn InstanceRepository>,
    paths: LauncherPaths,
}

impl UpdateInstanceUseCase {
    pub fn new(instance_repo: Arc<dyn InstanceRepository>, paths: LauncherPaths) -> Self {
        Self { instance_repo, paths }
    }

    pub async fn execute(&self, id: &str, dto: UpdateInstanceDto) -> Result<Instance, LauncherError> {
        let mut instance = self.instance_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| LauncherError::not_found(format!("Instance with id '{}' not found", id)))?;

        if let Some(name) = dto.name {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(LauncherError::validation("Instance name cannot be empty"));
            }
            instance.name = trimmed.to_string();
        }

        if let Some(java_path) = dto.java_path {
            instance.java_path = if java_path.trim().is_empty() {
                None
            } else {
                Some(java_path.trim().to_string())
            };
        }

        if dto.min_ram.is_some() || dto.max_ram.is_some() {
            let min = dto.min_ram.unwrap_or(instance.ram.min_mb);
            let max = dto.max_ram.unwrap_or(instance.ram.max_mb);
            instance.ram = RamConfig::new(min, max)?;
        }

        instance.updated_at = chrono::Utc::now();

        // Update instance.json file
        let meta_file = self.paths.instance_dir(id).join("instance.json");
        if meta_file.exists() {
            let meta_json = serde_json::to_string_pretty(&instance)
                .map_err(|e| LauncherError::internal(e.to_string()))?;
            let _ = std::fs::write(&meta_file, meta_json);
        }

        self.instance_repo.save(&instance).await?;

        Ok(instance)
    }
}
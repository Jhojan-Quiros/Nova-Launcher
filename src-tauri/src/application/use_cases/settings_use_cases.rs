use std::sync::Arc;
use crate::application::dto::UpdateSettingsDto;
use crate::domain::entities::AppSettings;
use crate::domain::errors::LauncherError;
use crate::domain::repositories::SettingsRepository;

pub struct GetSettingsUseCase {
    settings_repo: Arc<dyn SettingsRepository>,
}

impl GetSettingsUseCase {
    pub fn new(settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { settings_repo }
    }

    pub async fn execute(&self) -> Result<AppSettings, LauncherError> {
        self.settings_repo.get().await
    }
}

pub struct UpdateSettingsUseCase {
    settings_repo: Arc<dyn SettingsRepository>,
}

impl UpdateSettingsUseCase {
    pub fn new(settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { settings_repo }
    }

    pub async fn execute(&self, dto: UpdateSettingsDto) -> Result<AppSettings, LauncherError> {
        let mut settings = self.settings_repo.get().await?;
        dto.apply_to(&mut settings);
        self.settings_repo.save(&settings).await?;
        Ok(settings)
    }
}
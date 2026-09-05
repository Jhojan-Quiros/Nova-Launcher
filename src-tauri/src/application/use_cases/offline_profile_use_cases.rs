use std::sync::Arc;
use regex::Regex;
use crate::domain::entities::{AccountType, OfflineProfile};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::{OfflineProfileRepository, SettingsRepository};

pub struct GetActiveOfflineProfileUseCase {
    repo: Arc<dyn OfflineProfileRepository>,
}

impl GetActiveOfflineProfileUseCase {
    pub fn new(repo: Arc<dyn OfflineProfileRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<OfflineProfile, LauncherError> {
        self.repo.get_active().await
    }
}

pub struct ListOfflineProfilesUseCase {
    repo: Arc<dyn OfflineProfileRepository>,
}

impl ListOfflineProfilesUseCase {
    pub fn new(repo: Arc<dyn OfflineProfileRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<OfflineProfile>, LauncherError> {
        self.repo.list().await
    }
}

pub struct CreateOfflineProfileUseCase {
    repo: Arc<dyn OfflineProfileRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl CreateOfflineProfileUseCase {
    pub fn new(repo: Arc<dyn OfflineProfileRepository>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo, settings_repo }
    }

    pub async fn execute(&self, username: String) -> Result<OfflineProfile, LauncherError> {
        let trimmed = username.trim();

        if trimmed.len() < 3 || trimmed.len() > 16 {
            return Err(LauncherError::validation(
                "Minecraft username must be between 3 and 16 characters long",
            ));
        }

        let valid_chars = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !valid_chars.is_match(trimmed) {
            return Err(LauncherError::validation(
                "Minecraft username can only contain letters, numbers, and underscores (_)",
            ));
        }

        let profile = OfflineProfile::new(trimmed);
        self.repo.save(&profile, true).await?;

        let mut settings = self.settings_repo.get().await?;
        settings.active_auth_mode = AccountType::Offline;
        self.settings_repo.save(&settings).await?;

        Ok(profile)
    }
}

pub struct SelectOfflineProfileUseCase {
    repo: Arc<dyn OfflineProfileRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl SelectOfflineProfileUseCase {
    pub fn new(repo: Arc<dyn OfflineProfileRepository>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo, settings_repo }
    }

    pub async fn execute(&self, username: String) -> Result<OfflineProfile, LauncherError> {
        let profile = self.repo.set_active(username.trim()).await?;

        let mut settings = self.settings_repo.get().await?;
        settings.active_auth_mode = AccountType::Offline;
        self.settings_repo.save(&settings).await?;

        Ok(profile)
    }
}

pub struct DeleteOfflineProfileUseCase {
    repo: Arc<dyn OfflineProfileRepository>,
}

impl DeleteOfflineProfileUseCase {
    pub fn new(repo: Arc<dyn OfflineProfileRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, username: String) -> Result<(), LauncherError> {
        self.repo.delete(username.trim()).await
    }
}

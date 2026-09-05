use std::sync::Arc;
use crate::domain::entities::{AccountType, MinecraftAccount, StoredMicrosoftAccount};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::SettingsRepository;
use crate::infrastructure::auth::microsoft_auth::{DeviceCodeInfo, MicrosoftAuthService};

pub struct BeginMicrosoftLoginUseCase {
    service: Arc<MicrosoftAuthService>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl BeginMicrosoftLoginUseCase {
    pub fn new(service: Arc<MicrosoftAuthService>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { service, settings_repo }
    }

    pub async fn execute(&self) -> Result<DeviceCodeInfo, LauncherError> {
        let settings = self.settings_repo.get().await?;
        let client_id = settings.microsoft_client_id.ok_or_else(|| {
            LauncherError::validation(
                "Set a Microsoft Application Client ID in Settings first (register a free app at portal.azure.com).",
            )
        })?;
        self.service.begin_login(&client_id).await
    }
}

pub struct CompleteMicrosoftLoginUseCase {
    service: Arc<MicrosoftAuthService>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl CompleteMicrosoftLoginUseCase {
    pub fn new(service: Arc<MicrosoftAuthService>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { service, settings_repo }
    }

    pub async fn execute(&self, device_code: String, interval: u64, expires_in: u64) -> Result<MinecraftAccount, LauncherError> {
        let mut settings = self.settings_repo.get().await?;
        let client_id = settings.microsoft_client_id.clone().ok_or_else(|| {
            LauncherError::validation("Microsoft Client ID is not configured in Settings.")
        })?;

        let account = self.service.complete_login(&client_id, &device_code, interval, expires_in).await?;

        settings.active_auth_mode = AccountType::Microsoft;
        self.settings_repo.save(&settings).await?;

        Ok(account)
    }
}

pub struct GetMicrosoftAccountUseCase {
    service: Arc<MicrosoftAuthService>,
}

impl GetMicrosoftAccountUseCase {
    pub fn new(service: Arc<MicrosoftAuthService>) -> Self {
        Self { service }
    }

    pub async fn execute(&self) -> Result<Option<StoredMicrosoftAccount>, LauncherError> {
        self.service.get_stored_account().await
    }
}

/// Switches the active account mode back to an already signed-in Microsoft
/// account without repeating the browser sign-in flow (e.g. after the user had
/// switched to an offline profile and now wants to go back).
pub struct ActivateMicrosoftAccountUseCase {
    service: Arc<MicrosoftAuthService>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl ActivateMicrosoftAccountUseCase {
    pub fn new(service: Arc<MicrosoftAuthService>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { service, settings_repo }
    }

    pub async fn execute(&self) -> Result<(), LauncherError> {
        self.service.get_stored_account().await?.ok_or_else(|| {
            LauncherError::validation("No Microsoft account is signed in yet.")
        })?;

        let mut settings = self.settings_repo.get().await?;
        settings.active_auth_mode = AccountType::Microsoft;
        self.settings_repo.save(&settings).await?;
        Ok(())
    }
}

pub struct SignOutMicrosoftUseCase {
    service: Arc<MicrosoftAuthService>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl SignOutMicrosoftUseCase {
    pub fn new(service: Arc<MicrosoftAuthService>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { service, settings_repo }
    }

    pub async fn execute(&self) -> Result<(), LauncherError> {
        self.service.sign_out().await?;

        let mut settings = self.settings_repo.get().await?;
        if settings.active_auth_mode == AccountType::Microsoft {
            settings.active_auth_mode = AccountType::Offline;
            self.settings_repo.save(&settings).await?;
        }

        Ok(())
    }
}

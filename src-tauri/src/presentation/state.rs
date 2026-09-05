use std::sync::Arc;
use crate::application::use_cases::{
    ActivateMicrosoftAccountUseCase, BeginMicrosoftLoginUseCase, CompleteMicrosoftLoginUseCase, CreateInstanceUseCase,
    CreateOfflineProfileUseCase, DeleteInstanceUseCase, DeleteOfflineProfileUseCase,
    DetectJavaUseCase, FetchMinecraftVersionsUseCase, GetActiveOfflineProfileUseCase,
    GetForgeVersionsUseCase, GetInstanceUseCase, GetMicrosoftAccountUseCase, GetSettingsUseCase,
    InstallInstanceUseCase, LaunchInstanceUseCase, ListInstancesUseCase,
    ListOfflineProfilesUseCase, SelectOfflineProfileUseCase, SignOutMicrosoftUseCase,
    UpdateInstanceUseCase, UpdateSettingsUseCase,
};
use crate::application::modpacks::use_cases::{
    CancelModpackOperationUseCase, CheckAllInstalledModpackUpdatesUseCase,
    CheckModpackUpdatesUseCase, GetAvailableModpacksUseCase, GetModpackDetailsUseCase,
    GetModpackUpdateHistoryUseCase, InstallModpackUseCase, PrepareModpackUpdateUseCase,
    RepairModpackUseCase, UpdateModpackUseCase, VerifyModpackUseCase,
};
use crate::domain::modpacks::InstalledModpackRepository;
use crate::infrastructure::logging::LogBuffer;
use crate::shared::config::LauncherPaths;

pub struct AppState {
    pub create_instance_uc: Arc<CreateInstanceUseCase>,
    pub list_instances_uc: Arc<ListInstancesUseCase>,
    pub get_instance_uc: Arc<GetInstanceUseCase>,
    pub update_instance_uc: Arc<UpdateInstanceUseCase>,
    pub delete_instance_uc: Arc<DeleteInstanceUseCase>,
    pub fetch_versions_uc: Arc<FetchMinecraftVersionsUseCase>,
    pub install_instance_uc: Arc<InstallInstanceUseCase>,
    pub get_forge_versions_uc: Arc<GetForgeVersionsUseCase>,
    pub launch_instance_uc: Arc<LaunchInstanceUseCase>,
    pub detect_java_uc: Arc<DetectJavaUseCase>,
    pub get_settings_uc: Arc<GetSettingsUseCase>,
    pub update_settings_uc: Arc<UpdateSettingsUseCase>,
    pub get_active_offline_profile_uc: Arc<GetActiveOfflineProfileUseCase>,
    pub list_offline_profiles_uc: Arc<ListOfflineProfilesUseCase>,
    pub create_offline_profile_uc: Arc<CreateOfflineProfileUseCase>,
    pub select_offline_profile_uc: Arc<SelectOfflineProfileUseCase>,
    pub delete_offline_profile_uc: Arc<DeleteOfflineProfileUseCase>,
    pub begin_microsoft_login_uc: Arc<BeginMicrosoftLoginUseCase>,
    pub complete_microsoft_login_uc: Arc<CompleteMicrosoftLoginUseCase>,
    pub get_microsoft_account_uc: Arc<GetMicrosoftAccountUseCase>,
    pub sign_out_microsoft_uc: Arc<SignOutMicrosoftUseCase>,
    pub activate_microsoft_account_uc: Arc<ActivateMicrosoftAccountUseCase>,
    pub get_available_modpacks_uc: Arc<GetAvailableModpacksUseCase>,
    pub get_modpack_details_uc: Arc<GetModpackDetailsUseCase>,
    pub check_modpack_updates_uc: Arc<CheckModpackUpdatesUseCase>,
    pub check_all_installed_modpack_updates_uc: Arc<CheckAllInstalledModpackUpdatesUseCase>,
    pub prepare_modpack_update_uc: Arc<PrepareModpackUpdateUseCase>,
    pub install_modpack_uc: Arc<InstallModpackUseCase>,
    pub update_modpack_uc: Arc<UpdateModpackUseCase>,
    pub verify_modpack_uc: Arc<VerifyModpackUseCase>,
    pub repair_modpack_uc: Arc<RepairModpackUseCase>,
    pub cancel_modpack_operation_uc: Arc<CancelModpackOperationUseCase>,
    pub get_modpack_update_history_uc: Arc<GetModpackUpdateHistoryUseCase>,
    pub installed_modpack_repo: Arc<dyn InstalledModpackRepository>,
    pub log_buffer: LogBuffer,
    pub paths: LauncherPaths,
}
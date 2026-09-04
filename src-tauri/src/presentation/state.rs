use std::sync::Arc;
use crate::application::use_cases::{
    CreateInstanceUseCase, CreateOfflineProfileUseCase, DeleteInstanceUseCase,
    DeleteOfflineProfileUseCase, DetectJavaUseCase, FetchMinecraftVersionsUseCase,
    GetActiveOfflineProfileUseCase, GetInstanceUseCase, GetSettingsUseCase,
    InstallInstanceUseCase, LaunchInstanceUseCase, ListInstancesUseCase,
    ListOfflineProfilesUseCase, SelectOfflineProfileUseCase, UpdateInstanceUseCase,
    UpdateSettingsUseCase,
};
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
    pub launch_instance_uc: Arc<LaunchInstanceUseCase>,
    pub detect_java_uc: Arc<DetectJavaUseCase>,
    pub get_settings_uc: Arc<GetSettingsUseCase>,
    pub update_settings_uc: Arc<UpdateSettingsUseCase>,
    pub get_active_offline_profile_uc: Arc<GetActiveOfflineProfileUseCase>,
    pub list_offline_profiles_uc: Arc<ListOfflineProfilesUseCase>,
    pub create_offline_profile_uc: Arc<CreateOfflineProfileUseCase>,
    pub select_offline_profile_uc: Arc<SelectOfflineProfileUseCase>,
    pub delete_offline_profile_uc: Arc<DeleteOfflineProfileUseCase>,
    pub log_buffer: LogBuffer,
    pub paths: LauncherPaths,
}
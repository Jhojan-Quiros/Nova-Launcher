use std::sync::Arc;
use crate::application::use_cases::{
    CreateInstanceUseCase, DeleteInstanceUseCase, DetectJavaUseCase,
    FetchMinecraftVersionsUseCase, GetInstanceUseCase, GetSettingsUseCase,
    InstallInstanceUseCase, LaunchInstanceUseCase, ListInstancesUseCase,
    UpdateInstanceUseCase, UpdateSettingsUseCase,
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
    pub log_buffer: LogBuffer,
    pub paths: LauncherPaths,
}
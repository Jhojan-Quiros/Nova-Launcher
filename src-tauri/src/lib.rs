pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::application::ports::AuthenticationProviderPort;
use crate::application::use_cases::{
    CreateInstanceUseCase, CreateOfflineProfileUseCase, DeleteInstanceUseCase,
    DeleteOfflineProfileUseCase, DetectJavaUseCase, FetchMinecraftVersionsUseCase,
    GetActiveOfflineProfileUseCase, GetInstanceUseCase, GetSettingsUseCase,
    InstallInstanceUseCase, LaunchInstanceUseCase, ListInstancesUseCase,
    ListOfflineProfilesUseCase, SelectOfflineProfileUseCase, UpdateInstanceUseCase,
    UpdateSettingsUseCase,
};
use crate::infrastructure::auth::OfflineAuthenticationProvider;
use crate::infrastructure::downloads::manager::DownloadManager;
use crate::infrastructure::java::detector::JavaDetector;
use crate::infrastructure::logging::logger::{LauncherLogger, LogBuffer};
use crate::infrastructure::minecraft::installer::MinecraftInstaller;
use crate::infrastructure::minecraft::launcher::MinecraftLauncherService;
use crate::infrastructure::minecraft::manifest_client::MinecraftVersionManifestClient;
use crate::infrastructure::persistence::{
    DatabaseManager, SqliteInstanceRepository, SqliteOfflineProfileRepository,
    SqliteSettingsRepository,
};
use crate::presentation::commands::*;
use crate::presentation::state::AppState;
use crate::shared::config::LauncherPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = LauncherPaths::default_path();
    let _ = paths.ensure_directories();

    let db_pool = match DatabaseManager::init(&paths.database_file()) {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            DatabaseManager::init_in_memory().expect("Failed to initialize in-memory DB fallback")
        }
    };

    let instance_repo = Arc::new(SqliteInstanceRepository::new(db_pool.clone()));
    let settings_repo = Arc::new(SqliteSettingsRepository::new(db_pool.clone()));
    let profile_repo = Arc::new(SqliteOfflineProfileRepository::new(db_pool.clone()));
    let manifest_client = Arc::new(MinecraftVersionManifestClient::new(paths.clone()));
    let java_detector = Arc::new(JavaDetector::new());
    let auth_provider: Arc<dyn AuthenticationProviderPort> = Arc::new(OfflineAuthenticationProvider::new(profile_repo.clone()));
    let log_buffer = LogBuffer::new(2000);

    let log_buffer_for_events = log_buffer.clone();
    let logs_dir_for_file = paths.logs_dir();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Progress callback for download manager -> throttled to avoid IPC saturation
            let app_handle_dl = app_handle.clone();
            let last_emit_ms = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let dl_progress_cb = Arc::new(move |progress: crate::domain::entities::DownloadProgress| {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let last = last_emit_ms.load(std::sync::atomic::Ordering::Relaxed);
                if progress.percentage >= 100.0 || now_ms.saturating_sub(last) >= 80 {
                    last_emit_ms.store(now_ms, std::sync::atomic::Ordering::Relaxed);
                    let _ = app_handle_dl.emit("download-progress", &progress);
                }
            });

            let download_manager = Arc::new(DownloadManager::new(8, Some(dl_progress_cb)));

            let installer = Arc::new(MinecraftInstaller::new(
                manifest_client.clone(),
                download_manager.clone(),
                instance_repo.clone(),
                paths.clone(),
            ));

            // Log listener for game process -> emits to frontend and logs to file/buffer
            let app_handle_log = app_handle.clone();
            let log_buf_inst = log_buffer_for_events.clone();
            let logs_dir_inst = logs_dir_for_file.clone();

            let game_log_listener = Arc::new(move |inst_id: &str, level: &str, message: &str| {
                log_buf_inst.push(level, inst_id, message);
                LauncherLogger::append_to_file(&logs_dir_inst, level, inst_id, message);

                #[derive(serde::Serialize, Clone)]
                struct GameLogPayload<'a> {
                    instance_id: &'a str,
                    level: &'a str,
                    message: &'a str,
                }

                let _ = app_handle_log.emit("game-log", GameLogPayload {
                    instance_id: inst_id,
                    level,
                    message,
                });
            });

            let launcher_service = Arc::new(MinecraftLauncherService::new(
                instance_repo.clone(),
                settings_repo.clone(),
                auth_provider.clone(),
                java_detector.clone(),
                paths.clone(),
                Some(game_log_listener),
            ));

            let profile_repo_for_state = profile_repo.clone();

            let state = AppState {
                create_instance_uc: Arc::new(CreateInstanceUseCase::new(instance_repo.clone(), settings_repo.clone(), paths.clone())),
                list_instances_uc: Arc::new(ListInstancesUseCase::new(instance_repo.clone())),
                get_instance_uc: Arc::new(GetInstanceUseCase::new(instance_repo.clone())),
                update_instance_uc: Arc::new(UpdateInstanceUseCase::new(instance_repo.clone(), paths.clone())),
                delete_instance_uc: Arc::new(DeleteInstanceUseCase::new(instance_repo.clone(), paths.clone())),
                fetch_versions_uc: Arc::new(FetchMinecraftVersionsUseCase::new(manifest_client.clone())),
                install_instance_uc: Arc::new(InstallInstanceUseCase::new(installer.clone())),
                launch_instance_uc: Arc::new(LaunchInstanceUseCase::new(instance_repo.clone(), launcher_service.clone())),
                detect_java_uc: Arc::new(DetectJavaUseCase::new(java_detector.clone())),
                get_settings_uc: Arc::new(GetSettingsUseCase::new(settings_repo.clone())),
                update_settings_uc: Arc::new(UpdateSettingsUseCase::new(settings_repo.clone())),
                get_active_offline_profile_uc: Arc::new(GetActiveOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                list_offline_profiles_uc: Arc::new(ListOfflineProfilesUseCase::new(profile_repo_for_state.clone())),
                create_offline_profile_uc: Arc::new(CreateOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                select_offline_profile_uc: Arc::new(SelectOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                delete_offline_profile_uc: Arc::new(DeleteOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                log_buffer: log_buffer_for_events.clone(),
                paths: paths.clone(),
            };

            app.manage(state);

            // Log startup
            log_buffer_for_events.push("INFO", "Launcher", "Nova Launcher backend initialized successfully");
            LauncherLogger::append_to_file(&logs_dir_for_file, "INFO", "Launcher", "Nova Launcher backend initialized successfully");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_instance,
            list_instances,
            get_instance,
            update_instance,
            delete_instance,
            get_minecraft_versions,
            install_instance,
            launch_instance,
            detect_java,
            probe_java,
            get_settings,
            update_settings,
            get_logs,
            clear_logs,
            open_folder,
            get_active_offline_profile,
            list_offline_profiles,
            create_offline_profile,
            select_offline_profile,
            delete_offline_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
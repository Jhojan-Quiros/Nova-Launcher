pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::application::ports::AuthenticationProviderPort;
use crate::application::use_cases::{
    CreateInstanceUseCase, DeleteInstanceUseCase, DetectJavaUseCase,
    FetchMinecraftVersionsUseCase, GetInstanceUseCase, GetSettingsUseCase,
    InstallInstanceUseCase, LaunchInstanceUseCase, ListInstancesUseCase,
    UpdateInstanceUseCase, UpdateSettingsUseCase,
};
use crate::infrastructure::auth::DevOfflineAuthenticationProvider;
use crate::infrastructure::downloads::manager::DownloadManager;
use crate::infrastructure::java::detector::JavaDetector;
use crate::infrastructure::logging::logger::{LauncherLogger, LogBuffer};
use crate::infrastructure::minecraft::installer::MinecraftInstaller;
use crate::infrastructure::minecraft::launcher::MinecraftLauncherService;
use crate::infrastructure::minecraft::manifest_client::MinecraftVersionManifestClient;
use crate::infrastructure::persistence::{DatabaseManager, SqliteInstanceRepository, SqliteSettingsRepository};
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
    let manifest_client = Arc::new(MinecraftVersionManifestClient::new(paths.clone()));
    let java_detector = Arc::new(JavaDetector::new());
    let auth_provider: Arc<dyn AuthenticationProviderPort> = Arc::new(DevOfflineAuthenticationProvider::default());
    let log_buffer = LogBuffer::new(2000);

    let log_buffer_for_events = log_buffer.clone();
    let logs_dir_for_file = paths.logs_dir();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Progress callback for download manager -> emits to frontend
            let app_handle_dl = app_handle.clone();
            let dl_progress_cb = Arc::new(move |progress: crate::domain::entities::DownloadProgress| {
                let _ = app_handle_dl.emit("download-progress", &progress);
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
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
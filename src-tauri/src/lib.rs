pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::application::ports::{AuthenticationProviderPort, ModLoaderInstallerPort};
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
    GetModpackUpdateHistoryUseCase, InstallModpackUseCase, ModpackCancellationManager,
    PrepareModpackUpdateUseCase, RepairModpackUseCase, UpdateModpackUseCase,
    VerifyModpackUseCase,
};
use crate::domain::entities::ModLoader;
use crate::domain::modpacks::{InstalledModpackRepository, RemoteModpackRepository};
use crate::infrastructure::auth::{
    CompositeAuthenticationProvider, MicrosoftAuthService, MicrosoftAuthenticationProvider,
    OfflineAuthenticationProvider,
};
use crate::infrastructure::downloads::manager::DownloadManager;
use crate::infrastructure::java::detector::JavaDetector;
use crate::infrastructure::logging::logger::{LauncherLogger, LogBuffer};
use crate::infrastructure::minecraft::forge_installer::ForgeInstaller;
use crate::infrastructure::minecraft::installer::{MinecraftInstaller, VanillaInstaller};
use crate::infrastructure::minecraft::launcher::MinecraftLauncherService;
use crate::infrastructure::minecraft::manifest_client::MinecraftVersionManifestClient;
use crate::infrastructure::modpacks::{HttpModpackProvider, SqliteInstalledModpackRepository};
use crate::infrastructure::persistence::{
    DatabaseManager, SqliteInstanceRepository, SqliteMicrosoftAccountRepository,
    SqliteOfflineProfileRepository, SqliteSettingsRepository,
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
    let microsoft_account_repo = Arc::new(SqliteMicrosoftAccountRepository::new(db_pool.clone()));
    let installed_modpack_repo: Arc<dyn InstalledModpackRepository> =
        Arc::new(SqliteInstalledModpackRepository::new(db_pool.clone()));
    let remote_modpack_repo: Arc<dyn RemoteModpackRepository> =
        Arc::new(HttpModpackProvider::new());
    let modpack_cancellations = ModpackCancellationManager::new();

    let manifest_client = Arc::new(MinecraftVersionManifestClient::new(paths.clone()));
    let java_detector = Arc::new(JavaDetector::new());

    let offline_auth_provider: Arc<dyn AuthenticationProviderPort> =
        Arc::new(OfflineAuthenticationProvider::new(profile_repo.clone()));
    let microsoft_auth_service = Arc::new(MicrosoftAuthService::new(microsoft_account_repo.clone()));
    let microsoft_auth_provider: Arc<dyn AuthenticationProviderPort> = Arc::new(
        MicrosoftAuthenticationProvider::new(microsoft_auth_service.clone(), settings_repo.clone()),
    );
    let auth_provider: Arc<dyn AuthenticationProviderPort> = Arc::new(CompositeAuthenticationProvider::new(
        settings_repo.clone(),
        offline_auth_provider,
        microsoft_auth_provider,
    ));
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

            // Log listener shared by installers and the launcher -> emits to the
            // frontend's Logs tab and appends to the launcher.log file, so install
            // and launch failures are visible even before/without a game process.
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

            let installer = Arc::new(MinecraftInstaller::new(
                manifest_client.clone(),
                download_manager.clone(),
                paths.clone(),
                Some(game_log_listener.clone()),
            ));

            let forge_installer = Arc::new(ForgeInstaller::new(
                download_manager.clone(),
                java_detector.clone(),
                installer.clone(),
                paths.clone(),
                Some(game_log_listener.clone()),
            ));

            let mut mod_loader_installers: HashMap<ModLoader, Arc<dyn ModLoaderInstallerPort>> = HashMap::new();
            mod_loader_installers.insert(ModLoader::Vanilla, Arc::new(VanillaInstaller::new(installer.clone())));
            mod_loader_installers.insert(ModLoader::Forge, forge_installer.clone());

            let launcher_service = Arc::new(MinecraftLauncherService::new(
                instance_repo.clone(),
                settings_repo.clone(),
                auth_provider.clone(),
                java_detector.clone(),
                paths.clone(),
                Some(game_log_listener),
            ));

            let profile_repo_for_state = profile_repo.clone();

            let create_instance_uc = Arc::new(CreateInstanceUseCase::new(
                instance_repo.clone(),
                settings_repo.clone(),
                paths.clone(),
            ));

            let get_available_modpacks_uc = Arc::new(GetAvailableModpacksUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                settings_repo.clone(),
            ));
            let get_modpack_details_uc = Arc::new(GetModpackDetailsUseCase::new(remote_modpack_repo.clone()));
            let check_modpack_updates_uc = Arc::new(CheckModpackUpdatesUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
            ));
            let check_all_installed_modpack_updates_uc = Arc::new(CheckAllInstalledModpackUpdatesUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
            ));
            let prepare_modpack_update_uc = Arc::new(PrepareModpackUpdateUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                instance_repo.clone(),
            ));
            let install_modpack_uc = Arc::new(InstallModpackUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                instance_repo.clone(),
                create_instance_uc.clone(),
                paths.clone(),
                modpack_cancellations.clone(),
            ));
            let update_modpack_uc = Arc::new(UpdateModpackUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                instance_repo.clone(),
                paths.clone(),
                modpack_cancellations.clone(),
            ));
            let verify_modpack_uc = Arc::new(VerifyModpackUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                instance_repo.clone(),
            ));
            let repair_modpack_uc = Arc::new(RepairModpackUseCase::new(
                remote_modpack_repo.clone(),
                installed_modpack_repo.clone(),
                instance_repo.clone(),
                paths.clone(),
                modpack_cancellations.clone(),
            ));
            let cancel_modpack_operation_uc = Arc::new(CancelModpackOperationUseCase::new(modpack_cancellations.clone()));
            let get_modpack_update_history_uc = Arc::new(GetModpackUpdateHistoryUseCase::new(installed_modpack_repo.clone()));

            let state = AppState {
                create_instance_uc,
                list_instances_uc: Arc::new(ListInstancesUseCase::new(instance_repo.clone())),
                get_instance_uc: Arc::new(GetInstanceUseCase::new(instance_repo.clone())),
                update_instance_uc: Arc::new(UpdateInstanceUseCase::new(instance_repo.clone(), paths.clone())),
                delete_instance_uc: Arc::new(DeleteInstanceUseCase::new(instance_repo.clone(), paths.clone())),
                fetch_versions_uc: Arc::new(FetchMinecraftVersionsUseCase::new(manifest_client.clone())),
                install_instance_uc: Arc::new(InstallInstanceUseCase::new(instance_repo.clone(), mod_loader_installers)),
                get_forge_versions_uc: Arc::new(GetForgeVersionsUseCase::new(forge_installer.clone())),
                launch_instance_uc: Arc::new(LaunchInstanceUseCase::new(instance_repo.clone(), launcher_service.clone())),
                detect_java_uc: Arc::new(DetectJavaUseCase::new(java_detector.clone())),
                get_settings_uc: Arc::new(GetSettingsUseCase::new(settings_repo.clone())),
                update_settings_uc: Arc::new(UpdateSettingsUseCase::new(settings_repo.clone())),
                get_active_offline_profile_uc: Arc::new(GetActiveOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                list_offline_profiles_uc: Arc::new(ListOfflineProfilesUseCase::new(profile_repo_for_state.clone())),
                create_offline_profile_uc: Arc::new(CreateOfflineProfileUseCase::new(profile_repo_for_state.clone(), settings_repo.clone())),
                select_offline_profile_uc: Arc::new(SelectOfflineProfileUseCase::new(profile_repo_for_state.clone(), settings_repo.clone())),
                delete_offline_profile_uc: Arc::new(DeleteOfflineProfileUseCase::new(profile_repo_for_state.clone())),
                begin_microsoft_login_uc: Arc::new(BeginMicrosoftLoginUseCase::new(microsoft_auth_service.clone(), settings_repo.clone())),
                complete_microsoft_login_uc: Arc::new(CompleteMicrosoftLoginUseCase::new(microsoft_auth_service.clone(), settings_repo.clone())),
                get_microsoft_account_uc: Arc::new(GetMicrosoftAccountUseCase::new(microsoft_auth_service.clone())),
                sign_out_microsoft_uc: Arc::new(SignOutMicrosoftUseCase::new(microsoft_auth_service.clone(), settings_repo.clone())),
                activate_microsoft_account_uc: Arc::new(ActivateMicrosoftAccountUseCase::new(microsoft_auth_service.clone(), settings_repo.clone())),
                get_available_modpacks_uc,
                get_modpack_details_uc,
                check_modpack_updates_uc,
                check_all_installed_modpack_updates_uc: check_all_installed_modpack_updates_uc.clone(),
                prepare_modpack_update_uc,
                install_modpack_uc,
                update_modpack_uc,
                verify_modpack_uc,
                repair_modpack_uc,
                cancel_modpack_operation_uc,
                get_modpack_update_history_uc,
                installed_modpack_repo,
                log_buffer: log_buffer_for_events.clone(),
                paths: paths.clone(),
            };

            app.manage(state);

            // Startup background check for modpack updates
            let check_updates_task_uc = check_all_installed_modpack_updates_uc.clone();
            let app_handle_updates = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                if let Ok(count) = check_updates_task_uc.execute().await {
                    if count > 0 {
                        let _ = app_handle_updates.emit("modpack-updates-found", count);
                    }
                }
            });

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
            get_forge_versions,
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
            delete_offline_profile,
            begin_microsoft_login,
            complete_microsoft_login,
            get_microsoft_account,
            sign_out_microsoft,
            activate_microsoft_account,
            open_external_url,
            get_modpack_catalog,
            get_modpack_details,
            check_modpack_update,
            check_all_modpack_updates,
            prepare_modpack_update,
            install_modpack,
            update_modpack,
            verify_modpack,
            repair_modpack,
            cancel_modpack_operation,
            get_modpack_update_history,
            get_installed_modpack,
            list_installed_modpacks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
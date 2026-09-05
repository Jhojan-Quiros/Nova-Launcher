use chrono::Utc;
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::application::dto::CreateInstanceDto;
use crate::application::modpacks::dto::{CatalogItemWithStatusDto, InstallModpackDto};
use crate::application::use_cases::CreateInstanceUseCase;
use crate::domain::entities::InstanceStatus;
use crate::domain::modpacks::*;
use crate::domain::repositories::{InstanceRepository, SettingsRepository};
use crate::infrastructure::modpacks::{
    ModpackFileVerifier, ModpackProgressCallback, ModpackUpdatePlanner,
    ModpackUpdateTransaction,
};
use crate::shared::config::LauncherPaths;


#[derive(Clone)]
pub struct ModpackCancellationManager {
    cancellations: Arc<RwLock<HashMap<String, Arc<AtomicBool>>>>,
}

impl ModpackCancellationManager {
    pub fn new() -> Self {
        Self {
            cancellations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, instance_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        let mut map = self.cancellations.write().await;
        map.insert(instance_id.to_string(), flag.clone());
        flag
    }

    pub async fn cancel(&self, instance_id: &str) -> bool {
        let map = self.cancellations.read().await;
        if let Some(flag) = map.get(instance_id) {
            flag.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub async fn unregister(&self, instance_id: &str) {
        let mut map = self.cancellations.write().await;
        map.remove(instance_id);
    }
}

// 1. Get Available Modpacks Use Case
pub struct GetAvailableModpacksUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl GetAvailableModpacksUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            settings_repo,
        }
    }

    pub async fn execute(&self, force_refresh: bool) -> Result<Vec<CatalogItemWithStatusDto>, ModpackError> {
        let settings = self.settings_repo.get().await.map_err(|e| ModpackError::DatabaseError(e.to_string()))?;
        let catalog_url = settings.modpack_catalog_url
            .filter(|u| !u.trim().is_empty() && !u.contains("example.com"))
            .unwrap_or_else(|| "https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/catalog.json".to_string());

        let catalog = self.remote_repo.get_catalog(&catalog_url, force_refresh).await?;
        let installed_list = self.installed_repo.list_all().await?;

        let mut installed_by_pack_id: HashMap<String, InstalledModpack> = HashMap::new();
        for inst in installed_list {
            installed_by_pack_id.insert(inst.modpack_id.clone(), inst);
        }

        let mut result = Vec::new();
        for remote in catalog.modpacks {
            let installed = installed_by_pack_id.get(&remote.id).cloned();
            let mut status = ModpackStatus::NotInstalled;
            let mut update_available = false;
            let mut installed_version = None;

            if let Some(ref inst) = installed {
                installed_version = Some(inst.installed_version.clone());
                status = inst.status.clone();
                if ManifestSchemaValidator::is_newer_version(&inst.installed_version, &remote.latest_version) {
                    status = ModpackStatus::UpdateAvailable;
                    update_available = true;
                }
            }

            let associated_instance_id = installed.as_ref().map(|i| i.instance_id.clone());

            result.push(CatalogItemWithStatusDto {
                remote: remote.clone(),
                installed,
                status,
                update_available,
                latest_version: remote.latest_version.clone(),
                installed_version,
                associated_instance_id,
            });
        }

        Ok(result)
    }
}

// 2. Get Modpack Details Use Case
pub struct GetModpackDetailsUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
}

impl GetModpackDetailsUseCase {
    pub fn new(remote_repo: Arc<dyn RemoteModpackRepository>) -> Self {
        Self { remote_repo }
    }

    pub async fn execute(
        &self,
        manifest_url: &str,
    ) -> Result<(ModpackMainManifest, ModpackVersionManifest), ModpackError> {
        let main_manifest = self.remote_repo.get_main_manifest(manifest_url, false).await?;
        let version_manifest = self.remote_repo.get_version_manifest(&main_manifest.manifest_url).await?;
        Ok((main_manifest, version_manifest))
    }
}

// 3. Check Modpack Updates Use Case
pub struct CheckModpackUpdatesUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
}

impl CheckModpackUpdatesUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
        }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<Option<String>, ModpackError> {
        let mut installed = match self.installed_repo.get_by_instance_id(instance_id).await? {
            Some(i) => i,
            None => return Ok(None),
        };

        let main_manifest = self.remote_repo.get_main_manifest(&installed.manifest_url, false).await?;
        installed.latest_known_version = Some(main_manifest.latest_version.clone());

        if ManifestSchemaValidator::is_newer_version(&installed.installed_version, &main_manifest.latest_version) {
            installed.status = ModpackStatus::UpdateAvailable;
            self.installed_repo.save(&installed).await?;
            Ok(Some(main_manifest.latest_version))
        } else {
            if installed.status == ModpackStatus::UpdateAvailable {
                installed.status = ModpackStatus::Installed;
            }
            self.installed_repo.save(&installed).await?;
            Ok(None)
        }
    }
}

// 4. Check All Installed Modpack Updates Use Case
pub struct CheckAllInstalledModpackUpdatesUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
}

impl CheckAllInstalledModpackUpdatesUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
        }
    }

    pub async fn execute(&self) -> Result<usize, ModpackError> {
        let installed_list = self.installed_repo.list_all().await?;
        let mut update_count = 0;

        for mut installed in installed_list {
            if let Ok(main_manifest) = self.remote_repo.get_main_manifest(&installed.manifest_url, false).await {
                installed.latest_known_version = Some(main_manifest.latest_version.clone());
                if ManifestSchemaValidator::is_newer_version(&installed.installed_version, &main_manifest.latest_version) {
                    installed.status = ModpackStatus::UpdateAvailable;
                    update_count += 1;
                } else if installed.status == ModpackStatus::UpdateAvailable {
                    installed.status = ModpackStatus::Installed;
                }
                let _ = self.installed_repo.save(&installed).await;
            }
        }

        Ok(update_count)
    }
}

// 5. Prepare Modpack Update Use Case
pub struct PrepareModpackUpdateUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    instance_repo: Arc<dyn InstanceRepository>,
}

impl PrepareModpackUpdateUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        instance_repo: Arc<dyn InstanceRepository>,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            instance_repo,
        }
    }

    pub async fn execute(
        &self,
        instance_id: &str,
        target_version: Option<&str>,
    ) -> Result<UpdatePlan, ModpackError> {
        let installed = self.installed_repo.get_by_instance_id(instance_id).await?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "NOT_INSTALLED".to_string(),
                message: format!("Instance '{}' is not associated with a modpack.", instance_id),
                details: None,
            })?;

        let instance = self.instance_repo.find_by_id(instance_id).await
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "INSTANCE_NOT_FOUND".to_string(),
                message: format!("Instance '{}' not found.", instance_id),
                details: None,
            })?;

        let main_manifest = self.remote_repo.get_main_manifest(&installed.manifest_url, false).await?;
        let version_url = if let Some(ver) = target_version {
            // Target specific version
            let base = installed.manifest_url.trim_end_matches("/manifest.json");
            format!("{}/versions/{}.json", base, ver)
        } else {
            main_manifest.manifest_url.clone()
        };

        let target_manifest = self.remote_repo.get_version_manifest(&version_url).await?;
        let game_dir = PathBuf::from(&instance.game_directory);

        ModpackUpdatePlanner::plan_update(
            &game_dir,
            Some(&installed.installed_version),
            &target_manifest,
            installed.strict_mode,
        )
    }
}

// 6. Install Modpack Use Case
pub struct InstallModpackUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    instance_repo: Arc<dyn InstanceRepository>,
    create_instance_uc: Arc<CreateInstanceUseCase>,
    paths: LauncherPaths,
    cancellations: ModpackCancellationManager,
    http_client: Client,
}

impl InstallModpackUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        instance_repo: Arc<dyn InstanceRepository>,
        create_instance_uc: Arc<CreateInstanceUseCase>,
        paths: LauncherPaths,
        cancellations: ModpackCancellationManager,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            instance_repo,
            create_instance_uc,
            paths,
            cancellations,
            http_client: Client::builder().user_agent("NovaLauncher/1.0").build().unwrap_or_default(),
        }
    }

    pub async fn execute(
        &self,
        dto: InstallModpackDto,
        progress_cb: Option<ModpackProgressCallback>,
    ) -> Result<InstalledModpack, ModpackError> {
        let pack = &dto.catalog_pack;
        let main_manifest = self.remote_repo.get_main_manifest(&pack.manifest_url, false).await?;
        let version_manifest = self.remote_repo.get_version_manifest(&main_manifest.manifest_url).await?;

        // 1. Create the instance
        let inst_name = dto.instance_name.unwrap_or_else(|| pack.name.clone());
        let create_dto = CreateInstanceDto {
            name: inst_name.clone(),
            minecraft_version: pack.minecraft_version.clone(),
            loader: pack.loader.clone(),
            loader_version: pack.loader_version.clone(),
            min_ram: dto.custom_ram.as_ref().map(|r| r.min_mb),
            max_ram: dto.custom_ram.as_ref().map(|r| r.max_mb),
        };

        let instance = self.create_instance_uc.execute(create_dto).await
            .map_err(|e| ModpackError::UpdateError {
                code: "INSTANCE_CREATION_FAILED".to_string(),
                message: format!("Failed to create instance for modpack: {}", e),
                details: None,
            })?;

        let instance_dir = self.paths.instance_dir(&instance.id);
        let game_dir = PathBuf::from(&instance.game_directory);

        // 2. Plan installation
        let plan = ModpackUpdatePlanner::plan_update(
            &game_dir,
            None,
            &version_manifest,
            dto.strict_mode.unwrap_or(false),
        )?;

        // 3. Register cancellation token
        let cancel_flag = self.cancellations.register(&instance.id).await;

        // 4. Initial history record
        let history_id = Uuid::new_v4().to_string();
        let history_record = ModpackUpdateHistoryRecord {
            id: history_id.clone(),
            instance_id: instance.id.clone(),
            pack_id: pack.id.clone(),
            from_version: "None".to_string(),
            to_version: version_manifest.version.clone(),
            status: "installing".to_string(),
            download_size: plan.total_download_size,
            started_at: Utc::now(),
            completed_at: None,
            error_code: None,
        };
        let _ = self.installed_repo.record_history(&history_record).await;

        // 5. Execute transaction
        let tx_result = ModpackUpdateTransaction::execute(
            &instance_dir,
            &game_dir,
            &instance.id,
            &plan,
            &version_manifest,
            &self.http_client,
            cancel_flag.clone(),
            progress_cb,
        ).await;

        self.cancellations.unregister(&instance.id).await;

        match tx_result {
            Ok(()) => {
                let now = Utc::now();
                let installed = InstalledModpack {
                    instance_id: instance.id.clone(),
                    modpack_id: pack.id.clone(),
                    name: inst_name,
                    installed_version: version_manifest.version.clone(),
                    latest_known_version: Some(version_manifest.version.clone()),
                    manifest_url: pack.manifest_url.clone(),
                    icon_url: pack.icon_url.clone(),
                    banner_url: pack.banner_url.clone(),
                    installed_at: now,
                    updated_at: now,
                    last_verified_at: Some(now),
                    status: ModpackStatus::Installed,
                    strict_mode: dto.strict_mode.unwrap_or(false),
                };

                self.installed_repo.save(&installed).await?;

                // Mark instance ready
                let mut updated_instance = instance.clone();
                updated_instance.set_status(InstanceStatus::Ready);
                let _ = self.instance_repo.save(&updated_instance).await;

                // Update history
                let completed_history = ModpackUpdateHistoryRecord {
                    id: history_id,
                    instance_id: instance.id.clone(),
                    pack_id: pack.id.clone(),
                    from_version: "None".to_string(),
                    to_version: version_manifest.version.clone(),
                    status: "success".to_string(),
                    download_size: plan.total_download_size,
                    started_at: history_record.started_at,
                    completed_at: Some(Utc::now()),
                    error_code: None,
                };
                let _ = self.installed_repo.record_history(&completed_history).await;

                Ok(installed)
            }
            Err(e) => {
                let fail_history = ModpackUpdateHistoryRecord {
                    id: history_id,
                    instance_id: instance.id.clone(),
                    pack_id: pack.id.clone(),
                    from_version: "None".to_string(),
                    to_version: version_manifest.version.clone(),
                    status: "failed".to_string(),
                    download_size: plan.total_download_size,
                    started_at: history_record.started_at,
                    completed_at: Some(Utc::now()),
                    error_code: Some(e.to_payload().code),
                };
                let _ = self.installed_repo.record_history(&fail_history).await;

                Err(e)
            }
        }
    }
}

// 7. Update Modpack Use Case
pub struct UpdateModpackUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    instance_repo: Arc<dyn InstanceRepository>,
    paths: LauncherPaths,
    cancellations: ModpackCancellationManager,
    http_client: Client,
}

impl UpdateModpackUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        instance_repo: Arc<dyn InstanceRepository>,
        paths: LauncherPaths,
        cancellations: ModpackCancellationManager,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            instance_repo,
            paths,
            cancellations,
            http_client: Client::builder().user_agent("NovaLauncher/1.0").build().unwrap_or_default(),
        }
    }

    pub async fn execute(
        &self,
        instance_id: &str,
        target_version: Option<&str>,
        progress_cb: Option<ModpackProgressCallback>,
    ) -> Result<InstalledModpack, ModpackError> {
        let mut installed = self.installed_repo.get_by_instance_id(instance_id).await?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "NOT_INSTALLED".to_string(),
                message: format!("Instance '{}' has no installed modpack.", instance_id),
                details: None,
            })?;

        let instance = self.instance_repo.find_by_id(instance_id).await
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "INSTANCE_NOT_FOUND".to_string(),
                message: format!("Instance '{}' not found.", instance_id),
                details: None,
            })?;

        let main_manifest = self.remote_repo.get_main_manifest(&installed.manifest_url, false).await?;
        let version_url = if let Some(ver) = target_version {
            let base = installed.manifest_url.trim_end_matches("/manifest.json");
            format!("{}/versions/{}.json", base, ver)
        } else {
            main_manifest.manifest_url.clone()
        };

        let target_manifest = self.remote_repo.get_version_manifest(&version_url).await?;

        let instance_dir = self.paths.instance_dir(instance_id);
        let game_dir = PathBuf::from(&instance.game_directory);

        let plan = ModpackUpdatePlanner::plan_update(
            &game_dir,
            Some(&installed.installed_version),
            &target_manifest,
            installed.strict_mode,
        )?;

        // Set status Updating
        installed.status = ModpackStatus::Updating;
        self.installed_repo.save(&installed).await?;

        let cancel_flag = self.cancellations.register(instance_id).await;

        let history_id = Uuid::new_v4().to_string();
        let history_record = ModpackUpdateHistoryRecord {
            id: history_id.clone(),
            instance_id: instance_id.to_string(),
            pack_id: installed.modpack_id.clone(),
            from_version: installed.installed_version.clone(),
            to_version: target_manifest.version.clone(),
            status: "updating".to_string(),
            download_size: plan.total_download_size,
            started_at: Utc::now(),
            completed_at: None,
            error_code: None,
        };
        let _ = self.installed_repo.record_history(&history_record).await;

        let tx_result = ModpackUpdateTransaction::execute(
            &instance_dir,
            &game_dir,
            instance_id,
            &plan,
            &target_manifest,
            &self.http_client,
            cancel_flag.clone(),
            progress_cb,
        ).await;

        self.cancellations.unregister(instance_id).await;

        match tx_result {
            Ok(()) => {
                let now = Utc::now();
                installed.installed_version = target_manifest.version.clone();
                installed.latest_known_version = Some(target_manifest.version.clone());
                installed.updated_at = now;
                installed.last_verified_at = Some(now);
                installed.status = ModpackStatus::Installed;
                self.installed_repo.save(&installed).await?;

                let completed_history = ModpackUpdateHistoryRecord {
                    id: history_id,
                    instance_id: instance_id.to_string(),
                    pack_id: installed.modpack_id.clone(),
                    from_version: history_record.from_version,
                    to_version: target_manifest.version.clone(),
                    status: "success".to_string(),
                    download_size: plan.total_download_size,
                    started_at: history_record.started_at,
                    completed_at: Some(Utc::now()),
                    error_code: None,
                };
                let _ = self.installed_repo.record_history(&completed_history).await;

                Ok(installed)
            }
            Err(e) => {
                installed.status = ModpackStatus::Failed;
                let _ = self.installed_repo.save(&installed).await;

                let fail_history = ModpackUpdateHistoryRecord {
                    id: history_id,
                    instance_id: instance_id.to_string(),
                    pack_id: installed.modpack_id.clone(),
                    from_version: history_record.from_version,
                    to_version: target_manifest.version.clone(),
                    status: "failed".to_string(),
                    download_size: plan.total_download_size,
                    started_at: history_record.started_at,
                    completed_at: Some(Utc::now()),
                    error_code: Some(e.to_payload().code),
                };
                let _ = self.installed_repo.record_history(&fail_history).await;

                Err(e)
            }
        }
    }
}

// 8. Verify Modpack Use Case
pub struct VerifyModpackUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    instance_repo: Arc<dyn InstanceRepository>,
}

impl VerifyModpackUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        instance_repo: Arc<dyn InstanceRepository>,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            instance_repo,
        }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<VerificationResult, ModpackError> {
        let mut installed = self.installed_repo.get_by_instance_id(instance_id).await?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "NOT_INSTALLED".to_string(),
                message: format!("Instance '{}' has no installed modpack.", instance_id),
                details: None,
            })?;

        let instance = self.instance_repo.find_by_id(instance_id).await
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "INSTANCE_NOT_FOUND".to_string(),
                message: format!("Instance '{}' not found.", instance_id),
                details: None,
            })?;

        let base = installed.manifest_url.trim_end_matches("/manifest.json");
        let version_url = format!("{}/versions/{}.json", base, installed.installed_version);
        let version_manifest = self.remote_repo.get_version_manifest(&version_url).await?;

        let game_dir = PathBuf::from(&instance.game_directory);
        let result = ModpackFileVerifier::verify_manifest_files(&game_dir, &version_manifest)?;

        installed.last_verified_at = Some(Utc::now());
        if !result.missing.is_empty() || !result.modified.is_empty() {
            installed.status = ModpackStatus::Corrupted;
        } else if installed.status == ModpackStatus::Corrupted {
            installed.status = ModpackStatus::Installed;
        }
        self.installed_repo.save(&installed).await?;

        Ok(result)
    }
}

// 9. Repair Modpack Use Case
pub struct RepairModpackUseCase {
    remote_repo: Arc<dyn RemoteModpackRepository>,
    installed_repo: Arc<dyn InstalledModpackRepository>,
    instance_repo: Arc<dyn InstanceRepository>,
    paths: LauncherPaths,
    cancellations: ModpackCancellationManager,
    http_client: Client,
}

impl RepairModpackUseCase {
    pub fn new(
        remote_repo: Arc<dyn RemoteModpackRepository>,
        installed_repo: Arc<dyn InstalledModpackRepository>,
        instance_repo: Arc<dyn InstanceRepository>,
        paths: LauncherPaths,
        cancellations: ModpackCancellationManager,
    ) -> Self {
        Self {
            remote_repo,
            installed_repo,
            instance_repo,
            paths,
            cancellations,
            http_client: Client::builder().user_agent("NovaLauncher/1.0").build().unwrap_or_default(),
        }
    }

    pub async fn execute(
        &self,
        instance_id: &str,
        progress_cb: Option<ModpackProgressCallback>,
    ) -> Result<VerificationResult, ModpackError> {
        let mut installed = self.installed_repo.get_by_instance_id(instance_id).await?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "NOT_INSTALLED".to_string(),
                message: format!("Instance '{}' has no installed modpack.", instance_id),
                details: None,
            })?;

        let instance = self.instance_repo.find_by_id(instance_id).await
            .map_err(|e| ModpackError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ModpackError::ManifestError {
                code: "INSTANCE_NOT_FOUND".to_string(),
                message: format!("Instance '{}' not found.", instance_id),
                details: None,
            })?;

        let base = installed.manifest_url.trim_end_matches("/manifest.json");
        let version_url = format!("{}/versions/{}.json", base, installed.installed_version);
        let version_manifest = self.remote_repo.get_version_manifest(&version_url).await?;

        let game_dir = PathBuf::from(&instance.game_directory);
        let verification = ModpackFileVerifier::verify_manifest_files(&game_dir, &version_manifest)?;

        if verification.files_to_repair.is_empty() {
            installed.status = ModpackStatus::Installed;
            self.installed_repo.save(&installed).await?;
            return Ok(verification);
        }

        installed.status = ModpackStatus::Repairing;
        self.installed_repo.save(&installed).await?;

        let plan = UpdatePlan {
            pack_id: installed.modpack_id.clone(),
            from_version: Some(installed.installed_version.clone()),
            to_version: installed.installed_version.clone(),
            downloads: verification.files_to_repair.clone(),
            deletions: Vec::new(),
            unmodified_count: verification.valid,
            total_download_size: verification.repair_size,
            total_files: version_manifest.files.len(),
            changelog: vec!["Integrity repair of missing/corrupted files".to_string()],
        };

        let instance_dir = self.paths.instance_dir(instance_id);
        let cancel_flag = self.cancellations.register(instance_id).await;

        let tx_res = ModpackUpdateTransaction::execute(
            &instance_dir,
            &game_dir,
            instance_id,
            &plan,
            &version_manifest,
            &self.http_client,
            cancel_flag.clone(),
            progress_cb,
        ).await;

        self.cancellations.unregister(instance_id).await;
        tx_res?;

        // Re-verify after repair
        let post_verification = ModpackFileVerifier::verify_manifest_files(&game_dir, &version_manifest)?;
        installed.status = ModpackStatus::Installed;
        installed.last_verified_at = Some(Utc::now());
        self.installed_repo.save(&installed).await?;

        Ok(post_verification)
    }
}

// 10. Cancel Modpack Operation Use Case
pub struct CancelModpackOperationUseCase {
    cancellations: ModpackCancellationManager,
}

impl CancelModpackOperationUseCase {
    pub fn new(cancellations: ModpackCancellationManager) -> Self {
        Self { cancellations }
    }

    pub async fn execute(&self, instance_id: &str) -> bool {
        self.cancellations.cancel(instance_id).await
    }
}

// 11. Get Modpack Update History Use Case
pub struct GetModpackUpdateHistoryUseCase {
    installed_repo: Arc<dyn InstalledModpackRepository>,
}

impl GetModpackUpdateHistoryUseCase {
    pub fn new(installed_repo: Arc<dyn InstalledModpackRepository>) -> Self {
        Self { installed_repo }
    }

    pub async fn execute(&self, instance_id: &str) -> Result<Vec<ModpackUpdateHistoryRecord>, ModpackError> {
        self.installed_repo.get_history(instance_id).await
    }
}

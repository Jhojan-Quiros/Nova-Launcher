use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::domain::modpacks::{
    ModpackDownloadJobProgress, ModpackError, ModpackVersionManifest, UpdatePlan,
};
use super::file_verifier::ModpackFileVerifier;
use super::safe_path_resolver::SafePathResolver;

pub type ModpackProgressCallback = Arc<dyn Fn(ModpackDownloadJobProgress) + Send + Sync>;

pub struct ModpackUpdateTransaction;

impl ModpackUpdateTransaction {
    pub async fn execute(
        instance_dir: &Path,
        game_dir: &Path,
        instance_id: &str,
        plan: &UpdatePlan,
        target_manifest: &ModpackVersionManifest,
        client: &Client,
        cancel_flag: Arc<AtomicBool>,
        progress_cb: Option<ModpackProgressCallback>,
    ) -> Result<(), ModpackError> {
        let update_root = instance_dir.join(".update");
        let staging_dir = update_root.join("staging");
        let backup_dir = update_root.join("backup");

        // Clean previous staging/backup if any
        if update_root.exists() {
            let _ = fs::remove_dir_all(&update_root);
        }
        fs::create_dir_all(&staging_dir).map_err(|e| {
            ModpackError::FilesystemError(format!("Failed to create staging dir: {}", e))
        })?;
        fs::create_dir_all(&backup_dir).map_err(|e| {
            ModpackError::FilesystemError(format!("Failed to create backup dir: {}", e))
        })?;

        let total_bytes = plan.total_download_size;
        let downloaded_bytes = Arc::new(AtomicU64::new(0));
        let total_files = plan.downloads.len();

        // --- 1. DOWNLOAD INTO STAGING ---
        for (idx, item) in plan.downloads.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = fs::remove_dir_all(&update_root);
                return Err(ModpackError::Cancelled);
            }

            let staging_file_path = SafePathResolver::resolve_safe_path(&staging_dir, &item.path)?;
            if let Some(parent) = staging_file_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    ModpackError::FilesystemError(format!("Failed to create staging parent dir: {}", e))
                })?;
            }

            // Report progress
            if let Some(ref cb) = progress_cb {
                let current_dl = downloaded_bytes.load(Ordering::Relaxed);
                let pct = if total_bytes > 0 {
                    ((current_dl as f64) / (total_bytes as f64)) * 100.0
                } else {
                    0.0
                };
                cb(ModpackDownloadJobProgress {
                    instance_id: instance_id.to_string(),
                    pack_id: plan.pack_id.clone(),
                    version: plan.to_version.clone(),
                    downloaded_bytes: current_dl,
                    total_bytes,
                    percentage: pct.min(99.0),
                    current_file: item.file_name.clone(),
                    files_completed: idx,
                    total_files,
                    stage: "downloading".to_string(),
                });
            }

            // Download file
            Self::download_file_with_retry(
                client,
                &item.url,
                &staging_file_path,
                &cancel_flag,
            ).await?;

            // Verify SHA-256 in staging
            let computed = ModpackFileVerifier::compute_sha256(&staging_file_path)?;
            if !computed.eq_ignore_ascii_case(&item.sha256) {
                let _ = fs::remove_dir_all(&update_root);
                return Err(ModpackError::hash_mismatch(&item.path, &item.sha256, &computed));
            }

            downloaded_bytes.fetch_add(item.size, Ordering::Relaxed);
        }

        // Check cancellation before critical commit stage
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = fs::remove_dir_all(&update_root);
            return Err(ModpackError::Cancelled);
        }

        // --- 2. BACKUP EXISTING FILES ---
        if let Some(ref cb) = progress_cb {
            cb(ModpackDownloadJobProgress {
                instance_id: instance_id.to_string(),
                pack_id: plan.pack_id.clone(),
                version: plan.to_version.clone(),
                downloaded_bytes: total_bytes,
                total_bytes,
                percentage: 92.0,
                current_file: "Backing up files...".to_string(),
                files_completed: total_files,
                total_files,
                stage: "applying".to_string(),
            });
        }

        let mut backed_up_overwritten: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut backed_up_deletions: Vec<(PathBuf, PathBuf)> = Vec::new();

        // Backup files to be overwritten
        for item in &plan.downloads {
            let target_path = SafePathResolver::resolve_safe_path(game_dir, &item.path)?;
            if target_path.exists() {
                let backup_target = SafePathResolver::resolve_safe_path(&backup_dir.join("overwritten"), &item.path)?;
                if let Some(parent) = backup_target.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(()) = fs::copy(&target_path, &backup_target).map(|_| ()) {
                    backed_up_overwritten.push((target_path, backup_target));
                }
            }
        }

        // Backup files to be deleted
        for rel_del in &plan.deletions {
            let target_path = SafePathResolver::resolve_safe_path(game_dir, rel_del)?;
            if target_path.exists() {
                let backup_target = SafePathResolver::resolve_safe_path(&backup_dir.join("deleted"), rel_del)?;
                if let Some(parent) = backup_target.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(()) = fs::copy(&target_path, &backup_target).map(|_| ()) {
                    backed_up_deletions.push((target_path, backup_target));
                }
            }
        }

        // --- 3. APPLY STAGED FILES (ATOMIC MOVE) ---
        let mut applied_files: Vec<PathBuf> = Vec::new();

        for item in &plan.downloads {
            let staged_file = match SafePathResolver::resolve_safe_path(&staging_dir, &item.path) {
                Ok(p) => p,
                Err(e) => {
                    Self::rollback(&backed_up_overwritten, &backed_up_deletions, &applied_files);
                    let _ = fs::remove_dir_all(&update_root);
                    return Err(e);
                }
            };
            let target_file = match SafePathResolver::resolve_safe_path(game_dir, &item.path) {
                Ok(p) => p,
                Err(e) => {
                    Self::rollback(&backed_up_overwritten, &backed_up_deletions, &applied_files);
                    let _ = fs::remove_dir_all(&update_root);
                    return Err(e);
                }
            };

            if let Some(parent) = target_file.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    Self::rollback(&backed_up_overwritten, &backed_up_deletions, &applied_files);
                    let _ = fs::remove_dir_all(&update_root);
                    return Err(ModpackError::FilesystemError(format!("Failed to create dir {:?}: {}", parent, e)));
                }
            }

            // Move staged file to target
            if let Err(_) = fs::rename(&staged_file, &target_file) {
                // If cross-device or permission issue, fallback to copy + remove
                if let Err(e) = fs::copy(&staged_file, &target_file).and_then(|_| fs::remove_file(&staged_file)) {
                    Self::rollback(&backed_up_overwritten, &backed_up_deletions, &applied_files);
                    let _ = fs::remove_dir_all(&update_root);
                    return Err(ModpackError::RollbackError {
                        code: "APPLY_FILE_FAILED".to_string(),
                        message: format!("Failed to apply file {}: {}", item.path, e),
                        details: Some(serde_json::json!({ "file": item.path })),
                    });
                }
            }

            applied_files.push(target_file);
        }

        // --- 4. DELETE REMOVED FILES ---
        for rel_del in &plan.deletions {
            if let Ok(target_path) = SafePathResolver::resolve_safe_path(game_dir, rel_del) {
                if target_path.exists() {
                    let _ = fs::remove_file(&target_path);
                }
            }
        }

        // --- 5. WRITE LOCAL MANIFEST AUDIT COPIES ---
        let nova_dir = instance_dir.join(".nova");
        let _ = fs::create_dir_all(&nova_dir);

        let modpack_info = serde_json::json!({
            "packId": plan.pack_id,
            "installedVersion": plan.to_version,
            "installedAt": chrono::Utc::now().to_rfc3339(),
            "updatedAt": chrono::Utc::now().to_rfc3339()
        });
        let _ = fs::write(nova_dir.join("modpack.json"), serde_json::to_string_pretty(&modpack_info).unwrap_or_default());
        let _ = fs::write(
            nova_dir.join("installed-manifest.json"),
            serde_json::to_string_pretty(target_manifest).unwrap_or_default(),
        );

        // --- 6. CLEANUP TEMPORARY FILES ---
        let _ = fs::remove_dir_all(&update_root);

        if let Some(ref cb) = progress_cb {
            cb(ModpackDownloadJobProgress {
                instance_id: instance_id.to_string(),
                pack_id: plan.pack_id.clone(),
                version: plan.to_version.clone(),
                downloaded_bytes: total_bytes,
                total_bytes,
                percentage: 100.0,
                current_file: "Completed".to_string(),
                files_completed: total_files,
                total_files,
                stage: "completed".to_string(),
            });
        }

        Ok(())
    }

    fn rollback(
        overwritten: &[(PathBuf, PathBuf)],
        deletions: &[(PathBuf, PathBuf)],
        applied_new_files: &[PathBuf],
    ) {
        tracing::warn!("Initiating modpack update rollback...");
        // 1. Delete newly created files that were applied
        for new_file in applied_new_files {
            if new_file.exists() {
                let _ = fs::remove_file(new_file);
            }
        }

        // 2. Restore overwritten files from backup
        for (original, backup) in overwritten {
            if backup.exists() {
                let _ = fs::copy(backup, original);
            }
        }

        // 3. Restore deleted files from backup
        for (original, backup) in deletions {
            if backup.exists() {
                let _ = fs::copy(backup, original);
            }
        }
        tracing::info!("Modpack rollback completed successfully.");
    }

    async fn download_file_with_retry(
        client: &Client,
        url: &str,
        destination: &Path,
        cancel_flag: &AtomicBool,
    ) -> Result<(), ModpackError> {
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err(ModpackError::Cancelled);
            }

            attempts += 1;
            match client.get(url).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        if attempts >= max_attempts {
                            return Err(ModpackError::DownloadError {
                                code: format!("HTTP_{}", resp.status().as_u16()),
                                message: format!("HTTP {} downloading {}", resp.status(), url),
                                details: Some(serde_json::json!({ "url": url })),
                            });
                        }
                    } else {
                        match resp.bytes().await {
                            Ok(bytes) => {
                                let mut file = File::create(destination).map_err(|e| {
                                    ModpackError::FilesystemError(format!("Failed to create destination file: {}", e))
                                })?;
                                file.write_all(&bytes).map_err(|e| {
                                    ModpackError::FilesystemError(format!("Failed to write downloaded bytes: {}", e))
                                })?;
                                return Ok(());
                            }
                            Err(e) => {
                                if attempts >= max_attempts {
                                    return Err(ModpackError::DownloadError {
                                        code: "STREAM_ERROR".to_string(),
                                        message: format!("Failed reading response stream for {}: {}", url, e),
                                        details: Some(serde_json::json!({ "url": url })),
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(ModpackError::DownloadError {
                            code: "NETWORK_ERROR".to_string(),
                            message: format!("Network error downloading {}: {}", url, e),
                            details: Some(serde_json::json!({ "url": url })),
                        });
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(400 * attempts as u64)).await;
        }
    }
}

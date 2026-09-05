use std::collections::HashSet;
use std::path::Path;
use crate::domain::modpacks::{
    ManifestFileEntry, ModpackError, ModpackVersionManifest, UpdatePlan,
};
use super::file_verifier::ModpackFileVerifier;
use super::safe_path_resolver::SafePathResolver;

pub struct ModpackUpdatePlanner;

impl ModpackUpdatePlanner {
    pub fn plan_update(
        game_dir: &Path,
        from_version: Option<&str>,
        target_manifest: &ModpackVersionManifest,
        strict_mode: bool,
    ) -> Result<UpdatePlan, ModpackError> {
        let mut downloads: Vec<ManifestFileEntry> = Vec::new();
        let mut deletions: Vec<String> = Vec::new();
        let mut unmodified_count = 0usize;
        let mut total_download_size = 0u64;

        let mut manifest_paths = HashSet::new();

        // 1. Analyze files required by target manifest
        for file in &target_manifest.files {
            manifest_paths.insert(file.path.replace('\\', "/").to_lowercase());

            let local_path = SafePathResolver::resolve_safe_path(game_dir, &file.path)?;

            if local_path.exists() && ModpackFileVerifier::verify_file_sha256(&local_path, &file.sha256) {
                // File exists and checksum is 100% identical -> keep it!
                unmodified_count += 1;
            } else {
                // File is missing or modified -> schedule download
                downloads.push(file.clone());
                total_download_size += file.size;
            }
        }

        // 2. Handle files explicitly marked in removedFiles by the remote manifest
        for removed in &target_manifest.removed_files {
            if SafePathResolver::is_protected_path(removed) {
                // Never delete protected user paths (e.g. saves/, logs/)
                continue;
            }

            if let Ok(local_path) = SafePathResolver::resolve_safe_path(game_dir, removed) {
                if local_path.exists() {
                    deletions.push(removed.clone());
                }
            }
        }

        // 3. If strict_mode is true, remove any unmanaged mod in mods/
        if strict_mode {
            let mods_dir = game_dir.join("mods");
            if mods_dir.exists() && mods_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&mods_dir) {
                    for item in entries.flatten() {
                        if let Ok(ft) = item.file_type() {
                            if ft.is_file() {
                                let filename = item.file_name().to_string_lossy().to_string();
                                let rel_mod = format!("mods/{}", filename);
                                let norm = rel_mod.replace('\\', "/").to_lowercase();
                                if !manifest_paths.contains(&norm) && !deletions.contains(&rel_mod) {
                                    deletions.push(rel_mod);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(UpdatePlan {
            pack_id: target_manifest.pack_id.clone(),
            from_version: from_version.map(|s| s.to_string()),
            to_version: target_manifest.version.clone(),
            downloads,
            deletions,
            unmodified_count,
            total_download_size,
            total_files: target_manifest.files.len(),
            changelog: target_manifest.changelog.clone(),
        })
    }
}

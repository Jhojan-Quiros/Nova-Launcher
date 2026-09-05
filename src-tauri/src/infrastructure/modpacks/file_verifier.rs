use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use crate::domain::modpacks::{ModpackError, ModpackVersionManifest, VerificationResult};

use super::safe_path_resolver::SafePathResolver;


pub struct ModpackFileVerifier;

impl ModpackFileVerifier {
    /// Computes the SHA-256 hex string of a file on disk using buffered streaming
    pub fn compute_sha256(path: &Path) -> Result<String, ModpackError> {
        let file = File::open(path).map_err(|e| {
            ModpackError::FilesystemError(format!("Failed to open file {:?} for SHA-256: {}", path, e))
        })?;

        let mut reader = BufReader::with_capacity(64 * 1024, file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(e) => {
                    return Err(ModpackError::FilesystemError(format!(
                        "Error reading file {:?} during SHA-256: {}",
                        path, e
                    )));
                }
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Checks if a file exists and has the expected SHA-256
    pub fn verify_file_sha256(path: &Path, expected_sha256: &str) -> bool {
        if !path.exists() {
            return false;
        }
        match Self::compute_sha256(path) {
            Ok(actual) => actual.eq_ignore_ascii_case(expected_sha256),
            Err(_) => false,
        }
    }

    /// Verifies all manifest files inside a given instance game directory
    pub fn verify_manifest_files(
        game_dir: &Path,
        manifest: &ModpackVersionManifest,
    ) -> Result<VerificationResult, ModpackError> {
        let mut valid = 0;
        let mut missing = Vec::new();
        let mut modified = Vec::new();
        let mut files_to_repair = Vec::new();
        let mut repair_size = 0u64;

        for entry in &manifest.files {
            let full_path = SafePathResolver::resolve_safe_path(game_dir, &entry.path)?;

            if !full_path.exists() {
                missing.push(entry.path.clone());
                files_to_repair.push(entry.clone());
                repair_size += entry.size;
            } else if Self::verify_file_sha256(&full_path, &entry.sha256) {
                valid += 1;
            } else {
                modified.push(entry.path.clone());
                files_to_repair.push(entry.clone());
                repair_size += entry.size;
            }
        }

        // Check for user-added mods in mods/ folder
        let mut unmanaged_user_files = Vec::new();
        let mods_dir = game_dir.join("mods");
        if mods_dir.exists() && mods_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&mods_dir) {
                let manifest_mod_paths: std::collections::HashSet<String> = manifest
                    .files
                    .iter()
                    .map(|f| f.path.replace('\\', "/").to_lowercase())
                    .collect();

                for item in entries.flatten() {
                    if let Ok(ft) = item.file_type() {
                        if ft.is_file() {
                            let filename = item.file_name().to_string_lossy().to_string();
                            let rel_mod_path = format!("mods/{}", filename).to_lowercase();
                            if !manifest_mod_paths.contains(&rel_mod_path) {
                                unmanaged_user_files.push(format!("mods/{}", filename));
                            }
                        }
                    }
                }
            }
        }

        Ok(VerificationResult {
            total_checked: manifest.files.len(),
            valid,
            missing,
            modified,
            unmanaged_user_files,
            repair_size,
            files_to_repair,
        })
    }
}

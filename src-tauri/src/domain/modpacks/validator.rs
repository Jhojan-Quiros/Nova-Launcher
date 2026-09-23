use semver::Version;
use super::entities::*;
use super::errors::ModpackError;

pub struct ManifestSchemaValidator;

impl ManifestSchemaValidator {
    pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

    pub fn validate_schema_version(version: u32) -> Result<(), ModpackError> {
        if version != Self::SUPPORTED_SCHEMA_VERSION {
            return Err(ModpackError::unsupported_schema(version));
        }
        Ok(())
    }

    pub fn validate_catalog(catalog: &ModpackCatalog) -> Result<(), ModpackError> {
        Self::validate_schema_version(catalog.schema_version)?;
        for pack in &catalog.modpacks {
            if pack.id.trim().is_empty() {
                return Err(ModpackError::ManifestError {
                    code: "INVALID_MODPACK_ID".to_string(),
                    message: "Modpack ID in catalog cannot be empty.".to_string(),
                    details: None,
                });
            }
            if pack.manifest_url.trim().is_empty() {
                return Err(ModpackError::ManifestError {
                    code: "INVALID_MANIFEST_URL".to_string(),
                    message: format!("Manifest URL for modpack '{}' is missing.", pack.id),
                    details: None,
                });
            }
        }
        Ok(())
    }

    pub fn validate_main_manifest(manifest: &ModpackMainManifest) -> Result<(), ModpackError> {
        Self::validate_schema_version(manifest.schema_version)?;
        if manifest.id.trim().is_empty() {
            return Err(ModpackError::ManifestError {
                code: "INVALID_MODPACK_ID".to_string(),
                message: "Modpack ID cannot be empty.".to_string(),
                details: None,
            });
        }
        if manifest.latest_version.trim().is_empty() {
            return Err(ModpackError::ManifestError {
                code: "INVALID_VERSION".to_string(),
                message: "latestVersion in manifest cannot be empty.".to_string(),
                details: None,
            });
        }
        Ok(())
    }

    pub fn validate_version_manifest(manifest: &ModpackVersionManifest) -> Result<(), ModpackError> {
        Self::validate_schema_version(manifest.schema_version)?;
        if manifest.pack_id.trim().is_empty() {
            return Err(ModpackError::ManifestError {
                code: "INVALID_PACK_ID".to_string(),
                message: "packId in version manifest cannot be empty.".to_string(),
                details: None,
            });
        }
        if manifest.version.trim().is_empty() {
            return Err(ModpackError::ManifestError {
                code: "INVALID_VERSION".to_string(),
                message: "version in version manifest cannot be empty.".to_string(),
                details: None,
            });
        }
        // Validate each file entry
        for file in &manifest.files {
            if file.path.trim().is_empty() {
                return Err(ModpackError::ManifestError {
                    code: "INVALID_FILE_PATH".to_string(),
                    message: "Manifest file entry path cannot be empty.".to_string(),
                    details: None,
                });
            }
            if file.sha256.len() != 64 {
                return Err(ModpackError::ManifestError {
                    code: "INVALID_FILE_HASH".to_string(),
                    message: format!("Invalid SHA-256 for file '{}'. Must be 64 hex characters.", file.path),
                    details: Some(serde_json::json!({ "file": file.path, "sha256": file.sha256 })),
                });
            }
            if file.url.trim().is_empty() {
                return Err(ModpackError::ManifestError {
                    code: "INVALID_FILE_URL".to_string(),
                    message: format!("Download URL missing for file '{}'.", file.path),
                    details: None,
                });
            }
        }
        Ok(())
    }

    /// Compares two version strings. Uses semver when valid, falls back to lexicographic comparison.
    /// Returns:
    ///   - true if `latest` is strictly newer than `installed`
    ///   - false if `latest` is equal or older
    pub fn is_newer_version(installed: &str, latest: &str) -> bool {
        let clean_installed = installed.trim_start_matches('v');
        let clean_latest = latest.trim_start_matches('v');

        match (Version::parse(clean_installed), Version::parse(clean_latest)) {
            (Ok(inst_ver), Ok(latest_ver)) => latest_ver > inst_ver,
            _ => {
                // Fallback numeric segments comparison
                Self::fallback_compare_versions(clean_installed, clean_latest) > 0
            }
        }
    }

    fn fallback_compare_versions(v1: &str, v2: &str) -> i32 {
        let parts1: Vec<&str> = v1.split('.').collect();
        let parts2: Vec<&str> = v2.split('.').collect();
        let max_len = parts1.len().max(parts2.len());

        for i in 0..max_len {
            let n1 = parts1.get(i).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            let n2 = parts2.get(i).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            if n2 > n1 {
                return 1;
            } else if n2 < n1 {
                return -1;
            }
        }
        0
    }
}

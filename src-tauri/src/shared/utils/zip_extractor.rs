use std::fs::{self, File};
use std::io;
use std::path::Path;
use zip::ZipArchive;
use crate::domain::errors::LauncherError;

pub struct ZipExtractor;

impl ZipExtractor {
    pub fn extract_natives(jar_path: &Path, output_dir: &Path) -> Result<(), LauncherError> {
        if !jar_path.exists() {
            return Ok(());
        }

        let file = File::open(jar_path).map_err(|e| {
            LauncherError::filesystem(format!("Failed to open native jar {:?}: {}", jar_path, e))
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| {
            LauncherError::internal(format!("Invalid zip archive {:?}: {}", jar_path, e))
        })?;

        fs::create_dir_all(output_dir).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create natives dir {:?}: {}", output_dir, e))
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                LauncherError::internal(format!("Failed to read zip entry: {}", e))
            })?;

            let name = file.name().to_string();

            // Skip META-INF and non-native library files
            if name.starts_with("META-INF/") || name.starts_with(".git") || file.is_dir() {
                continue;
            }

            // Only extract dynamic libraries and relevant native extensions
            let is_native = name.ends_with(".dll")
                || name.ends_with(".dylib")
                || name.ends_with(".so")
                || name.ends_with(".jnilib");

            if !is_native {
                continue;
            }

            let file_name = Path::new(&name).file_name().unwrap_or_default();
            let out_path = output_dir.join(file_name);

            let mut out_file = File::create(&out_path).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create native file {:?}: {}", out_path, e))
            })?;

            io::copy(&mut file, &mut out_file).map_err(|e| {
                LauncherError::filesystem(format!("Failed to extract native file {:?}: {}", out_path, e))
            })?;
        }

        Ok(())
    }
}
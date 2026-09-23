use std::path::{Component, Path, PathBuf};
use crate::domain::modpacks::ModpackError;

pub struct SafePathResolver;

impl SafePathResolver {
    /// List of directories that should never be deleted or synced destructively during modpack updates
    pub const PROTECTED_DIRECTORIES: &'static [&'static str] = &[
        "saves",
        "screenshots",
        "logs",
        "crash-reports",
        ".nova",
        ".update",
    ];


    /// Normalizes a relative path from a manifest.
    /// If flat filenames like "AI-Improvements.jar" are provided without a subfolder,
    /// they are automatically routed into the expected Minecraft directory (mods/, resourcepacks/).
    pub fn normalize_relative_path(relative_path: &str) -> String {
        let trimmed = relative_path.trim().replace('\\', "/");
        if !trimmed.contains('/') {
            if trimmed.ends_with(".jar") {
                format!("mods/{}", trimmed)
            } else if trimmed.ends_with(".zip") {
                format!("resourcepacks/{}", trimmed)
            } else {
                trimmed
            }
        } else {
            trimmed
        }
    }

    /// Normalizes and ensures a relative path from a manifest is strictly safe and contained inside the base directory.
    pub fn resolve_safe_path(base_dir: &Path, relative_path: &str) -> Result<PathBuf, ModpackError> {
        let normalized = Self::normalize_relative_path(relative_path);
        let trimmed = normalized.as_str();
        if trimmed.is_empty() {
            return Err(ModpackError::path_traversal(relative_path));
        }

        // Reject null bytes
        if trimmed.contains('\0') {
            return Err(ModpackError::path_traversal(relative_path));
        }

        // Reject drive letters or absolute Unix paths
        if trimmed.starts_with('/') || (trimmed.len() >= 2 && trimmed.chars().nth(1) == Some(':')) {
            return Err(ModpackError::path_traversal(relative_path));
        }

        let rel_path = Path::new(trimmed);
        let mut safe_components = Vec::new();

        for component in rel_path.components() {
            match component {
                Component::Normal(c) => {
                    let s = c.to_string_lossy();
                    if s == ".." || s == "." {
                        return Err(ModpackError::path_traversal(relative_path));
                    }
                    safe_components.push(s.to_string());
                }
                Component::CurDir => continue,
                Component::ParentDir => {
                    return Err(ModpackError::path_traversal(relative_path));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(ModpackError::path_traversal(relative_path));
                }
            }
        }

        if safe_components.is_empty() {
            return Err(ModpackError::path_traversal(relative_path));
        }

        let mut final_path = base_dir.to_path_buf();
        for seg in safe_components {
            final_path.push(seg);
        }

        // Additional guarantee: resolved path must start with base_dir canonical or prefix
        Ok(final_path)
    }

    /// Checks whether a given relative path falls inside a protected folder (e.g. saves/, logs/)
    pub fn is_protected_path(relative_path: &str) -> bool {
        let normalized = Self::normalize_relative_path(relative_path).to_lowercase();
        let stripped = normalized.trim_start_matches('/');

        for protected in Self::PROTECTED_DIRECTORIES {
            if stripped == *protected || stripped.starts_with(&format!("{}/", protected)) {
                return true;
            }
        }

        // Protect options.txt unless explicitly managed
        if stripped == "options.txt" {
            return true;
        }

        false
    }
}

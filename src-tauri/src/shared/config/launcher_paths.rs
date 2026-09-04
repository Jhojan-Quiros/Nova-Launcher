use std::path::{Path, PathBuf};
use std::fs;
use crate::domain::errors::LauncherError;

#[derive(Debug, Clone)]
pub struct LauncherPaths {
    root_dir: PathBuf,
}

impl LauncherPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root.into(),
        }
    }

    pub fn default_path() -> Self {
        // Look for local launcher-data in current working directory first, fallback to user data dir
        let local_data = PathBuf::from("launcher-data");
        Self {
            root_dir: local_data,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root_dir
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.root_dir.join("instances")
    }

    pub fn instance_dir(&self, instance_id: &str) -> PathBuf {
        self.instances_dir().join(instance_id)
    }

    pub fn instance_game_dir(&self, instance_id: &str) -> PathBuf {
        self.instance_dir(instance_id).join("game")
    }

    pub fn instance_natives_dir(&self, instance_id: &str) -> PathBuf {
        self.instance_dir(instance_id).join("natives")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root_dir.join("assets")
    }

    pub fn asset_indexes_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    pub fn asset_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.root_dir.join("libraries")
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root_dir.join("versions")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root_dir.join("logs")
    }

    pub fn database_file(&self) -> PathBuf {
        self.root_dir.join("nova_launcher.db")
    }

    pub fn ensure_directories(&self) -> Result<(), LauncherError> {
        let dirs = [
            self.root(),
            &self.instances_dir(),
            &self.assets_dir(),
            &self.asset_indexes_dir(),
            &self.asset_objects_dir(),
            &self.libraries_dir(),
            &self.versions_dir(),
            &self.logs_dir(),
        ];

        for d in dirs {
            fs::create_dir_all(d).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create directory {:?}: {}", d, e))
            })?;
        }

        Ok(())
    }

    pub fn init_instance_directory(&self, instance_id: &str) -> Result<PathBuf, LauncherError> {
        let inst_dir = self.instance_dir(instance_id);
        let game_dir = self.instance_game_dir(instance_id);
        let natives_dir = self.instance_natives_dir(instance_id);

        let subdirs = [
            inst_dir.clone(),
            game_dir.clone(),
            game_dir.join("saves"),
            game_dir.join("mods"),
            game_dir.join("resourcepacks"),
            game_dir.join("shaderpacks"),
            game_dir.join("config"),
            natives_dir,
        ];

        for d in subdirs {
            fs::create_dir_all(&d).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create instance directory {:?}: {}", d, e))
            })?;
        }

        Ok(game_dir)
    }
}
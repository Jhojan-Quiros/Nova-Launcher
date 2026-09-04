use serde::{Deserialize, Serialize};
use crate::domain::errors::LauncherError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RamConfig {
    pub min_mb: u32,
    pub max_mb: u32,
}

impl Default for RamConfig {
    fn default() -> Self {
        Self {
            min_mb: 2048,
            max_mb: 4096,
        }
    }
}

impl RamConfig {
    pub fn new(min_mb: u32, max_mb: u32) -> Result<Self, LauncherError> {
        if min_mb < 512 {
            return Err(LauncherError::validation("Minimum RAM must be at least 512 MB"));
        }
        if min_mb > max_mb {
            return Err(LauncherError::validation("Minimum RAM cannot exceed Maximum RAM"));
        }
        Ok(Self { min_mb, max_mb })
    }
}

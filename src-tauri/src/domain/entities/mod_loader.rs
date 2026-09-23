use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}

impl Default for ModLoader {
    fn default() -> Self {
        Self::Vanilla
    }
}

impl fmt::Display for ModLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vanilla => write!(f, "Vanilla"),
            Self::Fabric => write!(f, "Fabric"),
            Self::Forge => write!(f, "Forge"),
            Self::NeoForge => write!(f, "NeoForge"),
        }
    }
}

impl ModLoader {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Fabric => "fabric",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        }
    }
}

/// Id used for the on-disk composite version JSON (vanilla version merged with the
/// mod loader's own version JSON), e.g. "1.20.1-forge-47.2.20".
pub fn composite_version_id(mc_version: &str, loader: ModLoader, loader_version: &str) -> String {
    format!("{}-{}-{}", mc_version, loader.slug(), loader_version)
}

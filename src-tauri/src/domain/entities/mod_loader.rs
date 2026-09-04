use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

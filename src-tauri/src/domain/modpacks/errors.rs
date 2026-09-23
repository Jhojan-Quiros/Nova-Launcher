use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl std::fmt::Display for ModpackErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}


#[derive(Error, Debug, Clone)]
pub enum ModpackError {
    #[error("Manifest error: {message}")]
    ManifestError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Download error: {message}")]
    DownloadError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Integrity verification failed: {message}")]
    IntegrityError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Update error: {message}")]
    UpdateError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Rollback error: {message}")]
    RollbackError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Compatibility error: {message}")]
    CompatibilityError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    #[error("Cancelled by user")]
    Cancelled,
}

impl ModpackError {
    pub fn to_payload(&self) -> ModpackErrorPayload {
        match self {
            ModpackError::ManifestError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::DownloadError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::IntegrityError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::UpdateError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::RollbackError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::CompatibilityError { code, message, details } => ModpackErrorPayload {
                code: code.clone(),
                message: message.clone(),
                details: details.clone(),
            },
            ModpackError::DatabaseError(msg) => ModpackErrorPayload {
                code: "MODPACK_DATABASE_ERROR".to_string(),
                message: msg.clone(),
                details: None,
            },
            ModpackError::FilesystemError(msg) => ModpackErrorPayload {
                code: "MODPACK_FILESYSTEM_ERROR".to_string(),
                message: msg.clone(),
                details: None,
            },
            ModpackError::Cancelled => ModpackErrorPayload {
                code: "MODPACK_OPERATION_CANCELLED".to_string(),
                message: "The modpack operation was cancelled by the user.".to_string(),
                details: None,
            },
        }
    }

    pub fn unsupported_schema(version: u32) -> Self {
        ModpackError::ManifestError {
            code: "UNSUPPORTED_SCHEMA_VERSION".to_string(),
            message: format!(
                "This modpack manifest uses schema version {}, which is not supported by this launcher version.",
                version
            ),
            details: Some(serde_json::json!({ "unsupportedVersion": version, "supportedVersion": 1 })),
        }
    }

    pub fn hash_mismatch(path: &str, expected: &str, actual: &str) -> Self {
        ModpackError::IntegrityError {
            code: "MODPACK_HASH_MISMATCH".to_string(),
            message: format!("A downloaded file failed integrity verification: {}", path),
            details: Some(serde_json::json!({
                "file": path,
                "expectedSha256": expected,
                "actualSha256": actual
            })),
        }
    }

    pub fn path_traversal(path: &str) -> Self {
        ModpackError::UpdateError {
            code: "PATH_TRAVERSAL_DETECTED".to_string(),
            message: format!("Security violation: Directory traversal attempt detected: {}", path),
            details: Some(serde_json::json!({ "unsafePath": path })),
        }
    }
}

impl Serialize for ModpackError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}


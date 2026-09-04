use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Minecraft error: {message}")]
    Minecraft { message: String, details: Option<String> },

    #[error("Download error: {message}")]
    Download { message: String, details: Option<String> },

    #[error("Java error: {message}")]
    Java { message: String },

    #[error("Filesystem error: {message}")]
    FileSystem { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Process error: {message}")]
    Process { message: String },

    #[error("Not found: {message}")]
    NotFound { message: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl LauncherError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation { message: msg.into() }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound { message: msg.into() }
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database { message: msg.into() }
    }

    pub fn minecraft(msg: impl Into<String>, details: Option<String>) -> Self {
        Self::Minecraft { message: msg.into(), details }
    }

    pub fn download(msg: impl Into<String>, details: Option<String>) -> Self {
        Self::Download { message: msg.into(), details }
    }

    pub fn java(msg: impl Into<String>) -> Self {
        Self::Java { message: msg.into() }
    }

    pub fn filesystem(msg: impl Into<String>) -> Self {
        Self::FileSystem { message: msg.into() }
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network { message: msg.into() }
    }

    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process { message: msg.into() }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            Self::Validation { message } => ErrorPayload {
                code: "VALIDATION_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::Database { message } => ErrorPayload {
                code: "DATABASE_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::Minecraft { message, details } => ErrorPayload {
                code: "MINECRAFT_ERROR".to_string(),
                message: message.clone(),
                details: details.clone(),
            },
            Self::Download { message, details } => ErrorPayload {
                code: "DOWNLOAD_ERROR".to_string(),
                message: message.clone(),
                details: details.clone(),
            },
            Self::Java { message } => ErrorPayload {
                code: "JAVA_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::FileSystem { message } => ErrorPayload {
                code: "FILESYSTEM_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::Network { message } => ErrorPayload {
                code: "NETWORK_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::Process { message } => ErrorPayload {
                code: "PROCESS_ERROR".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::NotFound { message } => ErrorPayload {
                code: "NOT_FOUND".to_string(),
                message: message.clone(),
                details: None,
            },
            Self::Internal(msg) => ErrorPayload {
                code: "INTERNAL_ERROR".to_string(),
                message: msg.clone(),
                details: None,
            },
        }
    }
}

impl Serialize for LauncherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}

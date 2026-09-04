use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    pub id: String,
    pub name: String,
    pub path: String,
    pub major_version: u32,
    pub raw_version: String,
    pub is_valid: bool,
}

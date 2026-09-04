use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::domain::errors::LauncherError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModsQuery {
    pub query: String,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub author: String,
    pub downloads: u64,
}

#[async_trait]
pub trait ModProviderPort: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn search_mods(&self, query: SearchModsQuery) -> Result<Vec<ModSearchResult>, LauncherError>;
}
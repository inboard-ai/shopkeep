//! Extension registry abstraction.

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::types::{Details, ListOptions, Page, Summary, Version};
use crate::Result;

pub mod fs;

/// Extension metadata stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub operations: Vec<String>,
}

impl Meta {
    pub fn to_summary(&self, version: &Version) -> Summary {
        Summary {
            id: self.id.clone(),
            name: self.name.clone(),
            version: version.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            categories: self.categories.clone(),
            updated_at: version.created_at,
        }
    }

    pub fn to_details(&self, latest: &Version, versions: Vec<String>) -> Details {
        Details {
            id: self.id.clone(),
            name: self.name.clone(),
            version: latest.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            categories: self.categories.clone(),
            updated_at: latest.created_at,
            homepage: self.homepage.clone(),
            repository: self.repository.clone(),
            keywords: self.keywords.clone(),
            versions,
            capabilities: self.capabilities.clone(),
            config_schema: self.config_schema.clone(),
            operations: self.operations.clone(),
        }
    }
}

/// Registry trait for extension storage backends.
#[async_trait]
pub trait Registry: Send + Sync {
    async fn list(&self, options: ListOptions) -> Result<Page<Summary>>;
    async fn get(&self, id: &str) -> Result<Details>;
    async fn get_versions(&self, id: &str) -> Result<Vec<Version>>;
    async fn get_version(&self, id: &str, version: &semver::Version) -> Result<Version>;
    async fn download(&self, id: &str, version: &semver::Version) -> Result<Bytes>;
    async fn publish(&self, package: Bytes) -> Result<()>;
    async fn get_latest_version(&self, id: &str) -> Result<Version>;
}

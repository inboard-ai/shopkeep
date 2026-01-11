use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub use shopkeep_core::{ListOptions, Page};

use crate::error::Result;
use crate::extension;

pub mod fs;

/// Extension metadata stored in the registry
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
    /// Convert to summary with version information
    pub fn to_summary(&self, version: &extension::Version) -> extension::Summary {
        extension::Summary {
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

    /// Convert to details with version information
    pub fn to_details(&self, latest: &extension::Version, versions: Vec<String>) -> extension::Details {
        extension::Details {
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

/// Registry trait for extension storage backends
#[async_trait]
pub trait Registry: Send + Sync {
    /// List extensions with pagination and filtering
    async fn list(&self, options: ListOptions) -> Result<Page<extension::Summary>>;

    /// Get extension details by ID
    async fn get(&self, id: &str) -> Result<extension::Details>;

    /// Get all versions of an extension
    async fn get_versions(&self, id: &str) -> Result<Vec<extension::Version>>;

    /// Get a specific version of an extension
    async fn get_version(&self, id: &str, version: &semver::Version) -> Result<extension::Version>;

    /// Download an extension package
    async fn download(&self, id: &str, version: &semver::Version) -> Result<Bytes>;

    /// Publish a new extension package
    async fn publish(&self, package: Bytes) -> Result<()>;

    /// Get the latest version of an extension
    async fn get_latest_version(&self, id: &str) -> Result<extension::Version>;
}

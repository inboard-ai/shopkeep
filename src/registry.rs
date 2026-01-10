use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::extension;

pub mod fs;

/// Options for listing extensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    /// Search query
    #[serde(default)]
    pub query: Option<String>,
    /// Filter by category
    #[serde(default)]
    pub category: Option<String>,
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// A paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u32, page: u32, per_page: u32) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

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

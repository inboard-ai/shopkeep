//! Core types for the extension registry.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// A paginated response wrapper.
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
        let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };
        Self { items, total, page, per_page, total_pages }
    }
}

/// Options for listing extensions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

/// Summary information for an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub updated_at: Timestamp,
}

/// Detailed information for an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Details {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub updated_at: Timestamp,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub operations: Vec<String>,
}

/// Version-specific information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: semver::Version,
    pub created_at: Timestamp,
    pub checksum_sha256: String,
    pub size_bytes: u64,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Summary information for an extension (used in listings)
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
    pub updated_at: DateTime<Utc>,
}

/// Detailed information for an extension
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
    pub updated_at: DateTime<Utc>,
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

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: semver::Version,
    pub created_at: DateTime<Utc>,
    pub checksum_sha256: String,
    pub size_bytes: u64,
}

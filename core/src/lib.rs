//! Core types for the shopkeep extension registry.
//!
//! This crate provides the shared data types used by both the shopkeep
//! server and clients that interact with the registry API.
//!
//! # Overview
//!
//! The main types are:
//!
//! - [`Page`] - A paginated response wrapper
//! - [`Summary`] - Brief extension info for listings
//! - [`Details`] - Full extension metadata
//! - [`Version`] - Version-specific information
//! - [`ListOptions`] - Query parameters for listing extensions
//!
//! # Example
//!
//! Fetching extensions from a shopkeep server:
//!
//! ```ignore
//! use shopkeep_core::{Page, Summary, Details, Version};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = reqwest::Client::new();
//!
//! // List extensions
//! let page: Page<Summary> = client
//!     .get("http://localhost:8080/api/v1/extensions")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//!
//! for ext in &page.items {
//!     println!("{} v{}: {}", ext.id, ext.version, ext.description);
//! }
//!
//! // Get extension details
//! let details: Details = client
//!     .get("http://localhost:8080/api/v1/extensions/my-extension")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//!
//! // List versions
//! let versions: Vec<Version> = client
//!     .get("http://localhost:8080/api/v1/extensions/my-extension/versions")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// A paginated response wrapper.
///
/// Used by the `/api/v1/extensions` endpoint to return a subset of results
/// along with pagination metadata.
///
/// # Example
///
/// ```
/// use shopkeep_core::Page;
///
/// let items = vec!["a", "b", "c"];
/// let page = Page::new(items, 100, 1, 20);
///
/// assert_eq!(page.total, 100);
/// assert_eq!(page.total_pages, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Total number of items across all pages.
    pub total: u32,
    /// Current page number (1-indexed).
    pub page: u32,
    /// Number of items per page.
    pub per_page: u32,
    /// Total number of pages.
    pub total_pages: u32,
}

impl<T> Page<T> {
    /// Creates a new paginated response.
    ///
    /// Automatically calculates `total_pages` from `total` and `per_page`.
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

/// Options for listing extensions.
///
/// Used as query parameters for the `/api/v1/extensions` endpoint.
///
/// # Example
///
/// ```
/// use shopkeep_core::ListOptions;
///
/// let options = ListOptions {
///     query: Some("storage".to_string()),
///     category: Some("utilities".to_string()),
///     page: 1,
///     per_page: 10,
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    /// Search query to filter by name, description, or ID.
    #[serde(default)]
    pub query: Option<String>,
    /// Filter by category.
    #[serde(default)]
    pub category: Option<String>,
    /// Page number (1-indexed). Defaults to 1.
    #[serde(default = "default_page")]
    pub page: u32,
    /// Number of items per page. Defaults to 20.
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// Summary information for an extension.
///
/// Returned in paginated listings from `/api/v1/extensions`. Contains
/// only the essential fields needed for displaying extension lists.
///
/// For full metadata, fetch [`Details`] from `/api/v1/extensions/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Unique extension identifier (e.g., `"my-extension"`).
    pub id: String,
    /// Human-readable name (e.g., `"My Extension"`).
    pub name: String,
    /// Latest version.
    pub version: semver::Version,
    /// Short description of the extension.
    pub description: String,
    /// Extension author.
    pub author: String,
    /// License identifier (e.g., `"MIT"`).
    pub license: String,
    /// Categories this extension belongs to.
    #[serde(default)]
    pub categories: Vec<String>,
    /// When the latest version was published.
    pub updated_at: Timestamp,
}

/// Detailed information for an extension.
///
/// Returned from `/api/v1/extensions/{id}`. Contains full metadata
/// including all available versions, capabilities, and configuration schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Details {
    /// Unique extension identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Latest version.
    pub version: semver::Version,
    /// Short description of the extension.
    pub description: String,
    /// Extension author.
    pub author: String,
    /// License identifier.
    pub license: String,
    /// Categories this extension belongs to.
    #[serde(default)]
    pub categories: Vec<String>,
    /// When the latest version was published.
    pub updated_at: Timestamp,
    /// Homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Source repository URL.
    #[serde(default)]
    pub repository: Option<String>,
    /// Search keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// All available version strings (e.g., `["1.0.0", "0.9.0"]`).
    #[serde(default)]
    pub versions: Vec<String>,
    /// Extension capabilities (e.g., `["storage", "networking"]`).
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// JSON Schema for extension configuration.
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
    /// Available operations/commands.
    #[serde(default)]
    pub operations: Vec<String>,
}

/// Version-specific information.
///
/// Returned from `/api/v1/extensions/{id}/versions` and
/// `/api/v1/extensions/{id}/versions/{version}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// The semantic version.
    pub version: semver::Version,
    /// When this version was published.
    pub created_at: Timestamp,
    /// SHA-256 checksum of the package file.
    pub checksum_sha256: String,
    /// Size of the package file in bytes.
    pub size_bytes: u64,
}

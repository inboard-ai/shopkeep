//! Extension registry module for runway.
//!
//! Provides a pluggable extension registry with filesystem storage backend.
//!
//! # Example
//!
//! ```ignore
//! use std::path::PathBuf;
//! use runway::{Module, Router};
//! use shopkeep::ExtensionModule;
//!
//! let mut router = Router::new();
//! let ext = ExtensionModule::new(PathBuf::from("./registry"));
//! ext.routes(&mut router);
//! ```

pub mod handler;
pub mod registry;
pub mod types;

use std::path::PathBuf;
use std::sync::Arc;

use runway::{Module, Router};

pub use registry::fs::FilesystemRegistry;
pub use registry::Registry;
pub use types::{Details, ListOptions, Page, Summary, Version};

/// Extension-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Version not found: {id}@{version}")]
    VersionNotFound { id: String, version: String },

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Invalid package: {0}")]
    InvalidPackage(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<Error> for runway::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::NotFound(msg) => runway::Error::NotFound(msg),
            Error::VersionNotFound { id, version } => {
                runway::Error::NotFound(format!("{}@{}", id, version))
            }
            Error::InvalidVersion(msg) | Error::InvalidPackage(msg) | Error::BadRequest(msg) => {
                runway::Error::BadRequest(msg)
            }
            Error::Io(e) => runway::Error::Internal(e.to_string()),
            Error::Json(e) => runway::Error::Internal(e.to_string()),
            Error::Internal(msg) => runway::Error::Internal(msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Extension registry module.
pub struct ExtensionModule {
    registry: Arc<dyn Registry>,
}

impl ExtensionModule {
    /// Create a new extension module with a filesystem registry.
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            registry: Arc::new(FilesystemRegistry::new(registry_path)),
        }
    }

    /// Create a new extension module with a custom registry implementation.
    pub fn with_registry(registry: Arc<dyn Registry>) -> Self {
        Self { registry }
    }
}

impl Module for ExtensionModule {
    fn name(&self) -> &'static str {
        "extension"
    }

    fn routes(&self, router: &mut Router) {
        use handler::*;
        let r = &self.registry;

        router.get("/api/v1/extensions", with(r, list_extensions));
        router.get("/api/v1/extensions/{id}", with(r, get_extension));
        router.get("/api/v1/extensions/{id}/versions", with(r, list_versions));
        router.get("/api/v1/extensions/{id}/versions/{version}", with(r, get_version));
        router.get("/api/v1/extensions/{id}/versions/{version}/download", with(r, download));
        router.get("/api/v1/extensions/{id}/latest/download", with(r, download_latest));
    }
}

/// Helper to bind a registry to a handler function.
fn with<F, Fut>(
    registry: &Arc<dyn Registry>,
    handler: F,
) -> impl Fn(runway::Context) -> std::pin::Pin<Box<dyn std::future::Future<Output = runway::Result<runway::response::HttpResponse>> + Send>> + Send + Sync + 'static
where
    F: Fn(runway::Context, Arc<dyn Registry>) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<runway::response::HttpResponse>> + Send + 'static,
{
    let r = registry.clone();
    move |ctx| {
        let r = r.clone();
        let handler = handler.clone();
        Box::pin(async move { handler(ctx, r).await.map_err(Into::into) })
    }
}

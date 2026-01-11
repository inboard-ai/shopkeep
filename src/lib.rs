//! shopkeep - HTTP server for the emporium extension marketplace
//!
//! This crate provides a standalone HTTP server for serving extension packages.

pub mod api;
pub mod config;
pub mod error;
pub mod extension;
pub mod registry;

pub use config::{Config, RegistryConfig};
pub use error::{Error, Result};
pub use registry::{ListOptions, Meta, Page, Registry};

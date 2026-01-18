//! Filesystem-based registry implementation.

use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use jiff::Timestamp;
use sha2::{Digest, Sha256};
use tokio::fs;
use tracing::{debug, info};

use crate::registry::{Meta, Registry};
use crate::types::{ListOptions, Page, Summary, Version};
use crate::{Error, Result};

/// Filesystem-based registry.
pub struct FilesystemRegistry {
    path: PathBuf,
}

impl FilesystemRegistry {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn extensions_dir(&self) -> PathBuf {
        self.path.join("extensions")
    }

    fn extension_dir(&self, id: &str) -> PathBuf {
        self.extensions_dir().join(id)
    }

    fn extension_meta_path(&self, id: &str) -> PathBuf {
        self.extension_dir(id).join("meta.json")
    }

    fn versions_dir(&self, id: &str) -> PathBuf {
        self.extension_dir(id).join("versions")
    }

    fn version_dir(&self, id: &str, version: &semver::Version) -> PathBuf {
        self.versions_dir(id).join(version.to_string())
    }

    fn version_meta_path(&self, id: &str, version: &semver::Version) -> PathBuf {
        self.version_dir(id, version).join("meta.json")
    }

    fn package_path(&self, id: &str, version: &semver::Version) -> PathBuf {
        self.version_dir(id, version).join("package.empkg")
    }

    async fn read_extension_meta(&self, id: &str) -> Result<Meta> {
        let path = self.extension_meta_path(id);
        let content = fs::read_to_string(&path)
            .await
            .map_err(|_| Error::NotFound(format!("Extension {}", id)))?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn read_version_meta(&self, id: &str, version: &semver::Version) -> Result<Version> {
        let path = self.version_meta_path(id, version);
        let content = fs::read_to_string(&path)
            .await
            .map_err(|_| Error::VersionNotFound {
                id: id.to_string(),
                version: version.to_string(),
            })?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn list_extension_ids(&self) -> Result<Vec<String>> {
        let dir = self.extensions_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut ids = Vec::new();
        let mut entries = fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    ids.push(name.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    async fn list_versions(&self, id: &str) -> Result<Vec<semver::Version>> {
        let dir = self.versions_dir(id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        let mut entries = fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(v) = semver::Version::parse(name) {
                        versions.push(v);
                    }
                }
            }
        }
        versions.sort();
        versions.reverse();
        Ok(versions)
    }
}

#[async_trait]
impl Registry for FilesystemRegistry {
    async fn list(&self, options: ListOptions) -> Result<Page<Summary>> {
        let ids = self.list_extension_ids().await?;
        let mut summaries = Vec::new();

        for id in &ids {
            let meta = match self.read_extension_meta(id).await {
                Ok(m) => m,
                Err(_) => continue,
            };

            if let Some(ref query) = options.query {
                let q = query.to_lowercase();
                if !meta.name.to_lowercase().contains(&q)
                    && !meta.description.to_lowercase().contains(&q)
                    && !meta.id.to_lowercase().contains(&q)
                {
                    continue;
                }
            }

            if let Some(ref category) = options.category {
                if !meta.categories.iter().any(|c| c.eq_ignore_ascii_case(category)) {
                    continue;
                }
            }

            let versions = self.list_versions(id).await?;
            if let Some(latest) = versions.first() {
                if let Ok(version_meta) = self.read_version_meta(id, latest).await {
                    summaries.push(meta.to_summary(&version_meta));
                }
            }
        }

        let total = summaries.len() as u32;
        let page = options.page.max(1);
        let per_page = options.per_page.clamp(1, 100);
        let start = ((page - 1) * per_page) as usize;
        let items: Vec<_> = summaries.into_iter().skip(start).take(per_page as usize).collect();

        Ok(Page::new(items, total, page, per_page))
    }

    async fn get(&self, id: &str) -> Result<crate::types::Details> {
        let meta = self.read_extension_meta(id).await?;
        let versions = self.list_versions(id).await?;
        let latest = versions.first().ok_or_else(|| Error::NotFound(format!("Extension {}", id)))?;
        let latest_meta = self.read_version_meta(id, latest).await?;
        let version_strings: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
        Ok(meta.to_details(&latest_meta, version_strings))
    }

    async fn get_versions(&self, id: &str) -> Result<Vec<Version>> {
        let _ = self.read_extension_meta(id).await?;
        let versions = self.list_versions(id).await?;
        let mut result = Vec::new();
        for v in versions {
            if let Ok(meta) = self.read_version_meta(id, &v).await {
                result.push(meta);
            }
        }
        Ok(result)
    }

    async fn get_version(&self, id: &str, version: &semver::Version) -> Result<Version> {
        let _ = self.read_extension_meta(id).await?;
        self.read_version_meta(id, version).await
    }

    async fn download(&self, id: &str, version: &semver::Version) -> Result<Bytes> {
        let _ = self.read_version_meta(id, version).await?;
        let path = self.package_path(id, version);
        let content = fs::read(&path).await.map_err(|_| Error::VersionNotFound {
            id: id.to_string(),
            version: version.to_string(),
        })?;
        debug!("Downloaded package: {}@{} ({} bytes)", id, version, content.len());
        Ok(Bytes::from(content))
    }

    async fn publish(&self, package: Bytes) -> Result<()> {
        let cursor = std::io::Cursor::new(&package);
        let decoder = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(decoder);

        let mut manifest: Option<serde_json::Value> = None;
        for entry in archive.entries().map_err(|e| Error::InvalidPackage(e.to_string()))? {
            let mut entry = entry.map_err(|e| Error::InvalidPackage(e.to_string()))?;
            let path = entry.path().map_err(|e| Error::InvalidPackage(e.to_string()))?;
            if path.ends_with("manifest.json") {
                let mut content = String::new();
                std::io::Read::read_to_string(&mut entry, &mut content)
                    .map_err(|e| Error::InvalidPackage(e.to_string()))?;
                manifest = Some(serde_json::from_str(&content)?);
                break;
            }
        }

        let manifest = manifest.ok_or_else(|| Error::InvalidPackage("Missing manifest.json".into()))?;
        let id = manifest["id"].as_str().ok_or_else(|| Error::InvalidPackage("Missing id".into()))?;
        let version_str = manifest["version"].as_str().ok_or_else(|| Error::InvalidPackage("Missing version".into()))?;
        let version = semver::Version::parse(version_str).map_err(|e| Error::InvalidVersion(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&package);
        let checksum = hex::encode(hasher.finalize());

        let version_dir = self.version_dir(id, &version);
        fs::create_dir_all(&version_dir).await?;

        let meta_path = self.extension_meta_path(id);
        if !meta_path.exists() {
            let meta = Meta {
                id: id.to_string(),
                name: manifest["name"].as_str().unwrap_or(id).to_string(),
                description: manifest["description"].as_str().unwrap_or("").to_string(),
                author: manifest["author"].as_str().unwrap_or("").to_string(),
                license: manifest["license"].as_str().unwrap_or("MIT").to_string(),
                categories: extract_strings(&manifest["categories"]),
                keywords: extract_strings(&manifest["keywords"]),
                homepage: manifest["homepage"].as_str().map(String::from),
                repository: manifest["repository"].as_str().map(String::from),
                capabilities: extract_strings(&manifest["capabilities"]),
                config_schema: manifest.get("config_schema").cloned(),
                operations: extract_strings(&manifest["operations"]),
            };
            fs::write(&meta_path, serde_json::to_string_pretty(&meta)?).await?;
        }

        let version_meta = Version {
            version: version.clone(),
            created_at: Timestamp::now(),
            checksum_sha256: checksum,
            size_bytes: package.len() as u64,
        };
        fs::write(self.version_meta_path(id, &version), serde_json::to_string_pretty(&version_meta)?).await?;
        fs::write(self.package_path(id, &version), &package).await?;

        info!("Published extension: {}@{}", id, version);
        Ok(())
    }

    async fn get_latest_version(&self, id: &str) -> Result<Version> {
        let versions = self.list_versions(id).await?;
        let latest = versions.first().ok_or_else(|| Error::NotFound(format!("Extension {}", id)))?;
        self.read_version_meta(id, latest).await
    }
}

fn extract_strings(value: &serde_json::Value) -> Vec<String> {
    value.as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

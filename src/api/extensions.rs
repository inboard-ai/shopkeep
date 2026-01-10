use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

use crate::error::{Error, Result};
use crate::registry::{ListOptions, Registry};

/// Handle GET /api/v1/extensions
pub async fn list(
    registry: Arc<dyn Registry>,
    query: Option<String>,
    category: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<Response<Full<Bytes>>> {
    let options = ListOptions {
        query,
        category,
        page: page.unwrap_or(1),
        per_page: per_page.unwrap_or(20),
    };

    let result = registry.list(options).await?;
    let body = serde_json::to_string(&result)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

/// Handle GET /api/v1/extensions/{id}
pub async fn get(registry: Arc<dyn Registry>, id: &str) -> Result<Response<Full<Bytes>>> {
    let details = registry.get(id).await?;
    let body = serde_json::to_string(&details)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

/// Handle GET /api/v1/extensions/{id}/versions
pub async fn list_versions(registry: Arc<dyn Registry>, id: &str) -> Result<Response<Full<Bytes>>> {
    let versions = registry.get_versions(id).await?;
    let body = serde_json::to_string(&versions)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

/// Handle GET /api/v1/extensions/{id}/versions/{version}
pub async fn get_version(
    registry: Arc<dyn Registry>,
    id: &str,
    version: &str,
) -> Result<Response<Full<Bytes>>> {
    let version = semver::Version::parse(version)
        .map_err(|e| Error::InvalidVersion(e.to_string()))?;

    let version_info = registry.get_version(id, &version).await?;
    let body = serde_json::to_string(&version_info)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

/// Handle GET /api/v1/extensions/{id}/versions/{version}/download
pub async fn download(
    registry: Arc<dyn Registry>,
    id: &str,
    version: &str,
) -> Result<Response<Full<Bytes>>> {
    let version = semver::Version::parse(version)
        .map_err(|e| Error::InvalidVersion(e.to_string()))?;

    let data = registry.download(id, &version).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}-{}.empkg\"", id, version),
        )
        .body(Full::new(data))
        .unwrap())
}

/// Handle GET /api/v1/extensions/{id}/latest/download
pub async fn download_latest(
    registry: Arc<dyn Registry>,
    id: &str,
) -> Result<Response<Full<Bytes>>> {
    let latest = registry.get_latest_version(id).await?;

    // Redirect to versioned download
    let location = format!(
        "/api/v1/extensions/{}/versions/{}/download",
        id, latest.version
    );

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", location)
        .body(Full::new(Bytes::new()))
        .unwrap())
}

/// Parse query string into key-value pairs
pub fn parse_query(query: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(q) = query {
        for part in q.split('&') {
            if let Some((key, value)) = part.split_once('=') {
                map.insert(
                    urlencoding_decode(key),
                    urlencoding_decode(value),
                );
            }
        }
    }
    map
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

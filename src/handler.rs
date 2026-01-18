//! HTTP handlers for extension registry endpoints.

use std::sync::Arc;

use runway::response::HttpResponse;
use runway::{response, Context};

use crate::registry::Registry;
use crate::types::ListOptions;
use crate::Error;

pub async fn list_extensions(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let query_params = parse_query(ctx.request.uri().query());
    let options = ListOptions {
        query: query_params.get("q").cloned(),
        category: query_params.get("category").cloned(),
        page: query_params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1),
        per_page: query_params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20),
    };
    let result = registry.list(options).await?;
    response::ok(&result).map_err(|e| Error::Internal(e.to_string()))
}

pub async fn get_extension(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let id = ctx.require_param("id").map_err(|e| Error::BadRequest(e.to_string()))?;
    let details = registry.get(id).await?;
    response::ok(&details).map_err(|e| Error::Internal(e.to_string()))
}

pub async fn list_versions(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let id = ctx.require_param("id").map_err(|e| Error::BadRequest(e.to_string()))?;
    let versions = registry.get_versions(id).await?;
    response::ok(&versions).map_err(|e| Error::Internal(e.to_string()))
}

pub async fn get_version(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let id = ctx.require_param("id").map_err(|e| Error::BadRequest(e.to_string()))?;
    let version_str = ctx.require_param("version").map_err(|e| Error::BadRequest(e.to_string()))?;
    let version = semver::Version::parse(version_str).map_err(|e| Error::InvalidVersion(e.to_string()))?;
    let info = registry.get_version(id, &version).await?;
    response::ok(&info).map_err(|e| Error::Internal(e.to_string()))
}

pub async fn download(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let id = ctx.require_param("id").map_err(|e| Error::BadRequest(e.to_string()))?.to_string();
    let version_str = ctx.require_param("version").map_err(|e| Error::BadRequest(e.to_string()))?;
    let version = semver::Version::parse(version_str).map_err(|e| Error::InvalidVersion(e.to_string()))?;
    let data = registry.download(&id, &version).await?;
    Ok(response::binary(data, "application/octet-stream", Some(&format!("{}-{}.empkg", id, version))))
}

pub async fn download_latest(ctx: Context, registry: Arc<dyn Registry>) -> crate::Result<HttpResponse> {
    let id = ctx.require_param("id").map_err(|e| Error::BadRequest(e.to_string()))?;
    let latest = registry.get_latest_version(id).await?;
    let location = format!("/api/v1/extensions/{}/versions/{}/download", id, latest.version);
    Ok(response::redirect(&location))
}

fn parse_query(query: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(q) = query {
        for part in q.split('&') {
            if let Some((key, value)) = part.split_once('=') {
                map.insert(urldecode(key), urldecode(value));
            }
        }
    }
    map
}

fn urldecode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }
    result
}

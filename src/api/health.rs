use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

use crate::error::Result;

/// Health check response
#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Handle GET /health
pub async fn health() -> Result<Response<Full<Bytes>>> {
    let response = HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    };

    let body = serde_json::to_string(&response)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

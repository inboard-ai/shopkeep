use hyper::{Response, StatusCode};
use http_body_util::Full;
use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Extension not found: {0}")]
    NotFound(String),

    #[error("Version not found: {id}@{version}")]
    VersionNotFound { id: String, version: String },

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Invalid package: {0}")]
    InvalidPackage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound(_) | Error::VersionNotFound { .. } => StatusCode::NOT_FOUND,
            Error::InvalidVersion(_) | Error::InvalidPackage(_) => StatusCode::BAD_REQUEST,
            Error::Io(_) | Error::Json(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_response(self) -> Response<Full<Bytes>> {
        let status = self.status_code();
        let body = serde_json::json!({
            "error": self.to_string()
        });

        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body.to_string())))
            .unwrap()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

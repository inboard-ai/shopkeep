use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use matchit::Router;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::registry::Registry;

pub mod extensions;
pub mod health;

/// Route identifier
#[derive(Clone, Copy)]
enum Route {
    Health,
    ListExtensions,
    GetExtension,
    ListVersions,
    GetVersion,
    Download,
    DownloadLatest,
}

/// Build the router
fn build_router() -> Router<Route> {
    let mut router = Router::new();
    router.insert("/health", Route::Health).unwrap();
    router.insert("/api/v1/extensions", Route::ListExtensions).unwrap();
    router.insert("/api/v1/extensions/{id}", Route::GetExtension).unwrap();
    router.insert("/api/v1/extensions/{id}/versions", Route::ListVersions).unwrap();
    router.insert("/api/v1/extensions/{id}/versions/{version}", Route::GetVersion).unwrap();
    router.insert("/api/v1/extensions/{id}/versions/{version}/download", Route::Download).unwrap();
    router.insert("/api/v1/extensions/{id}/latest/download", Route::DownloadLatest).unwrap();
    router
}

/// Handle incoming requests
async fn handle_request(
    req: Request<Incoming>,
    registry: Arc<dyn Registry>,
    router: Arc<Router<Route>>,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let method = req.method().clone();
    let path = req.uri().path();
    let query = req.uri().query();

    debug!("{} {}", method, path);

    // Match route
    let matched = match router.at(path) {
        Ok(m) => m,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(r#"{"error":"Not found"}"#)))
                .unwrap());
        }
    };

    let route = *matched.value;
    let params = matched.params;

    // Dispatch to handler
    let result = match (method, route) {
        (Method::GET, Route::Health) => health::health().await,

        (Method::GET, Route::ListExtensions) => {
            let query_params = extensions::parse_query(query);
            extensions::list(
                registry,
                query_params.get("q").cloned(),
                query_params.get("category").cloned(),
                query_params.get("page").and_then(|p| p.parse().ok()),
                query_params.get("per_page").and_then(|p| p.parse().ok()),
            )
            .await
        }

        (Method::GET, Route::GetExtension) => {
            let id = params.get("id").unwrap();
            extensions::get(registry, id).await
        }

        (Method::GET, Route::ListVersions) => {
            let id = params.get("id").unwrap();
            extensions::list_versions(registry, id).await
        }

        (Method::GET, Route::GetVersion) => {
            let id = params.get("id").unwrap();
            let version = params.get("version").unwrap();
            extensions::get_version(registry, id, version).await
        }

        (Method::GET, Route::Download) => {
            let id = params.get("id").unwrap();
            let version = params.get("version").unwrap();
            extensions::download(registry, id, version).await
        }

        (Method::GET, Route::DownloadLatest) => {
            let id = params.get("id").unwrap();
            extensions::download_latest(registry, id).await
        }

        _ => Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(r#"{"error":"Method not allowed"}"#)))
            .unwrap()),
    };

    // Convert result to response
    match result {
        Ok(response) => Ok(response),
        Err(e) => Ok(e.into_response()),
    }
}

/// Run the HTTP server
pub async fn run(config: Config, registry: Arc<dyn Registry>) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.bind, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    let router = Arc::new(build_router());

    info!("Server listening on http://{}", addr);

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let registry = Arc::clone(&registry);
        let router = Arc::clone(&router);

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let registry = Arc::clone(&registry);
                let router = Arc::clone(&router);
                handle_request(req, registry, router)
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection from {}: {}", remote_addr, e);
            }
        });
    }
}

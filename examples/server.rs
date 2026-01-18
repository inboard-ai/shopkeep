//! Example shopkeep server using runway.

use std::path::PathBuf;

use clap::Parser;
use runway::{Module, Router};
use shopkeep::ExtensionModule;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "shopkeep")]
#[command(version, about = "Extension registry server")]
struct Args {
    /// Registry path
    #[arg(long, default_value = "./registry")]
    registry_path: PathBuf,

    /// Bind address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port number
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,hyper=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    info!("Starting shopkeep server on {}:{}", args.host, args.port);
    info!("Registry path: {}", args.registry_path.display());

    let mut router = Router::new();

    // Health endpoint
    router.get("/health", |_ctx| async move {
        runway::response::ok(&serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION")
        }))
    });

    // Extension module
    let ext = ExtensionModule::new(args.registry_path);
    info!("Loading module: {}", ext.name());
    ext.routes(&mut router);

    // Build minimal config for runway server
    let config = runway::Config {
        server: runway::config::Server {
            host: args.host,
            port: args.port,
        },
        database: runway::config::Database {
            url: String::new(), // Not used
        },
        auth: runway::config::Auth {
            jwt_secret: String::new(), // Not used for extension registry
            token_expiry_days: 0,
        },
    };

    runway::server::run(config, None, router.into_handle()).await?;

    Ok(())
}

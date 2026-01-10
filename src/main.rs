use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use shopkeep::config::{Config, RegistryConfig};
use shopkeep::registry::fs::FilesystemRegistry;

/// HTTP server for the emporium extension marketplace
#[derive(Parser, Debug)]
#[command(name = "shopkeep")]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Registry path (filesystem backend)
    #[arg(long, value_name = "PATH")]
    registry_path: Option<PathBuf>,

    /// Bind address
    #[arg(long, value_name = "ADDR")]
    bind: Option<String>,

    /// Port number
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,hyper=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let args = Args::parse();

    // Load configuration
    let config = Config::load(
        args.config.as_ref(),
        args.bind.as_deref(),
        args.port,
        args.registry_path.as_ref(),
    )?;

    info!("Configuration loaded: bind={}:{}", config.bind, config.port);

    // Create registry
    let registry: Arc<dyn shopkeep::Registry> = match &config.registry {
        RegistryConfig::Filesystem { path } => {
            info!("Using filesystem registry at: {}", path.display());
            Arc::new(FilesystemRegistry::new(path.clone()))
        }
    };

    // Start server
    shopkeep::api::run(config, registry).await
}

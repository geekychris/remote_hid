use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, error};

mod server;
mod session;
mod config;

use config::Config;
use server::SessionServer;

#[derive(Parser, Debug)]
#[command(name = "session-server")]
#[command(about = "Remote HID Session Server")]
#[command(version = "0.1.0")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Server bind address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    
    /// Server bind port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("session_server={},remote_hid_shared={}", log_level, log_level))
        .init();
    
    info!("Starting Remote HID Session Server v{}", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = Config::load(&args.config).unwrap_or_else(|_| {
        info!("Could not load config file, using defaults with CLI overrides");
        Config::default()
    });
    
    // Override config with CLI arguments
    let mut config = config;
    config.server.host = args.host;
    config.server.port = args.port;
    
    info!("Server configuration: {:?}", config.server);
    
    // Create and start the server
    let server = Arc::new(SessionServer::new(config).await?);
    
    match server.run().await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Server error: {}", e);
            Err(e)
        }
    }
}
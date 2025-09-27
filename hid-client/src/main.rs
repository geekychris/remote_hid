use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

mod client;
mod hid;

use client::HidClient;

#[derive(Parser, Debug)]
#[command(name = "hid-client")]
#[command(about = "Remote HID Client")]
#[command(version = "0.1.0")]
struct Args {
    /// Session server URL
    #[arg(short, long, default_value = "ws://127.0.0.1:8080")]
    server: String,
    
    /// Client identifier
    #[arg(long)]
    client_id: Option<String>,
    
    /// Client display name
    #[arg(long)]
    client_name: Option<String>,
    
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
        .with_env_filter(format!("hid_client={},remote_hid_shared={}", log_level, log_level))
        .init();
    
    info!("Starting Remote HID Client v{}", env!("CARGO_PKG_VERSION"));
    
    // Generate client ID if not provided
    let client_id = args.client_id.unwrap_or_else(|| {
        format!("hid-{}", uuid::Uuid::new_v4().simple())
    });
    
    info!("Client ID: {}", client_id);
    info!("Connecting to server: {}", args.server);
    
    // Create and run the client
    let client = HidClient::new(args.server, client_id, args.client_name)?;
    
    match client.run().await {
        Ok(_) => {
            info!("Client shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Client error: {}", e);
            Err(e)
        }
    }
}
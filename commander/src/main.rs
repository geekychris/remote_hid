use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

mod client;
mod input_capture;

use client::Commander;

#[derive(Parser, Debug)]
#[command(name = "commander")]
#[command(about = "Remote HID Commander")]
#[command(version = "0.1.0")]
struct Args {
    /// Session server URL
    #[arg(short, long, default_value = "ws://127.0.0.1:8080")]
    server: String,
    
    /// Target HID client ID to control
    #[arg(short, long)]
    target: String,
    
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
        .with_env_filter(format!("commander={},remote_hid_shared={}", log_level, log_level))
        .init();
    
    info!("Starting Remote HID Commander v{}", env!("CARGO_PKG_VERSION"));
    info!("Connecting to server: {}", args.server);
    info!("Target HID client: {}", args.target);
    
    println!("===============================================");
    println!("Remote HID Commander");
    println!("===============================================");
    println!("Target: {}", args.target);
    println!("Server: {}", args.server);
    println!();
    println!("Instructions:");
    println!("- Move your mouse to control the remote cursor");
    println!("- Click mouse buttons to send clicks");
    println!("- Type on keyboard to send key events");
    println!("- Press Ctrl+C to exit");
    println!("===============================================");
    
    // Create and run the commander
    let commander = Commander::new(args.server, args.target)?;
    
    match commander.run().await {
        Ok(_) => {
            info!("Commander shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Commander error: {}", e);
            Err(e)
        }
    }
}
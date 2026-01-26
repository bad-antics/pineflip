//! PineFlip - Professional Flipper Zero Companion Application
//!
//! A modern GTK4/libadwaita application for managing Flipper Zero devices.

mod app;
mod device;
mod ui;
mod protocol;
mod screen;
mod files;
mod firmware;
mod config;

use anyhow::Result;
use clap::Parser;
use gtk4::prelude::*;
use libadwaita as adw;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "pineflip")]
#[command(author = "bad-antics")]
#[command(version = "0.1.0")]
#[command(about = "Professional Flipper Zero companion application")]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Run in CLI mode (no GUI)
    #[arg(long)]
    cli: bool,
    
    /// Serial port to use (auto-detect if not specified)
    #[arg(short, long)]
    port: Option<String>,
    
    /// Start screen mirroring immediately
    #[arg(long)]
    mirror: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let filter = if args.debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| filter.into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("PineFlip v{} starting...", env!("CARGO_PKG_VERSION"));
    
    if args.cli {
        // CLI mode
        cli_main(args)
    } else {
        // GUI mode
        gui_main(args)
    }
}

fn gui_main(_args: Args) -> Result<()> {
    // Initialize GTK
    let application = adw::Application::builder()
        .application_id("io.github.badantics.pineflip")
        .build();
    
    application.connect_activate(|app| {
        let window = app::PineFlipWindow::new(app);
        window.present();
    });
    
    application.run();
    Ok(())
}

fn cli_main(args: Args) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // Find device
        let device = if let Some(port) = args.port {
            device::FlipperDevice::connect(&port).await?
        } else {
            tracing::info!("Searching for Flipper Zero...");
            device::FlipperDevice::auto_detect().await?
        };
        
        tracing::info!("Connected to: {}", device.name());
        
        if args.mirror {
            // CLI screen mirror (ASCII art mode)
            screen::cli_mirror(&device, 10).await?;
        } else {
            // Interactive CLI
            cli_interactive(&device).await?;
        }
        
        Ok(())
    })
}

async fn cli_interactive(_device: &device::FlipperDevice) -> Result<()> {
    tracing::info!("CLI interactive mode not yet implemented");
    Ok(())
}

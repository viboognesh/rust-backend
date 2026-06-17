mod config;

use anyhow::{Context, Result};
use axum::{
    extract::Path,
    routing::get,
    Router,
};
use clap::Parser;
use std::env;
use tokio::net::TcpListener;
use tracing::{debug, info, warn, Level};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(long)]
    debug: bool,

    #[arg(long)]
    log_level: Option<String>,
}

async fn hello_name(Path(name): Path<String>) -> String {
    debug!("Entered hello_name function");
    debug!("Received name input: {}", name);

    format!("Hello {}", name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if let Some(level) = cli.log_level.as_deref() {
        match level.to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "error" => Level::ERROR,
            _ => {
                Level::INFO
            }
        }
    } else if cli.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    let port: u16 = match env::var("PORT") {
        Ok(port_str) => match port_str.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                warn!(
                    "Invalid port '{}' found in environment variable. Using default port {}",
                    port_str,
                    config::DEFAULT_PORT
                );
                config::DEFAULT_PORT
            }
        },
        Err(_) => config::DEFAULT_PORT,
    };

    let address = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/hello/{name}", get(hello_name));

    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind to {}", address))?;

    info!("Serving on http://{}", address);

    axum::serve(listener, app)
        .await
        .context("Failed to start HTTP server")?;

    Ok(())
}
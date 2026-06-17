mod config;

use anyhow::{Context, Result};
use axum::{
    extract::Path,
    routing::get,
    Router,
};
use std::env;
use tokio::net::TcpListener;

async fn hello_name(Path(name): Path<String>) -> String {
    format!("Hello {}", name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let port = match env::var("PORT") {
        Ok(port) => port,
        Err(_) => config::DEFAULT_PORT.to_string(),
    };

    let address = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/hello/{name}", get(hello_name));

    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind to {}", address))?;

    println!("Server running on {}", address);

    axum::serve(listener, app)
        .await
        .context("Failed to start HTTP server")?;

    Ok(())
}
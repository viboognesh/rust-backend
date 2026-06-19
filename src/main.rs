mod config;

use anyhow::{Context, Result};
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use clap::Parser;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::{
    env,
    sync::{Arc, Mutex},
};
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

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

#[derive(Deserialize)]
struct HelloRequest {
    name: String,
}

async fn hello_name(
    State(state): State<AppState>,
    Json(payload): Json<HelloRequest>,
) -> String {
    debug!("Entered hello_name function");
    debug!("Received name input: {}", payload.name);

    let conn = match state.db.lock() {
        Ok(conn) => conn,
        Err(err) => {
            warn!("Failed to acquire database lock: {}", err);
            return "Internal server error".to_string();
        }
    };

    let exists: bool = match conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM names WHERE name = ?1)",
        params![payload.name],
        |row| row.get(0),
    ) {
        Ok(exists) => exists,
        Err(err) => {
            warn!("Failed to query database: {}", err);
            return "Database error".to_string();
        }
    };

    if exists {
        format!("Hello {}, again", payload.name)
    } else {
        match conn.execute(
            "INSERT INTO names(name) VALUES (?1)",
            params![payload.name],
        ) {
            Ok(_) => format!("Hello {}", payload.name),
            Err(err) => {
                warn!("Failed to insert name into database: {}", err);
                "Database error".to_string()
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if let Some(level) = cli.log_level.as_deref() {
        match level.to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "error" => Level::ERROR,
            _ => Level::INFO,
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

    let conn = Connection::open("names.db")
        .context("Failed to open database")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS names (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL
        )",
        [],
    )
    .context("Failed to create names table")?;

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let address = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/hello", post(hello_name))
        .with_state(state);

    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind to {}", address))?;

    info!("Serving on http://{}", address);

    axum::serve(listener, app)
        .await
        .context("Failed to start HTTP server")?;

    Ok(())
}
use tracing_subscriber::EnvFilter;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{db::create_table, handlers::new_router};

mod db;
mod handlers;
mod model;

// `main` can't be async on its own: Rust has no built-in async runtime.
// This macro starts a tokio runtime and runs our async `main` on it.
#[tokio::main]
async fn main() {
    // Filter: use RUST_LOG if it is set and valid, otherwise fall back to our
    // defaults.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::builder().parse_lossy("plant_water_intake=debug,tower_http=info")
    });

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // A typed address: unlike a string, an invalid address can't even compile.
    // Localhost only, because Caddy on the same machine will proxy to us.
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let connection = rusqlite::Connection::open("Plant-Watering.db")
        .unwrap_or_else(|e| panic!("failed to open sqlite db: {e}"));

    create_table(&connection).unwrap_or_else(|e| panic!("failed to create sqlite db: {e}"));

    // --- Router ---------------------------------------------
    let app = new_router(connection);

    // --- Start the server --------------------------------------------------
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));

    // `%addr` records `addr` as a structured field using its Display impl

    tracing::info!(%addr, "listening");

    // Runs until the process is stopped. It only resolves to Ok(()) once
    // graceful shutdown is added (`with_graceful_shutdown`).
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to serve at {addr}: {e}"));
}

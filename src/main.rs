use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{Router, routing::get};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::builder().parse_lossy("plant_water_intake=debug,tower_http=debug")
    });

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http().on_response(DefaultOnResponse::new()));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));

    tracing::info!(%addr, "listening");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to serve at {addr}: {e}"));
}

async fn root() -> &'static str {
    "Hello, World!"
}

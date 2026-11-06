use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));

    println!("listening at {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to serve at {addr}: {e}"));
}

async fn root() -> &'static str {
    "Hello, World!"
}

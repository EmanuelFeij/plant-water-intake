use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{Router, routing::get};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;

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

    // --- Routes and middleware ---------------------------------------------
    // `.layer()` wraps every route in tower middleware. TraceLayer creates a
    // *span* per request (the context: method, uri) and an *event* when the
    // response is sent (status, latency). Both log at DEBUG by default; we
    // raise them to INFO so every request gets exactly one line in production.
    let app = Router::new().route("/", get(root)).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
            .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    );

    // --- Start the server --------------------------------------------------
    // Panicking is fine at startup: if we can't bind, the program can't do anything useful. The closure builds
    // the message only on failure and includes the OS reason (`e`).
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));

    // `%addr` records `addr` as a structured field using its Display impl
    // (`?addr` would use Debug). It is not interpolated into the message.
    tracing::info!(%addr, "listening");

    // Runs until the process is stopped. It only resolves to Ok(()) once
    // graceful shutdown is added (`with_graceful_shutdown`).
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to serve at {addr}: {e}"));
}

// A handler is an async fn whose return type implements `IntoResponse`.
// `&'static str` becomes a 200 response with a text/plain body. `()` would
// give an empty 200. The last expression (no `;`) is the return value.
async fn root() -> &'static str {
    "Hello, World!"
}

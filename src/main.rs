use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::{
    db::{create_table, get_plant, get_plants, insert_plant},
    model::Plant,
};

mod db;
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

    let app_state = AppState {
        db_connection: Arc::new(Mutex::new(connection)),
    };

    // --- Routes and middleware ---------------------------------------------
    // `.layer()` wraps every route in tower middleware. TraceLayer creates a
    // *span* per request (the context: method, uri) and an *event* when the
    // response is sent (status, latency). Both log at DEBUG by default; we
    // raise them to INFO so every request gets exactly one line in production.
    let app = Router::new()
        .route("/", get(root))
        .route("/w/{slug}", get(handle_get_plant))
        .route("/w", get(handle_get_plants))
        .route("/w", post(handle_post_plant))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(app_state);

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
async fn root() -> &'static str {
    "Hello, World!"
}

// TODO:
async fn handle_get_plant(Path(slug): Path<String>, State(state): State<AppState>) -> StatusCode {
    let res = state.db_connection.lock().unwrap(); // TODO: improve this

    match get_plant(&res, &slug) {
        Ok(None) => StatusCode::NOT_FOUND,
        Ok(_) => StatusCode::OK,
        Err(_) => {
            tracing::error!("some error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn handle_get_plants(State(state): State<AppState>) -> StatusCode {
    let res = state.db_connection.lock().unwrap(); // TODO: improve this

    // TODO: we need to return json of the plants
    match get_plants(&res) {
        Ok(plants) => {
            for ele in plants {
                println!("{:?}", ele)
            }
            StatusCode::OK
        }
        Err(_) => {
            tracing::error!("some error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn handle_post_plant(State(state): State<AppState>, Json(plant): Json<Plant>) -> StatusCode {
    let res = state.db_connection.lock().unwrap(); // TODO: improve this

    match insert_plant(&res, &plant) {
        Ok(_) => StatusCode::CREATED,
        Err(_) => {
            tracing::error!("some error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    db_connection: Arc<Mutex<rusqlite::Connection>>,
}

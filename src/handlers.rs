use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use maud::{Markup, html};
use serde::Deserialize;

use model::MarkedWateredOutcome::*;
use model::time_since;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::{
    db::{get_plant, get_plants, insert_plant, mark_watered},
    model::{self, InsertOutcome, Plant},
};

// ------------------------- App State -----------------------
#[derive(Clone)]
pub struct AppState {
    db_connection: Arc<Mutex<rusqlite::Connection>>,
}

impl AppState {
    fn new(conn: rusqlite::Connection) -> Self {
        AppState {
            db_connection: Arc::new(Mutex::new(conn)),
        }
    }
}

// --------------------------- Main router --------------------------------------
pub fn new_router(conn: rusqlite::Connection) -> Router<()> {
    let internal_api_router = new_internal_router();

    let app_state = AppState::new(conn);

    Router::new()
        .nest("/api/plants", internal_api_router)
        .route("/w/{slug}", get(handle_get_tap))
        .route("/", get(handle_get_dashboard))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(app_state)
}

// -------------------------- Internal Endpoints --------------------------

fn new_internal_router() -> Router<AppState> {
    Router::new()
        .route("/", get(handle_get_plants))
        .route("/", post(handle_post_plant))
        .route("/{slug}", get(handle_get_plant))
}

pub async fn handle_get_plant(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Plant>, StatusCode> {
    let conn = state.db_connection.lock().unwrap(); // TODO: improve this

    match get_plant(&conn, &slug) {
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Ok(Some(p)) => Ok(Json(p)),
        Err(e) => {
            tracing::error!(%e, "get plant failed");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_get_plants(
    State(state): State<AppState>,
) -> Result<Json<Vec<Plant>>, StatusCode> {
    let conn = state.db_connection.lock().unwrap(); // TODO: improve this

    match get_plants(&conn) {
        Ok(plants) => Ok(Json(plants)),
        Err(e) => {
            tracing::error!(%e, "get plants failed");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
#[derive(Debug, PartialEq, Deserialize)]
pub struct PostPlantRequest {
    pub slug: String,
    pub name: String,
    pub interval_days: i64,
}

impl From<PostPlantRequest> for Plant {
    fn from(value: PostPlantRequest) -> Self {
        Plant {
            slug: value.slug,
            name: value.name,
            interval_days: value.interval_days,
            last_watered_at: None,
        }
    }
}

pub async fn handle_post_plant(
    State(state): State<AppState>,
    Json(post_plant_request): Json<PostPlantRequest>,
) -> StatusCode {
    let conn = state.db_connection.lock().unwrap(); // TODO: improve this

    let plant: Plant = post_plant_request.into();

    match insert_plant(&conn, &plant) {
        Err(e) => {
            tracing::error!(%e, "post plant failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(InsertOutcome::Created) => StatusCode::CREATED,
        Ok(InsertOutcome::PlantAlreadyExists) => StatusCode::CONFLICT,
    }
}

// --------------- External Endpoints ---------------------------

pub async fn handle_get_tap(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> (StatusCode, String) {
    let conn = state.db_connection.lock().unwrap();

    let unix_now_in_seconds = jiff::Timestamp::now().as_second();

    match mark_watered(&conn, &slug, unix_now_in_seconds) {
        Err(e) => {
            tracing::error!(%e, "watering plant failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("watering plant failed"),
            )
        }
        Ok(NotFound) => {
            tracing::info!(%slug, "plant not found");
            (StatusCode::NOT_FOUND, String::from("plant not found"))
        }
        Ok(WateredRecently(plant_name)) => {
            (StatusCode::OK, format!("{plant_name} was watered recently"))
        }
        Ok(Watered(plant_name)) => (StatusCode::OK, format!("{plant_name} watered ✓")),
    }
}

pub async fn handle_get_dashboard(State(state): State<AppState>) -> (StatusCode, Markup) {
    let conn = state.db_connection.lock().unwrap();

    let unix_now_in_seconds = jiff::Timestamp::now().as_second();

    match get_plants(&conn) {
        Ok(plants) => {
            let html: Markup = html! {
                h1 {"Plants"}
                ul {
                    @for plant in &plants {

                        li {
                            (&plant.name)
                            @match plant.last_watered_at {
                                Some(t) => {   (time_since(t, unix_now_in_seconds))  },
                                None => { "never watered" }
                            }
                        }
                    }
                }
            };

            (StatusCode::OK, html)
        }
        Err(e) => {
            tracing::error!(%e, "list plants failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                html! {
                    h1 {"Some error occured"}
                },
            )
        }
    }
}

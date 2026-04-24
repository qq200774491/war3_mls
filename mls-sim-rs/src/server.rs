use axum::http::header;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{get, post, put};
use rust_embed::Embed;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::api;
use crate::state::AppState;

#[derive(Embed)]
#[folder = "web/"]
struct WebAssets;

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match WebAssets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            (
                [(header::CONTENT_TYPE, mime)],
                file.data.to_vec(),
            )
                .into_response()
        }
        None => {
            // SPA fallback
            match WebAssets::get("index.html") {
                Some(file) => (
                    [(header::CONTENT_TYPE, "text/html".to_string())],
                    file.data.to_vec(),
                )
                    .into_response(),
                None => (
                    axum::http::StatusCode::NOT_FOUND,
                    "Not Found",
                )
                    .into_response(),
            }
        }
    }
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/api/health", get(api::health::health))
        // Rooms
        .route("/api/rooms", get(api::rooms::list_rooms).post(api::rooms::create_room))
        .route("/api/rooms/{room_id}", get(api::rooms::get_room).delete(api::rooms::delete_room))
        .route("/api/rooms/{room_id}/start", post(api::rooms::start_room))
        .route("/api/rooms/{room_id}/stop", post(api::rooms::stop_room))
        .route("/api/rooms/{room_id}/state", get(api::rooms::get_state))
        .route("/api/rooms/{room_id}/events", post(api::events::send_event))
        // Players
        .route("/api/rooms/{room_id}/players", post(api::players::add_player))
        .route(
            "/api/rooms/{room_id}/players/{idx}",
            put(api::players::update_player).delete(api::players::remove_player),
        )
        .route("/api/rooms/{room_id}/players/{idx}/leave", post(api::players::player_leave))
        .route("/api/rooms/{room_id}/players/{idx}/join", post(api::players::player_join))
        .route("/api/rooms/{room_id}/players/{idx}/exit", post(api::players::player_exit))
        // Bridge
        .route("/api/bridge/login", post(api::bridge::login))
        .route("/api/bridge/event", post(api::bridge::event))
        .route("/api/bridge/poll/{room_id}/{player_index}", get(api::bridge::poll))
        .route("/api/bridge/rooms", get(api::bridge::list_rooms))
        .route("/api/bridge/config", post(api::bridge::config))
        // Archives
        .route("/api/archives", get(api::archives::list))
        .route("/api/archives/{script_name}", get(api::archives::get))
        // Settings
        .route("/api/settings", get(api::settings::get_settings).put(api::settings::update_settings))
        // WebSocket
        .route("/ws", get(api::ws::ws_handler));

    api.fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use crate::state::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let manager = state.manager.read().unwrap();
    let config = state.config.read().unwrap();
    Json(serde_json::json!({
        "ok": true,
        "name": "mls-sim",
        "version": env!("CARGO_PKG_VERSION"),
        "host": config.host,
        "room_count": manager.rooms.len(),
        "rooms": manager.list_rooms(),
    }))
}

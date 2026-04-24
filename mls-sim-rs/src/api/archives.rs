use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

use crate::state::AppState;
use crate::storage;

pub async fn list(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let archive_dir = state.config.read().unwrap().archive_dir.clone();
    Json(serde_json::Value::Array(storage::list_archives(
        &archive_dir,
    )))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(script_name): Path<String>,
) -> Json<serde_json::Value> {
    let archive_dir = state.config.read().unwrap().archive_dir.clone();
    Json(storage::load_player_archives(&archive_dir, &script_name))
}

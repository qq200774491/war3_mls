use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct SendEventRequest {
    pub ename: String,
    #[serde(default)]
    pub evalue: String,
    #[serde(default)]
    pub player_index: i32,
}

pub async fn send_event(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    Json(req): Json<SendEventRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.ename.is_empty() {
        return Err(AppError::BadRequest("ename is required".into()));
    }
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    room.send_event(req.ename, req.evalue, req.player_index);
    Ok(Json(serde_json::json!({"ok": true})))
}

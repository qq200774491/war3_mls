use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

use crate::error::AppError;
use crate::player::Player;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct BridgeLoginRequest {
    pub room_id: String,
    #[serde(default)]
    pub player_index: i32,
    #[serde(default)]
    pub name: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BridgeLoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&req.room_id)
        .ok_or_else(|| AppError::NotFound(format!("Room not found: {}", req.room_id)))?;
    {
        let mut shared = room.shared.write().unwrap();
        if !shared.players.contains_key(&req.player_index) {
            let name = if req.name.is_empty() {
                format!("Player_{}", req.player_index)
            } else {
                req.name.clone()
            };
            shared
                .players
                .insert(req.player_index, Player::new(req.player_index, name));
        }
    }
    Ok(Json(serde_json::json!({
        "ok": true,
        "player_index": req.player_index,
        "room_id": req.room_id,
    })))
}

#[derive(serde::Deserialize)]
pub struct BridgeEventRequest {
    pub room_id: String,
    #[serde(default)]
    pub player_index: i32,
    pub ename: String,
    #[serde(default)]
    pub evalue: String,
}

pub async fn event(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BridgeEventRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.room_id.is_empty() || req.ename.is_empty() {
        return Err(AppError::BadRequest(
            "room_id and ename are required".into(),
        ));
    }
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&req.room_id)
        .ok_or_else(|| AppError::NotFound(format!("Room not found: {}", req.room_id)))?;
    room.send_event(req.ename, req.evalue, req.player_index);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn poll(
    State(state): State<Arc<AppState>>,
    Path((room_id, player_index)): Path<(String, i32)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound(format!("Room not found: {}", room_id)))?;
    let events = room.poll_events(player_index);
    Ok(Json(serde_json::json!({"events": events})))
}

pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let manager = state.manager.read().unwrap();
    let rooms: Vec<serde_json::Value> = manager
        .list_rooms()
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r["id"],
                "status": r["status"],
                "player_count": r["player_count"],
                "mode_id": r["mode_id"],
            })
        })
        .collect();
    Json(serde_json::Value::Array(rooms))
}

#[derive(serde::Deserialize)]
pub struct BridgeConfigRequest {
    #[serde(default)]
    pub room_id: String,
    #[serde(default)]
    pub player_index: i32,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: f64,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

fn default_poll_interval() -> f64 {
    0.05
}

pub async fn config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BridgeConfigRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cfg = state.config.read().unwrap();
    let port = req.port.unwrap_or(cfg.port);
    let base_url = req
        .base_url
        .unwrap_or_else(|| format!("http://{}:{}", cfg.host, port));
    let content = format!(
        r#"-- MLS Bridge config
return {{
    base_url = "{}",
    room_id = "{}",
    player_index = {},
    poll_interval = {},
    req_sign_enable = false,
}}"#,
        base_url, req.room_id, req.player_index, req.poll_interval
    );
    Ok(Json(serde_json::json!({
        "base_url": base_url,
        "room_id": req.room_id,
        "player_index": req.player_index,
        "poll_interval": req.poll_interval,
        "content": content,
    })))
}

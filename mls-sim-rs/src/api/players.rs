use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use crate::error::AppError;
use crate::player::Player;
use crate::room::RoomCommand;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct AddPlayerRequest {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub items: std::collections::HashMap<String, i32>,
}

pub async fn add_player(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    Json(req): Json<AddPlayerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let mut p = Player::new(req.index, req.name);
    if !req.items.is_empty() {
        p.items = req.items;
    }
    let json = p.to_json();
    room.shared.write().unwrap().players.insert(req.index, p);
    Ok((StatusCode::CREATED, Json(json)))
}

pub async fn update_player(
    State(state): State<Arc<AppState>>,
    Path((room_id, idx)): Path<(String, i32)>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let mut shared = room.shared.write().unwrap();
    let p = shared
        .players
        .get_mut(&idx)
        .ok_or_else(|| AppError::NotFound("Player not found".into()))?;

    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
        p.name = name.to_string();
    }
    if let Some(items) = data.get("items").and_then(|v| v.as_object()) {
        p.items = items
            .iter()
            .map(|(k, v)| (k.clone(), v.as_i64().unwrap_or(0) as i32))
            .collect();
    }
    if let Some(v) = data.get("map_level").and_then(|v| v.as_i64()) {
        p.map_level = v as i32;
    }
    if let Some(v) = data.get("map_exp").and_then(|v| v.as_i64()) {
        p.map_exp = v as i32;
    }
    if let Some(v) = data.get("script_archive") {
        p.script_archive = v.as_str().map(String::from);
    }
    if let Some(v) = data.get("common_archive").and_then(|v| v.as_object()) {
        p.common_archive = v
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();
    }
    if let Some(v) = data.get("read_archive").and_then(|v| v.as_object()) {
        p.read_archive = v
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();
    }
    if let Some(v) = data.get("cfg_archive").and_then(|v| v.as_object()) {
        p.cfg_archive = v
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();
    }

    Ok(Json(p.to_json()))
}

pub async fn remove_player(
    State(state): State<Arc<AppState>>,
    Path((room_id, idx)): Path<(String, i32)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    room.shared.write().unwrap().players.remove(&idx);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn player_leave(
    State(state): State<Arc<AppState>>,
    Path((room_id, idx)): Path<(String, i32)>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let reason = body
        .and_then(|b| b.get("reason").and_then(|r| r.as_str()).map(String::from))
        .unwrap_or_else(|| "Disconnect".into());
    {
        let mut shared = room.shared.write().unwrap();
        if let Some(p) = shared.players.get_mut(&idx) {
            p.is_connected = false;
        }
    }
    let data = serde_json::json!({"reason": reason}).to_string();
    let _ = room.command_tx.send(RoomCommand::DispatchEvent {
        ename: "_playerleave".into(),
        evalue: data,
        player_index: idx,
    });
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn player_join(
    State(state): State<Arc<AppState>>,
    Path((room_id, idx)): Path<(String, i32)>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let reason = body
        .and_then(|b| b.get("reason").and_then(|r| r.as_str()).map(String::from))
        .unwrap_or_else(|| "Connect".into());
    {
        let mut shared = room.shared.write().unwrap();
        if let Some(p) = shared.players.get_mut(&idx) {
            p.is_connected = true;
        }
    }
    let data = serde_json::json!({"reason": reason}).to_string();
    let _ = room.command_tx.send(RoomCommand::DispatchEvent {
        ename: "_playerjoin".into(),
        evalue: data,
        player_index: idx,
    });
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn player_exit(
    State(state): State<Arc<AppState>>,
    Path((room_id, idx)): Path<(String, i32)>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let reason = body
        .and_then(|b| b.get("reason").and_then(|r| r.as_str()).map(String::from))
        .unwrap_or_else(|| "Logout".into());
    let data = serde_json::json!({"reason": reason}).to_string();
    let _ = room.command_tx.send(RoomCommand::DispatchEvent {
        ename: "_playerexit".into(),
        evalue: data,
        player_index: idx,
    });
    Ok(Json(serde_json::json!({"ok": true})))
}

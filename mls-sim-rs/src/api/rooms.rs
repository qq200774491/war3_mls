use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::PlayerConfig;
use crate::error::AppError;
use crate::player::Player;
use crate::state::AppState;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct CreateRoomRequest {
    pub script_dir: String,
    #[serde(default)]
    pub mode_id: i32,
    #[serde(default)]
    pub players: Vec<PlayerConfig>,
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

fn default_true() -> bool {
    true
}

fn build_players(configs: &[PlayerConfig]) -> HashMap<i32, Player> {
    let configs = if configs.is_empty() {
        vec![PlayerConfig {
            index: 0,
            name: "Player_0".into(),
            items: Default::default(),
            map_level: None,
            map_exp: None,
            played_count: None,
            script_archive: None,
            common_archive: None,
            read_archive: None,
            cfg_archive: None,
        }]
    } else {
        configs.to_vec()
    };

    let mut players = HashMap::new();
    for pc in &configs {
        let mut p = Player::new(pc.index, pc.name.clone());
        if !pc.items.is_empty() {
            p.items = pc.items.clone();
        }
        if let Some(v) = pc.map_level {
            p.map_level = v;
        }
        if let Some(v) = pc.map_exp {
            p.map_exp = v;
        }
        if let Some(v) = pc.played_count {
            p.played_count = v;
        }
        if let Some(ref v) = pc.script_archive {
            p.script_archive = Some(v.clone());
        }
        if let Some(ref v) = pc.common_archive {
            p.common_archive = v.clone();
        }
        if let Some(ref v) = pc.read_archive {
            p.read_archive = v.clone();
        }
        if let Some(ref v) = pc.cfg_archive {
            p.cfg_archive = v.clone();
        }
        players.insert(pc.index, p);
    }
    players
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let script_dir = PathBuf::from(&req.script_dir);
    if !script_dir.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Invalid script_dir: {}",
            req.script_dir
        )));
    }

    let players = build_players(&req.players);
    let archive_dir = state.config.read().unwrap().archive_dir.clone();
    let mut manager = state.manager.write().unwrap();
    let room_id = manager.create_room(script_dir, req.mode_id, players, archive_dir);

    let room = manager.get_room(&room_id).unwrap();
    let shared = room.shared.read().unwrap();
    Ok((StatusCode::CREATED, Json(shared.to_json())))
}

pub async fn list_rooms(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let manager = state.manager.read().unwrap();
    Json(serde_json::Value::Array(manager.list_rooms()))
}

pub async fn get_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let json = room.shared.read().unwrap().to_json();
    Ok(Json(json))
}

pub async fn delete_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut manager = state.manager.write().unwrap();
    if manager.destroy_room(&room_id) {
        Ok(Json(serde_json::json!({"ok": true})))
    } else {
        Err(AppError::NotFound("Room not found".into()))
    }
}

pub async fn start_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let _room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    // Room auto-starts on creation, this is a no-op for already running rooms
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn stop_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let reason = body
        .and_then(|b| b.get("reason").and_then(|r| r.as_str()).map(String::from))
        .unwrap_or_else(|| "GameEnd".into());
    room.stop(reason);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn get_state(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let manager = state.manager.read().unwrap();
    let room = manager
        .get_room(&room_id)
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    let json = room.shared.read().unwrap().to_json();
    Ok(Json(json))
}

pub fn build_players_from_config(configs: &[PlayerConfig]) -> HashMap<i32, Player> {
    build_players(configs)
}

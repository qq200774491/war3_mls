use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::{Arc, RwLock};

use crate::config::AppConfig;
use crate::room::{RoomManager, ERR_OK, ERR_PLAYER_NOT_EXIST, ERR_ROOM_NOT_EXIST};

#[derive(Clone)]
struct BridgeState {
    manager: Arc<RwLock<RoomManager>>,
    config: Arc<RwLock<AppConfig>>,
}

pub async fn run_bridge_server(
    host: String,
    port: u16,
    manager: Arc<RwLock<RoomManager>>,
    config: Arc<RwLock<AppConfig>>,
) {
    let state = BridgeState { manager, config };

    let app = Router::new()
        .route("/api/bridge/login", post(login))
        .route("/api/bridge/event", post(event))
        .route("/api/bridge/poll/{room_id}/{player_index}", get(poll))
        .route("/api/bridge/rooms", get(list_rooms))
        .route("/api/bridge/config", post(bridge_config))
        .route("/api/health", get(health))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Bridge server failed to bind {}: {}", addr, e);
            return;
        }
    };
    tracing::info!("Bridge server listening on {}", addr);
    let _ = axum::serve(listener, app).await;
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    room_id: String,
    #[serde(default)]
    player_index: i32,
    #[serde(default)]
    name: String,
}

async fn login(
    State(state): State<BridgeState>,
    Json(req): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = state.manager.read().unwrap();
    let room = match manager.get_room(&req.room_id) {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "ok": false,
                    "errnu": ERR_ROOM_NOT_EXIST,
                    "error": format!("Room not found: {}", req.room_id),
                })),
            );
        }
    };
    let name = if req.name.is_empty() {
        format!("Player_{}", req.player_index)
    } else {
        req.name.clone()
    };
    let errnu = room.join_player(req.player_index, name, "Connect".to_string());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": errnu == ERR_OK,
            "errnu": errnu,
            "player_index": req.player_index,
            "room_id": req.room_id,
        })),
    )
}

#[derive(serde::Deserialize)]
struct EventRequest {
    room_id: String,
    #[serde(default)]
    player_index: i32,
    ename: String,
    #[serde(default)]
    evalue: String,
}

async fn event(
    State(state): State<BridgeState>,
    Json(req): Json<EventRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if req.room_id.is_empty() || req.ename.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "errnu": if req.room_id.is_empty() { ERR_ROOM_NOT_EXIST } else { crate::room::ERR_EVENT_KEY_LEN },
                "error": "room_id and ename required",
            })),
        );
    }
    let manager = state.manager.read().unwrap();
    match manager.get_room(&req.room_id) {
        Some(room) => {
            let errnu = room.send_event(req.ename, req.evalue, req.player_index);
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": errnu == ERR_OK, "errnu": errnu})),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "ok": false,
                "errnu": ERR_ROOM_NOT_EXIST,
                "error": format!("Room not found: {}", req.room_id),
            })),
        ),
    }
}

async fn poll(
    State(state): State<BridgeState>,
    Path((room_id, player_index)): Path<(String, i32)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = state.manager.read().unwrap();
    match manager.get_room(&room_id) {
        Some(room) => {
            if !room.has_player(player_index) {
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "ok": false,
                        "errnu": ERR_PLAYER_NOT_EXIST,
                        "events": [],
                    })),
                );
            }
            let events = room.poll_events(player_index);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ok": true,
                    "errnu": ERR_OK,
                    "events": events,
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "ok": false,
                "errnu": ERR_ROOM_NOT_EXIST,
                "events": [],
                "error": format!("Room not found: {}", room_id),
            })),
        ),
    }
}

async fn list_rooms(State(state): State<BridgeState>) -> Json<serde_json::Value> {
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
struct ConfigRequest {
    #[serde(default)]
    room_id: String,
    #[serde(default)]
    player_index: i32,
    #[serde(default = "default_poll_interval")]
    poll_interval: f64,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

fn default_poll_interval() -> f64 {
    0.05
}

async fn bridge_config(
    State(state): State<BridgeState>,
    Json(req): Json<ConfigRequest>,
) -> Json<serde_json::Value> {
    let cfg = state.config.read().unwrap();
    let port = req.port.unwrap_or(cfg.port);
    let base_url = req
        .base_url
        .unwrap_or_else(|| format!("http://{}:{}", cfg.host, port));
    let content = format!(
        "-- MLS Bridge config\nreturn {{\n    base_url = \"{}\",\n    room_id = \"{}\",\n    player_index = {},\n    poll_interval = {},\n    req_sign_enable = false,\n}}",
        base_url, req.room_id, req.player_index, req.poll_interval
    );
    Json(serde_json::json!({
        "base_url": base_url,
        "room_id": req.room_id,
        "player_index": req.player_index,
        "poll_interval": req.poll_interval,
        "content": content,
    }))
}

async fn health(State(state): State<BridgeState>) -> Json<serde_json::Value> {
    let manager = state.manager.read().unwrap();
    Json(serde_json::json!({
        "ok": true,
        "name": "mls-sim",
        "version": env!("CARGO_PKG_VERSION"),
        "room_count": manager.rooms.len(),
    }))
}

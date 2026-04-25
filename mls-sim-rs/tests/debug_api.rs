use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use mls_sim::config::AppConfig;
use mls_sim::player::Player;
use mls_sim::room::{LogEntry, RoomManager};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    mls_sim::bridge::build_bridge_router(manager, config)
}

async fn json_response(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if body.is_some() {
        request = request.header("content-type", "application/json");
    }
    let request = request
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap();
    (status, value)
}

fn create_room_with_logs(manager: &Arc<RwLock<RoomManager>>) {
    let script_dir = std::env::temp_dir().join(format!(
        "mls-sim-debug-api-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&script_dir).unwrap();
    fs::write(script_dir.join("main.lua"), "Log.Info('boot ok')\n").unwrap();

    let mut players = HashMap::new();
    players.insert(0, Player::new(0, "Player_0".to_string()));

    let room_id = manager.write().unwrap().create_room(
        script_dir,
        0,
        players,
        std::env::temp_dir().to_string_lossy().into_owned(),
    );
    assert_eq!(room_id, "room-001");

    let manager_guard = manager.read().unwrap();
    let room = manager_guard.get_room("room-001").unwrap();
    let mut shared = room.shared.write().unwrap();
    shared.log_buffer.push_back(LogEntry {
        timestamp: 10.0,
        level: "INF".to_string(),
        source: "System".to_string(),
        message: "first boot".to_string(),
        room_id: "room-001".to_string(),
        player_index: -1,
    });
    shared.log_buffer.push_back(LogEntry {
        timestamp: 20.0,
        level: "ERR".to_string(),
        source: "Lua".to_string(),
        message: "script failed".to_string(),
        room_id: "room-001".to_string(),
        player_index: -1,
    });
}

#[tokio::test]
async fn debug_logs_support_filters_and_limit() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app,
        Method::GET,
        "/api/debug/rooms/room-001/logs?level=ERR&q=script&since=15&limit=1",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["count"], 1);
    assert_eq!(body["total"], 1);
    assert_eq!(body["logs"][0]["level"], "ERR");
    assert_eq!(body["logs"][0]["message"], "script failed");
}

#[tokio::test]
async fn clear_debug_logs_only_clears_room_log_buffer() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app.clone(),
        Method::POST,
        "/api/debug/rooms/room-001/logs/clear",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared"], 2);

    let (status, body) = json_response(
        app,
        Method::GET,
        "/api/debug/rooms/room-001/logs",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 0);
}

#[tokio::test]
async fn restart_room_returns_new_room_id_and_service_restart_is_unsupported() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app.clone(),
        Method::POST,
        "/api/debug/rooms/room-001/restart",
        Some(serde_json::json!({"reason": "test restart"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["old_room_id"], "room-001");
    assert_eq!(body["room_id"], "room-002");
    assert_eq!(body["status"], "restarted");

    let (status, body) = json_response(
        app,
        Method::POST,
        "/api/debug/service/restart",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(body["ok"], false);
    assert_eq!(body["errnu"], 1);
}

#[tokio::test]
async fn debug_api_returns_not_found_for_unknown_room() {
    let app = test_app();

    let (status, body) = json_response(
        app,
        Method::GET,
        "/api/debug/rooms/missing/logs",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["ok"], false);
    assert_eq!(body["errnu"], 2);
}

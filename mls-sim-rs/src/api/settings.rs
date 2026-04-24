use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::state::AppState;

pub async fn get_settings(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let config = state.config.read().unwrap();
    Json(serde_json::to_value(&*config).unwrap_or_default())
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<AppConfig>,
) -> Json<serde_json::Value> {
    {
        let mut config = state.config.write().unwrap();
        *config = new_config;
        if let Err(e) = config.save(&state.config_path) {
            return Json(serde_json::json!({"error": format!("Failed to save: {}", e)}));
        }
    }
    Json(serde_json::json!({"ok": true}))
}

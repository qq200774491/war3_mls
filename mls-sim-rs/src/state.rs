use crate::config::AppConfig;
use crate::room::RoomManager;
use std::sync::{Arc, RwLock};

pub struct AppState {
    pub manager: Arc<RwLock<RoomManager>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: String,
}

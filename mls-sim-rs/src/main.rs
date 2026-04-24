mod api;
mod config;
mod error;
mod player;
mod room;
mod server;
mod state;
mod storage;

use clap::Parser;
use config::{AppConfig, Cli};
use room::RoomManager;
use state::AppState;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let config_path = cli.config.clone();
    let config = AppConfig::load(&cli);

    let host = config.host.clone();
    let port = config.port;
    let auto_open = config.auto_open_browser;
    let auto_room = config.auto_room.clone();

    let state = Arc::new(AppState {
        manager: Arc::new(RwLock::new(RoomManager::new())),
        config: Arc::new(RwLock::new(config)),
        config_path,
    });

    // Auto-create room if configured
    if let Some(ref room_cfg) = auto_room {
        let script_dir = PathBuf::from(&room_cfg.script_dir);
        if script_dir.is_dir() {
            let players =
                api::rooms::build_players_from_config(&room_cfg.players);
            let archive_dir = state.config.read().unwrap().archive_dir.clone();
            let mut manager = state.manager.write().unwrap();
            let room_id =
                manager.create_room(script_dir.clone(), room_cfg.mode_id, players, archive_dir);
            tracing::info!(
                "Auto-created room {} with script_dir: {}",
                room_id,
                script_dir.display()
            );
        } else {
            tracing::warn!(
                "Auto-room script_dir not found: {}",
                room_cfg.script_dir
            );
        }
    }

    let app = server::create_router(state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!();
    println!("  MLS Simulator (Rust) v{}", env!("CARGO_PKG_VERSION"));
    println!("  Running at http://{}", addr);
    println!();

    if auto_open {
        let url = format!("http://{}:{}", host, port);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = open::that(&url);
        });
    }

    axum::serve(listener, app).await.unwrap();
}

mod bridge;
mod config;
mod gui;
mod player;
mod room;
mod storage;

use clap::Parser;
use config::{AppConfig, Cli};
use eframe::egui;
use room::RoomManager;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

fn main() {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        unsafe {
            let handle = std::io::stdout().as_raw_handle();
            let mut mode: u32 = 0;
            windows_sys::Win32::System::Console::GetConsoleMode(handle, &mut mode);
            windows_sys::Win32::System::Console::SetConsoleMode(
                handle,
                mode | windows_sys::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            );
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    #[cfg(windows)]
    if cli.console_notwrte {
        unsafe {
            windows_sys::Win32::System::Console::FreeConsole();
        }
    }

    let config_path = cli.config.clone();
    let config = AppConfig::load(&cli);

    let host = config.host.clone();
    let port = config.port;
    let auto_room = config.auto_room.clone();

    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(config));

    if let Some(ref room_cfg) = auto_room {
        let script_dir = PathBuf::from(&room_cfg.script_dir);
        if script_dir.is_dir() {
            let mut players = config::build_players_from_config(&room_cfg.players);
            let archive_dir = config.read().unwrap().archive_dir.clone();
            storage::apply_saved_archives(&archive_dir, &room_cfg.script_dir, &mut players);
            let mut mgr = manager.write().unwrap();
            let room_id =
                mgr.create_room(script_dir.clone(), room_cfg.mode_id, players, archive_dir);
            tracing::info!(
                "Auto-created room {} with script_dir: {}",
                room_id,
                script_dir.display()
            );
        } else {
            tracing::warn!("Auto-room script_dir not found: {}", room_cfg.script_dir);
        }
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    {
        let manager = manager.clone();
        let config = config.clone();
        rt.spawn(async move {
            bridge::run_bridge_server(host, port, manager, config).await;
        });
    }

    println!();
    println!(
        "  MLS Simulator v{} (Native GUI)",
        env!("CARGO_PKG_VERSION")
    );
    println!();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        &format!("MLS 云脚本环境模拟 v{}", env!("CARGO_PKG_VERSION")),
        native_options,
        Box::new(move |cc| Ok(Box::new(gui::GuiApp::new(cc, manager, config, config_path)))),
    );
}

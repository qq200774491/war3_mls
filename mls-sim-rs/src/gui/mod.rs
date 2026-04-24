mod console;
mod rooms;
mod settings;
mod state_view;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use eframe::egui;

use crate::config::AppConfig;
use crate::room::{LogEntry, OutEvent, RoomManager};

#[derive(PartialEq)]
pub enum Tab {
    Rooms,
    Console,
    State,
    Settings,
}

pub struct GuiApp {
    pub manager: Arc<RwLock<RoomManager>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: String,
    pub active_tab: Tab,

    // Rooms tab
    pub selected_room_id: Option<String>,
    pub show_create_room: bool,
    pub new_room_script_dir: String,
    pub new_room_mode_id: i32,
    pub add_player_index: String,
    pub add_player_name: String,
    pub event_name: String,
    pub event_data: String,
    pub event_player_idx: i32,

    // Console
    pub logs: Vec<LogEntry>,
    pub out_events: Vec<OutEvent>,
    subscribed_rooms: HashSet<String>,
    log_receivers: HashMap<String, broadcast::Receiver<LogEntry>>,
    event_receivers: HashMap<String, broadcast::Receiver<OutEvent>>,
    pub log_level_filter: String,
    pub log_search: String,
    pub log_room_filter: String,
    pub auto_scroll: bool,

    // State viewer
    pub state_room_id: Option<String>,
    pub state_json_text: String,

    // Settings
    pub settings_host: String,
    pub settings_port: String,
    pub settings_archive_dir: String,
}

impl GuiApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        manager: Arc<RwLock<RoomManager>>,
        config: Arc<RwLock<AppConfig>>,
        config_path: String,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);

        let (host, port, archive_dir) = {
            let cfg = config.read().unwrap();
            (cfg.host.clone(), cfg.port.to_string(), cfg.archive_dir.clone())
        };

        Self {
            manager,
            config,
            config_path,
            active_tab: Tab::Rooms,
            selected_room_id: None,
            show_create_room: false,
            new_room_script_dir: String::new(),
            new_room_mode_id: 0,
            add_player_index: "0".into(),
            add_player_name: String::new(),
            event_name: String::new(),
            event_data: String::new(),
            event_player_idx: -1,
            logs: Vec::new(),
            out_events: Vec::new(),
            subscribed_rooms: HashSet::new(),
            log_receivers: HashMap::new(),
            event_receivers: HashMap::new(),
            log_level_filter: String::new(),
            log_search: String::new(),
            log_room_filter: String::new(),
            auto_scroll: true,
            state_room_id: None,
            state_json_text: String::new(),
            settings_host: host,
            settings_port: port,
            settings_archive_dir: archive_dir,
        }
    }

    fn sync_subscriptions(&mut self) {
        let manager = match self.manager.try_read() {
            Ok(m) => m,
            Err(_) => return,
        };
        let current: HashSet<String> = manager.rooms.keys().cloned().collect();

        for id in &current {
            if !self.subscribed_rooms.contains(id) {
                if let Some(room) = manager.rooms.get(id) {
                    if let Ok(shared) = room.shared.try_read() {
                        self.logs.extend(shared.log_buffer.iter().cloned());
                        self.out_events.extend(shared.event_buffer.iter().cloned());
                    }
                    self.log_receivers
                        .insert(id.clone(), room.log_tx.subscribe());
                    self.event_receivers
                        .insert(id.clone(), room.out_event_tx.subscribe());
                }
            }
        }

        let removed: Vec<String> = self
            .subscribed_rooms
            .difference(&current)
            .cloned()
            .collect();
        for id in &removed {
            self.log_receivers.remove(id);
            self.event_receivers.remove(id);
        }

        self.subscribed_rooms = current;
    }

    fn drain_channels(&mut self) {
        for rx in self.log_receivers.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(entry) => self.logs.push(entry),
                    Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                    _ => break,
                }
            }
        }
        for rx in self.event_receivers.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(event) => self.out_events.push(event),
                    Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                    _ => break,
                }
            }
        }

        const MAX_LOGS: usize = 10000;
        if self.logs.len() > MAX_LOGS {
            self.logs.drain(0..self.logs.len() - MAX_LOGS);
        }
        if self.out_events.len() > MAX_LOGS {
            self.out_events.drain(0..self.out_events.len() - MAX_LOGS);
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_subscriptions();
        self.drain_channels();

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Rooms, "  房间  ");
                ui.selectable_value(&mut self.active_tab, Tab::Console, "  控制台  ");
                ui.selectable_value(&mut self.active_tab, Tab::State, "  状态  ");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "  设置  ");
            });
        });

        match self.active_tab {
            Tab::Rooms => self.rooms_tab(ctx),
            Tab::Console => self.console_tab(ctx),
            Tab::State => self.state_tab(ctx),
            Tab::Settings => self.settings_tab(ctx),
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/simsun.ttc",
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("chinese".to_owned(), egui::FontData::from_owned(data));
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "chinese".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("chinese".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

pub fn format_time(ts: f64) -> String {
    let secs = ts as i64;
    let nanos = ((ts - secs as f64) * 1e9) as u32;
    chrono::DateTime::from_timestamp(secs, nanos)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| format!("{:.1}", ts))
}

pub fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

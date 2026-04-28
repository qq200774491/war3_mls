mod console;
mod events;
mod profiler;
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
    Events,
    State,
    Profiler,
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

    // Profiler
    pub profiler_room_id: Option<String>,
    pub profiler_hook_count: i32,
    pub profiler_window: i32,
    pub profiler_frame_ms: f32,
    pub profiler_hover_info: String,

    // Status
    pub save_msg: Option<(String, bool, f64)>,
}

impl GuiApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        manager: Arc<RwLock<RoomManager>>,
        config: Arc<RwLock<AppConfig>>,
        config_path: String,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);
        setup_style(&cc.egui_ctx);

        let (host, port, archive_dir) = {
            let cfg = config.read().unwrap();
            (
                cfg.host.clone(),
                cfg.port.to_string(),
                cfg.archive_dir.clone(),
            )
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
            profiler_room_id: None,
            profiler_hook_count: 5000,
            profiler_window: 15,
            profiler_frame_ms: 50.0,
            profiler_hover_info: String::new(),
            settings_host: host,
            settings_port: port,
            settings_archive_dir: archive_dir,
            save_msg: None,
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

impl Drop for GuiApp {
    fn drop(&mut self) {
        if let Ok(mut mgr) = self.manager.write() {
            mgr.shutdown_all();
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // DPI: sync to native pixels_per_point for crisp rendering
        if let Some(native_ppp) = ctx.native_pixels_per_point() {
            if (ctx.pixels_per_point() - native_ppp).abs() > 0.01 {
                ctx.set_pixels_per_point(native_ppp);
            }
        }


        self.sync_subscriptions();
        self.drain_channels();

        let room_count = self.subscribed_rooms.len();

        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (tab, label) in [
                        (Tab::Rooms, "房间"),
                        (Tab::Console, "控制台"),
                        (Tab::Events, "出站事件"),
                        (Tab::State, "状态"),
                        (Tab::Profiler, "性能分析"),
                        (Tab::Settings, "设置"),
                    ] {
                        let selected = self.active_tab == tab;
                        let text = egui::RichText::new(label).size(15.0);
                        let text = if selected { text.strong() } else { text };
                        if ui
                            .selectable_label(selected, text)
                            .clicked()
                        {
                            self.active_tab = tab;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui: &mut egui::Ui| {
                        ui.label(
                            egui::RichText::new(format!("房间: {}  |  日志: {}", room_count, self.logs.len()))
                                .small()
                                .color(egui::Color32::from_rgb(140, 148, 165)),
                        );
                    });
                });
            });

        match self.active_tab {
            Tab::Rooms => self.rooms_tab(ctx),
            Tab::Console => self.console_tab(ctx),
            Tab::Events => self.events_tab(ctx),
            Tab::State => self.state_tab(ctx),
            Tab::Profiler => self.profiler_tab(ctx),
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

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(15.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(15.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(10.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);

    let v = &mut style.visuals;
    v.window_fill = egui::Color32::from_rgb(22, 22, 28);
    v.panel_fill = egui::Color32::from_rgb(28, 28, 35);
    v.faint_bg_color = egui::Color32::from_rgb(38, 40, 50);
    v.extreme_bg_color = egui::Color32::from_rgb(14, 14, 18);
    v.code_bg_color = egui::Color32::from_rgb(38, 40, 50);

    v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
    v.warn_fg_color = egui::Color32::from_rgb(255, 200, 80);
    v.hyperlink_color = egui::Color32::from_rgb(100, 170, 255);

    v.selection.bg_fill = egui::Color32::from_rgb(45, 75, 135);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 165, 255));

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 36, 44);
    v.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(32, 33, 40);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(62, 64, 80));
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 205, 220));
    v.widgets.noninteractive.rounding = egui::Rounding::same(4.0);

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 46, 58);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(40, 41, 52);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 72, 90));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(215, 218, 228));
    v.widgets.inactive.rounding = egui::Rounding::same(4.0);

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 60, 76);
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(52, 54, 68);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 120, 190));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(240, 242, 250));
    v.widgets.hovered.rounding = egui::Rounding::same(4.0);

    v.widgets.active.bg_fill = egui::Color32::from_rgb(65, 75, 110);
    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(58, 66, 96);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 145, 220));
    v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    v.widgets.active.rounding = egui::Rounding::same(4.0);

    v.widgets.open.bg_fill = egui::Color32::from_rgb(48, 50, 64);
    v.widgets.open.weak_bg_fill = egui::Color32::from_rgb(44, 46, 58);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(82, 85, 105));
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(222, 225, 240));
    v.widgets.open.rounding = egui::Rounding::same(4.0);

    v.window_rounding = egui::Rounding::same(6.0);
    v.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 58, 72));
    v.striped = true;

    ctx.set_style(style);
}

pub fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).strong().size(16.0));
    ui.add_space(2.0);
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

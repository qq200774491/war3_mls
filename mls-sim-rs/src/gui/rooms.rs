use std::collections::HashMap;
use std::path::PathBuf;

use crate::player::Player;
use crate::room::RoomStatus;
use crate::storage;

use eframe::egui;

use super::{format_duration, section_heading, GuiApp};

struct RoomInfo {
    id: String,
    status: RoomStatus,
    script_dir: String,
    mode_id: i32,
    game_time: i64,
    error_msg: String,
    players: Vec<PlayerInfo>,
}

struct PlayerInfo {
    index: i32,
    name: String,
    map_level: i32,
    is_connected: bool,
}

fn status_label(status: &RoomStatus) -> (&'static str, egui::Color32) {
    match status {
        RoomStatus::Running => ("运行中", egui::Color32::from_rgb(75, 215, 105)),
        RoomStatus::Stopped => ("已停止", egui::Color32::from_rgb(140, 148, 165)),
        RoomStatus::Error => ("错误", egui::Color32::from_rgb(255, 95, 95)),
        RoomStatus::Created => ("已创建", egui::Color32::from_rgb(105, 165, 255)),
    }
}

impl GuiApp {
    pub(crate) fn rooms_tab(&mut self, ctx: &egui::Context) {
        let rooms: Vec<RoomInfo> = {
            let manager = self.manager.read().unwrap();
            let mut list: Vec<RoomInfo> = manager
                .rooms
                .iter()
                .map(|(id, room)| {
                    let shared = room.shared.read().unwrap();
                    let game_time = if shared.loaded_ts > 0 {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                            - shared.loaded_ts
                    } else {
                        0
                    };
                    RoomInfo {
                        id: id.clone(),
                        status: shared.status.clone(),
                        script_dir: shared.script_dir.to_string_lossy().to_string(),
                        mode_id: shared.mode_id,
                        game_time,
                        error_msg: shared.error_message.clone(),
                        players: {
                            let mut ps: Vec<PlayerInfo> = shared
                                .players
                                .iter()
                                .map(|(idx, p)| PlayerInfo {
                                    index: *idx,
                                    name: p.name.clone(),
                                    map_level: p.map_level,
                                    is_connected: p.is_connected,
                                })
                                .collect();
                            ps.sort_by_key(|p| p.index);
                            ps
                        },
                    }
                })
                .collect();
            list.sort_by(|a, b| a.id.cmp(&b.id));
            list
        };

        let mut delete_room: Option<String> = None;

        // ── Sidebar ──
        egui::SidePanel::left("room_sidebar")
            .default_width(220.0)
            .resizable(true)
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    section_heading(ui, "房间列表");
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui: &mut egui::Ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", rooms.len()))
                                    .small()
                                    .color(egui::Color32::from_rgb(140, 148, 165)),
                            );
                        },
                    );
                });

                ui.add_space(4.0);
                let btn_text = if self.show_create_room {
                    "收起"
                } else {
                    "新建房间"
                };
                let btn = egui::Button::new(btn_text);
                if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                    self.show_create_room = !self.show_create_room;
                }

                if self.show_create_room {
                    ui.add_space(4.0);
                    ui.group(|ui: &mut egui::Ui| {
                        ui.label("脚本目录:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_room_script_dir)
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(2.0);
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label("模式ID:");
                            ui.add(egui::DragValue::new(&mut self.new_room_mode_id));
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui: &mut egui::Ui| {
                            if ui.button("创建").clicked() && !self.new_room_script_dir.is_empty()
                            {
                                let script_dir = PathBuf::from(&self.new_room_script_dir);
                                if script_dir.is_dir() {
                                    let archive_dir =
                                        self.config.read().unwrap().archive_dir.clone();
                                    let mut players = HashMap::new();
                                    players.insert(0, Player::new(0, "Player_0".into()));
                                    storage::apply_saved_archives(
                                        &archive_dir,
                                        &self.new_room_script_dir,
                                        &mut players,
                                    );
                                    let room_id = self.manager.write().unwrap().create_room(
                                        script_dir,
                                        self.new_room_mode_id,
                                        players,
                                        archive_dir,
                                    );
                                    self.selected_room_id = Some(room_id);
                                    self.show_create_room = false;
                                }
                            }
                            if ui.button("取消").clicked() {
                                self.show_create_room = false;
                            }
                        });
                    });
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                egui::ScrollArea::vertical().show(ui, |ui: &mut egui::Ui| {
                    if rooms.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(140, 148, 165), "暂无房间");
                    }
                    for room in &rooms {
                        let selected = self.selected_room_id.as_ref() == Some(&room.id);
                        let (status_text, color) = status_label(&room.status);

                        let resp = ui.vertical(|ui: &mut egui::Ui| {
                            let resp = ui
                                .selectable_label(selected, &room.id)
                                .on_hover_text(&room.script_dir);
                            ui.colored_label(
                                color,
                                egui::RichText::new(format!(
                                    "  {} | {} 人",
                                    status_text,
                                    room.players.len()
                                ))
                                .size(12.0),
                            );
                            resp
                        });
                        if resp.inner.clicked() {
                            self.selected_room_id = Some(room.id.clone());
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        // ── Detail Panel ──
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui: &mut egui::Ui| {
                let room_id = match self.selected_room_id.clone() {
                    Some(id) => id,
                    None => {
                        ui.vertical_centered(|ui: &mut egui::Ui| {
                            ui.add_space(ui.available_height() * 0.35);
                            ui.label(
                                egui::RichText::new("请选择或创建一个房间")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(140, 148, 165)),
                            );
                        });
                        return;
                    }
                };

                let room_info = rooms.iter().find(|r| r.id == room_id);
                let room_info = match room_info {
                    Some(r) => r,
                    None => {
                        self.selected_room_id = None;
                        ui.label("房间不存在");
                        return;
                    }
                };

                egui::ScrollArea::vertical().show(ui, |ui: &mut egui::Ui| {
                    // ── Section: Room Info ──
                    ui.group(|ui: &mut egui::Ui| {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(egui::RichText::new(&room_id).strong().size(16.0));
                            ui.add_space(8.0);
                            let (status_text, color) = status_label(&room_info.status);
                            ui.colored_label(color, egui::RichText::new(status_text).size(13.0));
                        });
                        ui.add_space(4.0);

                        egui::Grid::new("room_info_grid")
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.label(
                                    egui::RichText::new("脚本:")
                                        .color(egui::Color32::from_rgb(140, 148, 165)),
                                );
                                ui.label(&room_info.script_dir);
                                ui.end_row();
                                ui.label(
                                    egui::RichText::new("模式:")
                                        .color(egui::Color32::from_rgb(140, 148, 165)),
                                );
                                ui.label(room_info.mode_id.to_string());
                                ui.end_row();
                                ui.label(
                                    egui::RichText::new("运行时间:")
                                        .color(egui::Color32::from_rgb(140, 148, 165)),
                                );
                                ui.label(format_duration(room_info.game_time));
                                ui.end_row();
                            });

                        if !room_info.error_msg.is_empty() {
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 95, 95),
                                format!("错误: {}", room_info.error_msg),
                            );
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui: &mut egui::Ui| {
                            if ui.button("停止").clicked() {
                                let manager = self.manager.read().unwrap();
                                if let Some(room) = manager.rooms.get(&room_id) {
                                    room.stop("UserStop".into());
                                }
                            }
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new("删除")
                                        .color(egui::Color32::from_rgb(255, 105, 105)),
                                ))
                                .clicked()
                            {
                                delete_room = Some(room_id.clone());
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // ── Section: Players ──
                    ui.group(|ui: &mut egui::Ui| {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            section_heading(ui, "玩家列表");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui: &mut egui::Ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} 人",
                                            room_info.players.len()
                                        ))
                                        .small()
                                        .color(egui::Color32::from_rgb(140, 148, 165)),
                                    );
                                },
                            );
                        });

                        let mut remove_player: Option<i32> = None;
                        let mut leave_player: Option<i32> = None;
                        let mut join_player: Option<i32> = None;

                        egui::Grid::new("players_grid")
                            .striped(true)
                            .min_col_width(50.0)
                            .spacing([12.0, 6.0])
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.label(egui::RichText::new("槽位").strong());
                                ui.label(egui::RichText::new("名称").strong());
                                ui.label(egui::RichText::new("等级").strong());
                                ui.label(egui::RichText::new("状态").strong());
                                ui.label(egui::RichText::new("操作").strong());
                                ui.end_row();

                                for p in &room_info.players {
                                    ui.label(p.index.to_string());
                                    ui.label(&p.name);
                                    ui.label(p.map_level.to_string());
                                    if p.is_connected {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(75, 215, 105),
                                            "在线",
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(140, 148, 165),
                                            "离线",
                                        );
                                    }
                                    ui.horizontal(|ui: &mut egui::Ui| {
                                        if p.is_connected {
                                            if ui.small_button("离线").clicked() {
                                                leave_player = Some(p.index);
                                            }
                                        } else if ui.small_button("上线").clicked() {
                                            join_player = Some(p.index);
                                        }
                                        if ui.small_button("移除").clicked() {
                                            remove_player = Some(p.index);
                                        }
                                    });
                                    ui.end_row();
                                }
                            });

                        if let Some(idx) = remove_player {
                            let manager = self.manager.read().unwrap();
                            if let Some(room) = manager.rooms.get(&room_id) {
                                room.exit_player(idx, "Logout".into());
                            }
                        }
                        if let Some(idx) = leave_player {
                            let manager = self.manager.read().unwrap();
                            if let Some(room) = manager.rooms.get(&room_id) {
                                room.leave_player(idx, "Disconnect".into());
                            }
                        }
                        if let Some(idx) = join_player {
                            let manager = self.manager.read().unwrap();
                            if let Some(room) = manager.rooms.get(&room_id) {
                                let name = room
                                    .shared
                                    .read()
                                    .unwrap()
                                    .players
                                    .get(&idx)
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| format!("Player_{}", idx));
                                room.join_player(idx, name, "Connect".into());
                            }
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label("槽位:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.add_player_index)
                                    .desired_width(35.0),
                            );
                            ui.label("名称:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.add_player_name)
                                    .desired_width(100.0),
                            );
                            if ui.button("添加").clicked() {
                                if let Ok(idx) = self.add_player_index.parse::<i32>() {
                                    let name = if self.add_player_name.is_empty() {
                                        format!("Player_{}", idx)
                                    } else {
                                        self.add_player_name.clone()
                                    };
                                    let manager = self.manager.read().unwrap();
                                    if let Some(room) = manager.rooms.get(&room_id) {
                                        room.join_player(idx, name, "Connect".into());
                                    }
                                    self.add_player_name.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // ── Section: Send Event ──
                    ui.group(|ui: &mut egui::Ui| {
                        section_heading(ui, "发送事件");

                        egui::Grid::new("event_form_grid")
                            .num_columns(2)
                            .spacing([8.0, 6.0])
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.label("事件名:");
                                ui.horizontal(|ui: &mut egui::Ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.event_name)
                                            .desired_width(200.0),
                                    );
                                    ui.label("玩家:");
                                    let player_text = if self.event_player_idx < 0 {
                                        "全部".to_string()
                                    } else {
                                        format!("{}", self.event_player_idx)
                                    };
                                    egui::ComboBox::from_id_salt("event_player")
                                        .selected_text(player_text)
                                        .width(80.0)
                                        .show_ui(ui, |ui: &mut egui::Ui| {
                                            ui.selectable_value(
                                                &mut self.event_player_idx,
                                                -1,
                                                "全部",
                                            );
                                            for p in &room_info.players {
                                                ui.selectable_value(
                                                    &mut self.event_player_idx,
                                                    p.index,
                                                    format!("{} ({})", p.index, p.name),
                                                );
                                            }
                                        });
                                });
                                ui.end_row();

                                ui.label("数据:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.event_data)
                                        .desired_width(f32::INFINITY),
                                );
                                ui.end_row();
                            });

                        ui.add_space(4.0);
                        if ui.button("发送").clicked() && !self.event_name.is_empty() {
                            {
                                let manager = self.manager.read().unwrap();
                                if let Some(room) = manager.rooms.get(&room_id) {
                                    let errnu = room.send_event(
                                        self.event_name.clone(),
                                        self.event_data.clone(),
                                        self.event_player_idx,
                                    );
                                    if errnu != crate::room::ERR_OK {
                                        tracing::warn!("Send event failed: errnu={}", errnu);
                                    }
                                }
                            }
                            self.event_name.clear();
                            self.event_data.clear();
                        }
                    });
                });
            });

        if let Some(id) = delete_room {
            self.manager.write().unwrap().destroy_room(&id);
            if self.selected_room_id.as_ref() == Some(&id) {
                self.selected_room_id = None;
            }
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;

use crate::player::Player;
use crate::room::RoomStatus;
use crate::storage;

use eframe::egui;

use super::{format_duration, GuiApp};

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

impl GuiApp {
    pub(crate) fn rooms_tab(&mut self, ctx: &egui::Context) {
        let rooms: Vec<RoomInfo> = {
            let manager = self.manager.read().unwrap();
            manager
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
                        players: shared
                            .players
                            .iter()
                            .map(|(idx, p)| PlayerInfo {
                                index: *idx,
                                name: p.name.clone(),
                                map_level: p.map_level,
                                is_connected: p.is_connected,
                            })
                            .collect(),
                    }
                })
                .collect()
        };

        let mut delete_room: Option<String> = None;

        egui::SidePanel::left("room_sidebar")
            .default_width(200.0)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.heading("房间列表");

                if ui.button("+ 新建房间").clicked() {
                    self.show_create_room = !self.show_create_room;
                }

                if self.show_create_room {
                    ui.separator();
                    ui.label("脚本目录:");
                    ui.text_edit_singleline(&mut self.new_room_script_dir);
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.label("模式ID:");
                        ui.add(egui::DragValue::new(&mut self.new_room_mode_id));
                    });
                    ui.horizontal(|ui: &mut egui::Ui| {
                        if ui.button("创建").clicked() && !self.new_room_script_dir.is_empty() {
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
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui: &mut egui::Ui| {
                    for room in &rooms {
                        let selected = self.selected_room_id.as_ref() == Some(&room.id);
                        let (icon, color) = match room.status {
                            RoomStatus::Running => {
                                ("●", egui::Color32::from_rgb(80, 200, 80))
                            }
                            RoomStatus::Stopped => ("○", egui::Color32::GRAY),
                            RoomStatus::Error => {
                                ("●", egui::Color32::from_rgb(255, 80, 80))
                            }
                            RoomStatus::Created => {
                                ("○", egui::Color32::from_rgb(100, 150, 255))
                            }
                        };
                        let text = egui::RichText::new(format!("{} {}", icon, room.id)).color(color);
                        if ui.selectable_label(selected, text).clicked() {
                            self.selected_room_id = Some(room.id.clone());
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let room_id = match self.selected_room_id.clone() {
                Some(id) => id,
                None => {
                    ui.centered_and_justified(|ui: &mut egui::Ui| {
                        ui.label("请选择或创建一个房间");
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

            // Room header
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.heading(&room_id);
                let (status_text, color) = match room_info.status {
                    RoomStatus::Running => ("运行中", egui::Color32::from_rgb(80, 200, 80)),
                    RoomStatus::Stopped => ("已停止", egui::Color32::GRAY),
                    RoomStatus::Error => ("错误", egui::Color32::from_rgb(255, 80, 80)),
                    RoomStatus::Created => ("已创建", egui::Color32::from_rgb(100, 150, 255)),
                };
                ui.colored_label(color, status_text);
            });
            ui.label(format!("脚本: {}", room_info.script_dir));
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(format!("模式: {}", room_info.mode_id));
                ui.label(format!("运行时间: {}", format_duration(room_info.game_time)));
            });
            if !room_info.error_msg.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 80, 80),
                    format!("错误: {}", room_info.error_msg),
                );
            }

            // Control buttons
            ui.horizontal(|ui: &mut egui::Ui| {
                if ui.button("■ 停止").clicked() {
                    let manager = self.manager.read().unwrap();
                    if let Some(room) = manager.rooms.get(&room_id) {
                        room.stop("UserStop".into());
                    }
                }
                if ui.button("✕ 删除").clicked() {
                    delete_room = Some(room_id.clone());
                }
            });

            ui.separator();

            // Players section
            ui.heading("玩家列表");

            egui::Grid::new("players_grid")
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui: &mut egui::Ui| {
                    ui.strong("槽位");
                    ui.strong("名称");
                    ui.strong("等级");
                    ui.strong("状态");
                    ui.strong("操作");
                    ui.end_row();

                    let mut remove_player: Option<i32> = None;
                    let mut leave_player: Option<i32> = None;
                    let mut join_player: Option<i32> = None;

                    for p in &room_info.players {
                        ui.label(p.index.to_string());
                        ui.label(&p.name);
                        ui.label(p.map_level.to_string());
                        if p.is_connected {
                            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "在线");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "离线");
                        }
                        ui.horizontal(|ui: &mut egui::Ui| {
                            if p.is_connected {
                                if ui.small_button("离线").clicked() {
                                    leave_player = Some(p.index);
                                }
                            } else {
                                if ui.small_button("上线").clicked() {
                                    join_player = Some(p.index);
                                }
                            }
                            if ui.small_button("移除").clicked() {
                                remove_player = Some(p.index);
                            }
                        });
                        ui.end_row();
                    }

                    // Apply player actions
                    if let Some(idx) = remove_player {
                        let manager = self.manager.read().unwrap();
                        if let Some(room) = manager.rooms.get(&room_id) {
                            room.shared.write().unwrap().players.remove(&idx);
                        }
                    }
                    if let Some(idx) = leave_player {
                        let manager = self.manager.read().unwrap();
                        if let Some(room) = manager.rooms.get(&room_id) {
                            if let Some(p) = room.shared.write().unwrap().players.get_mut(&idx) {
                                p.is_connected = false;
                            }
                            let data = serde_json::json!({"reason": "Disconnect"}).to_string();
                            room.send_event("_playerleave".into(), data, idx);
                        }
                    }
                    if let Some(idx) = join_player {
                        let manager = self.manager.read().unwrap();
                        if let Some(room) = manager.rooms.get(&room_id) {
                            if let Some(p) = room.shared.write().unwrap().players.get_mut(&idx) {
                                p.is_connected = true;
                            }
                            let data = serde_json::json!({"reason": "Connect"}).to_string();
                            room.send_event("_playerjoin".into(), data, idx);
                        }
                    }
                });

            // Add player form
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("槽位:");
                ui.add(egui::TextEdit::singleline(&mut self.add_player_index).desired_width(40.0));
                ui.label("名称:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_player_name).desired_width(100.0),
                );
                if ui.button("+ 添加玩家").clicked() {
                    if let Ok(idx) = self.add_player_index.parse::<i32>() {
                        let name = if self.add_player_name.is_empty() {
                            format!("Player_{}", idx)
                        } else {
                            self.add_player_name.clone()
                        };
                        let manager = self.manager.read().unwrap();
                        if let Some(room) = manager.rooms.get(&room_id) {
                            room.shared
                                .write()
                                .unwrap()
                                .players
                                .insert(idx, Player::new(idx, name));
                        }
                        self.add_player_name.clear();
                    }
                }
            });

            ui.separator();

            // Send event form
            ui.heading("发送事件");
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("事件名:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_name).desired_width(150.0),
                );
                ui.label("玩家:");
                let player_text = if self.event_player_idx < 0 {
                    "全部".to_string()
                } else {
                    format!("{}", self.event_player_idx)
                };
                egui::ComboBox::from_id_salt("event_player")
                    .selected_text(player_text)
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(&mut self.event_player_idx, -1, "全部");
                        for p in &room_info.players {
                            ui.selectable_value(
                                &mut self.event_player_idx,
                                p.index,
                                format!("{} ({})", p.index, p.name),
                            );
                        }
                    });
            });
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("数据:  ");
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_data).desired_width(400.0),
                );
            });
            if ui.button("发送").clicked() && !self.event_name.is_empty() {
                {
                    let manager = self.manager.read().unwrap();
                    if let Some(room) = manager.rooms.get(&room_id) {
                        room.send_event(
                            self.event_name.clone(),
                            self.event_data.clone(),
                            self.event_player_idx,
                        );
                    }
                }
                self.event_name.clear();
                self.event_data.clear();
            }
        });

        if let Some(id) = delete_room {
            self.manager.write().unwrap().destroy_room(&id);
            if self.selected_room_id.as_ref() == Some(&id) {
                self.selected_room_id = None;
            }
        }
    }
}

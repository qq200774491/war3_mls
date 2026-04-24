use eframe::egui;

use super::GuiApp;

impl GuiApp {
    pub(crate) fn state_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let room_ids: Vec<String> = {
                let manager = self.manager.read().unwrap();
                manager.rooms.keys().cloned().collect()
            };

            ui.horizontal(|ui: &mut egui::Ui| {
                ui.heading("房间状态");
                ui.separator();

                ui.label("房间:");
                let selected_text = self
                    .state_room_id
                    .as_deref()
                    .unwrap_or("选择房间");
                egui::ComboBox::from_id_salt("state_room_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        for id in &room_ids {
                            let is_selected = self.state_room_id.as_ref() == Some(id);
                            if ui.selectable_label(is_selected, id).clicked() {
                                self.state_room_id = Some(id.clone());
                                self.refresh_state_json();
                            }
                        }
                    });

                if ui.button("刷新").clicked() {
                    self.refresh_state_json();
                }
            });

            ui.separator();

            if self.state_room_id.is_none() {
                ui.label("请选择一个房间查看状态");
                return;
            }

            egui::ScrollArea::both().show(ui, |ui: &mut egui::Ui| {
                let mut text = self.state_json_text.as_str();
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30),
                );
            });
        });
    }

    fn refresh_state_json(&mut self) {
        if let Some(room_id) = &self.state_room_id {
            let manager = self.manager.read().unwrap();
            if let Some(room) = manager.rooms.get(room_id) {
                let shared = room.shared.read().unwrap();
                let json = shared.to_json();
                self.state_json_text =
                    serde_json::to_string_pretty(&json).unwrap_or_else(|e| format!("Error: {}", e));
            } else {
                self.state_json_text = "房间不存在".to_string();
                self.state_room_id = None;
            }
        }
    }
}

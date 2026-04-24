use eframe::egui;

use super::{format_time, GuiApp};

impl GuiApp {
    pub(crate) fn console_tab(&mut self, ctx: &egui::Context) {
        let room_ids: Vec<String> = self.subscribed_rooms.iter().cloned().collect();

        // Filter bar
        egui::TopBottomPanel::top("console_filter").show(ctx, |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("级别:");
                let levels = [("全部", ""), ("DBG", "DBG"), ("INF", "INF"), ("ERR", "ERR")];
                for (label, value) in &levels {
                    if ui
                        .selectable_label(self.log_level_filter == *value, *label)
                        .clicked()
                    {
                        self.log_level_filter = value.to_string();
                    }
                }

                ui.separator();

                ui.label("房间:");
                egui::ComboBox::from_id_salt("log_room_filter")
                    .selected_text(if self.log_room_filter.is_empty() {
                        "全部"
                    } else {
                        &self.log_room_filter
                    })
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(&mut self.log_room_filter, String::new(), "全部");
                        for id in &room_ids {
                            ui.selectable_value(&mut self.log_room_filter, id.clone(), id);
                        }
                    });

                ui.separator();

                ui.label("搜索:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.log_search).desired_width(150.0),
                );

                ui.separator();

                ui.checkbox(&mut self.auto_scroll, "自动滚动");

                if ui.button("清空").clicked() {
                    self.logs.clear();
                    self.out_events.clear();
                }
            });
        });

        // Out events panel at bottom
        egui::TopBottomPanel::bottom("out_events_panel")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.heading("出站事件");
                let filtered_events: Vec<_> = self
                    .out_events
                    .iter()
                    .filter(|e| {
                        if !self.log_room_filter.is_empty() && e.room_id != self.log_room_filter {
                            return false;
                        }
                        true
                    })
                    .collect();

                egui::ScrollArea::vertical()
                    .id_salt("out_events_scroll")
                    .stick_to_bottom(true)
                    .show(ui, |ui: &mut egui::Ui| {
                        egui::Grid::new("out_events_grid")
                            .striped(true)
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.strong("时间");
                                ui.strong("房间");
                                ui.strong("玩家");
                                ui.strong("事件");
                                ui.strong("数据");
                                ui.end_row();

                                let start = if filtered_events.len() > 200 {
                                    filtered_events.len() - 200
                                } else {
                                    0
                                };
                                for ev in &filtered_events[start..] {
                                    ui.label(format_time(ev.timestamp));
                                    ui.label(&ev.room_id);
                                    ui.label(
                                        if ev.player_index < 0 {
                                            "全部".to_string()
                                        } else {
                                            ev.player_index.to_string()
                                        },
                                    );
                                    ui.label(&ev.ename);
                                    ui.label(truncate(&ev.evalue, 60));
                                    ui.end_row();
                                }
                            });
                    });
            });

        // Log entries in center
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let filtered_logs: Vec<_> = self
                .logs
                .iter()
                .filter(|l| {
                    if !self.log_level_filter.is_empty() && l.level != self.log_level_filter {
                        return false;
                    }
                    if !self.log_room_filter.is_empty() && l.room_id != self.log_room_filter {
                        return false;
                    }
                    if !self.log_search.is_empty()
                        && !l
                            .message
                            .to_lowercase()
                            .contains(&self.log_search.to_lowercase())
                    {
                        return false;
                    }
                    true
                })
                .collect();

            let row_height = 18.0;
            let total = filtered_logs.len();

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(self.auto_scroll)
                .show_rows(ui, row_height, total, |ui: &mut egui::Ui, row_range| {
                    for i in row_range {
                        let log = filtered_logs[i];
                        let color = match log.level.as_str() {
                            "ERR" => egui::Color32::from_rgb(255, 100, 100),
                            "DBG" => egui::Color32::GRAY,
                            _ => ui.visuals().text_color(),
                        };
                        let text = format!(
                            "{} [{}] [{}] {}",
                            format_time(log.timestamp),
                            log.level,
                            log.room_id,
                            log.message
                        );
                        ui.colored_label(color, text);
                    }
                });

            ui.allocate_space(ui.available_size());
        });
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

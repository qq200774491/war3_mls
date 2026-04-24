use eframe::egui;

use super::{format_time, section_heading, GuiApp};

impl GuiApp {
    pub(crate) fn events_tab(&mut self, ctx: &egui::Context) {
        let room_ids: Vec<String> = {
            let mut ids: Vec<String> = self.subscribed_rooms.iter().cloned().collect();
            ids.sort();
            ids
        };

        // ── Filter bar ──
        egui::TopBottomPanel::top("events_filter")
            .frame(
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    section_heading(ui, "出站事件");

                    ui.add_space(12.0);

                    ui.label(egui::RichText::new("房间:").strong());
                    egui::ComboBox::from_id_salt("event_room_filter")
                        .selected_text(if self.log_room_filter.is_empty() {
                            "全部"
                        } else {
                            &self.log_room_filter
                        })
                        .width(100.0)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            ui.selectable_value(&mut self.log_room_filter, String::new(), "全部");
                            for id in &room_ids {
                                ui.selectable_value(&mut self.log_room_filter, id.clone(), id);
                            }
                        });

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui: &mut egui::Ui| {
                            if ui.button("清空").clicked() {
                                self.out_events.clear();
                            }
                            ui.label(
                                egui::RichText::new(format!("{} 条", self.out_events.len()))
                                    .small()
                                    .color(egui::Color32::from_rgb(140, 148, 165)),
                            );
                        },
                    );
                });
            });

        // ── Events table ──
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui: &mut egui::Ui| {
                let filtered: Vec<_> = self
                    .out_events
                    .iter()
                    .filter(|e| {
                        self.log_room_filter.is_empty() || e.room_id == self.log_room_filter
                    })
                    .collect();

                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui: &mut egui::Ui| {
                        egui::Grid::new("events_full_grid")
                            .striped(true)
                            .spacing([14.0, 5.0])
                            .min_col_width(50.0)
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.label(egui::RichText::new("时间").strong());
                                ui.label(egui::RichText::new("房间").strong());
                                ui.label(egui::RichText::new("玩家").strong());
                                ui.label(egui::RichText::new("事件名").strong());
                                ui.label(egui::RichText::new("数据").strong());
                                ui.end_row();

                                let start = filtered.len().saturating_sub(500);
                                for ev in &filtered[start..] {
                                    ui.label(
                                        egui::RichText::new(format_time(ev.timestamp))
                                            .monospace()
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(140, 148, 165)),
                                    );
                                    ui.label(&ev.room_id);
                                    ui.label(if ev.player_index < 0 {
                                        "全部".to_string()
                                    } else {
                                        ev.player_index.to_string()
                                    });
                                    ui.label(
                                        egui::RichText::new(&ev.ename)
                                            .color(egui::Color32::from_rgb(140, 195, 255)),
                                    );
                                    ui.label(
                                        egui::RichText::new(truncate(&ev.evalue, 80))
                                            .monospace()
                                            .size(13.0),
                                    );
                                    ui.end_row();
                                }
                            });
                    });
            });
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

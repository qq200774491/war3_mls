use eframe::egui;

use super::{section_heading, GuiApp};

impl GuiApp {
    pub(crate) fn settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.set_max_width(500.0);

                // ── Bridge ──
                ui.group(|ui: &mut egui::Ui| {
                    section_heading(ui, "Bridge 服务");
                    ui.add_space(2.0);

                    egui::Grid::new("settings_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.label("监听地址:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.settings_host)
                                    .desired_width(180.0),
                            );
                            ui.end_row();

                            ui.label("端口:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.settings_port)
                                    .desired_width(80.0),
                            );
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // ── Storage ──
                ui.group(|ui: &mut egui::Ui| {
                    section_heading(ui, "存档");
                    ui.add_space(2.0);

                    egui::Grid::new("storage_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.label("存档目录:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.settings_archive_dir)
                                    .desired_width(280.0),
                            );
                            ui.end_row();
                        });
                });

                ui.add_space(12.0);

                ui.horizontal(|ui: &mut egui::Ui| {
                    if ui.button("保存配置").clicked() {
                        let port: u16 = self.settings_port.parse().unwrap_or(5000);
                        let mut cfg = self.config.write().unwrap();
                        cfg.host = self.settings_host.clone();
                        cfg.port = port;
                        cfg.archive_dir = self.settings_archive_dir.clone();
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64();
                        match cfg.save(&self.config_path) {
                            Ok(_) => {
                                self.save_msg = Some(("配置已保存".into(), false, now));
                            }
                            Err(e) => {
                                self.save_msg =
                                    Some((format!("保存失败: {}", e), true, now));
                            }
                        }
                    }

                    if let Some((ref msg, is_err, ts)) = self.save_msg {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64();
                        if now - ts < 3.0 {
                            let color = if is_err {
                                egui::Color32::from_rgb(255, 95, 95)
                            } else {
                                egui::Color32::from_rgb(75, 215, 105)
                            };
                            ui.add_space(8.0);
                            ui.colored_label(color, msg);
                        }
                    }
                });

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(8.0);

                // ── About ──
                ui.group(|ui: &mut egui::Ui| {
                    section_heading(ui, "关于");
                    ui.add_space(2.0);
                    ui.label(format!(
                        "MLS 云脚本模拟器 v{}",
                        env!("CARGO_PKG_VERSION")
                    ));
                    ui.label("本地 Lua 云脚本测试环境");
                    ui.add_space(4.0);
                    ui.label(format!("参与者: {}", env!("CARGO_PKG_AUTHORS")));
                    ui.hyperlink_to(
                        "GitHub 仓库",
                        env!("CARGO_PKG_REPOSITORY"),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Bridge API: http://{}:{}/api/bridge/",
                            self.settings_host, self.settings_port
                        ))
                        .monospace()
                        .size(12.0),
                    );
                });
            });
    }
}

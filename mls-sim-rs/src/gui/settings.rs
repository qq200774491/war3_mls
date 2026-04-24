use eframe::egui;

use super::GuiApp;

impl GuiApp {
    pub(crate) fn settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.heading("服务设置");
            ui.add_space(10.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui: &mut egui::Ui| {
                    ui.label("监听地址:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings_host).desired_width(200.0),
                    );
                    ui.end_row();

                    ui.label("端口:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings_port).desired_width(80.0),
                    );
                    ui.end_row();

                    ui.label("存档目录:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings_archive_dir)
                            .desired_width(300.0),
                    );
                    ui.end_row();
                });

            ui.add_space(20.0);

            if ui.button("保存配置").clicked() {
                let port: u16 = self.settings_port.parse().unwrap_or(5000);
                let mut cfg = self.config.write().unwrap();
                cfg.host = self.settings_host.clone();
                cfg.port = port;
                cfg.archive_dir = self.settings_archive_dir.clone();
                match cfg.save(&self.config_path) {
                    Ok(_) => {
                        ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "配置已保存");
                    }
                    Err(e) => {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 80, 80),
                            format!("保存失败: {}", e),
                        );
                    }
                }
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("关于");
            ui.label(format!(
                "MLS 云脚本模拟器 v{}",
                env!("CARGO_PKG_VERSION")
            ));
            ui.label("本地 Lua 云脚本测试环境");
            ui.label(format!(
                "Bridge API: http://{}:{}/api/bridge/",
                self.settings_host,
                self.settings_port
            ));
        });
    }
}

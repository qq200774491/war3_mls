use std::collections::HashMap;

use eframe::egui;

use crate::room::{ProfileData, ProfileNode};

use super::{section_heading, GuiApp};

const DIM: egui::Color32 = egui::Color32::from_rgb(140, 148, 165);
const BAR_H: f32 = 20.0;
const BAR_GAP: f32 = 1.0;
const MIN_BAR_W: f32 = 1.5;

struct HotFunc {
    name: String,
    id: String,
    self_count: u64,
    total_count: u64,
}

fn collect_hot_funcs(node: &ProfileNode, map: &mut HashMap<String, HotFunc>) {
    if node.id != "(root)" {
        let e = map.entry(node.id.clone()).or_insert(HotFunc {
            name: node.name.clone(),
            id: node.id.clone(),
            self_count: 0,
            total_count: 0,
        });
        e.self_count += node.self_count;
        e.total_count += node.count;
    }
    for c in &node.children {
        collect_hot_funcs(c, map);
    }
}

impl GuiApp {
    pub(crate) fn profiler_tab(&mut self, ctx: &egui::Context) {
        let room_ids: Vec<String> = {
            let mgr = self.manager.read().unwrap();
            let mut ids: Vec<String> = mgr.rooms.keys().cloned().collect();
            ids.sort();
            ids
        };

        // ── Toolbar ──
        egui::TopBottomPanel::top("profiler_toolbar")
            .frame(
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    section_heading(ui, "性能分析");
                    ui.add_space(12.0);

                    ui.label(egui::RichText::new("房间:").strong());
                    let sel = self.profiler_room_id.as_deref().unwrap_or("--");
                    egui::ComboBox::from_id_salt("prof_room")
                        .selected_text(sel)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for id in &room_ids {
                                let on = self.profiler_room_id.as_ref() == Some(id);
                                if ui.selectable_label(on, id).clicked() {
                                    self.profiler_room_id = Some(id.clone());
                                }
                            }
                        });

                    ui.separator();

                    ui.label(egui::RichText::new("频率:").strong());
                    ui.add(
                        egui::DragValue::new(&mut self.profiler_hook_count)
                            .range(100..=500_000)
                            .speed(100),
                    );
                    ui.label("指令");

                    ui.label(egui::RichText::new("窗口:").strong());
                    ui.add(
                        egui::DragValue::new(&mut self.profiler_window)
                            .range(1..=120)
                            .speed(1),
                    );
                    ui.label("秒");

                    ui.label(egui::RichText::new("帧时:").strong());
                    ui.add(
                        egui::DragValue::new(&mut self.profiler_frame_ms)
                            .range(1.0..=1000.0)
                            .speed(1.0)
                            .suffix(" ms"),
                    );
                });

                let room_id = self.profiler_room_id.clone();
                if let Some(ref rid) = room_id {
                    let (available, running, data) = {
                        let mgr = self.manager.read().unwrap();
                        if let Some(r) = mgr.rooms.get(rid) {
                            let s = r.shared.read().unwrap();
                            (s.profiler_available, s.profiler_running, s.profile_data.clone())
                        } else {
                            (false, false, None)
                        }
                    };

                    if available {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            if running {
                                let t = ui.ctx().input(|i| i.time);
                                let pulse = (t * 3.0).sin() * 0.5 + 0.5;
                                let r = (200.0 + 55.0 * pulse) as u8;
                                let g = (40.0 * (1.0 - pulse)) as u8;
                                ui.label(egui::RichText::new("●").color(
                                    egui::Color32::from_rgb(r, g, g),
                                ));
                                ui.label(egui::RichText::new("采样中").strong());

                                if ui
                                    .button(
                                        egui::RichText::new("■ 停止")
                                            .color(egui::Color32::from_rgb(255, 140, 140)),
                                    )
                                    .clicked()
                                {
                                    let mgr = self.manager.read().unwrap();
                                    if let Some(r) = mgr.rooms.get(rid) {
                                        r.profiler_stop();
                                    }
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new("●")
                                        .color(egui::Color32::from_rgb(80, 80, 80)),
                                );
                                ui.label("已停止");

                                if ui
                                    .button(
                                        egui::RichText::new("▶ 开始")
                                            .color(egui::Color32::from_rgb(120, 230, 140)),
                                    )
                                    .clicked()
                                {
                                    let mgr = self.manager.read().unwrap();
                                    if let Some(r) = mgr.rooms.get(rid) {
                                        r.profiler_start(
                                            self.profiler_hook_count,
                                            self.profiler_window,
                                        );
                                    }
                                }
                            }

                            if ui.button("↺ 重置").clicked() {
                                let mgr = self.manager.read().unwrap();
                                if let Some(r) = mgr.rooms.get(rid) {
                                    r.profiler_reset();
                                }
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if let Some(ref d) = data {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "采样 {}  |  时间桶 {}  |  窗口 {}s",
                                                format_count(d.total_samples),
                                                d.bucket_count,
                                                d.window,
                                            ))
                                            .small()
                                            .color(DIM),
                                        );
                                    }
                                },
                            );
                        });
                    }
                }
            });

        // ── Resolve profile data for remaining panels ──
        let (profile_data, room_ok) = match &self.profiler_room_id {
            Some(rid) => {
                let mgr = self.manager.read().unwrap();
                if let Some(r) = mgr.rooms.get(rid) {
                    let s = r.shared.read().unwrap();
                    if !s.profiler_available {
                        (None, false)
                    } else {
                        (s.profile_data.clone(), true)
                    }
                } else {
                    (None, false)
                }
            }
            None => (None, false),
        };

        let has_data = profile_data
            .as_ref()
            .map_or(false, |d| d.total_samples > 0);

        // ── Hot functions table (bottom) ──
        if has_data {
            egui::TopBottomPanel::bottom("profiler_hot")
                .default_height(200.0)
                .resizable(true)
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                        .fill(ctx.style().visuals.panel_fill),
                )
                .show(ctx, |ui| {
                    let data = profile_data.as_ref().unwrap();
                    let total = data.root.count.max(1) as f64;
                    let frame_ms = self.profiler_frame_ms as f64;

                    let mut map = HashMap::new();
                    collect_hot_funcs(&data.root, &mut map);
                    let mut funcs: Vec<HotFunc> = map.into_values().collect();
                    funcs.sort_by(|a, b| b.self_count.cmp(&a.self_count));

                    ui.horizontal(|ui| {
                        section_heading(ui, "热点函数");
                        ui.label(
                            egui::RichText::new(format!("({} 个函数)", funcs.len()))
                                .small()
                                .color(DIM),
                        );
                    });

                    ui.add_space(2.0);

                    let col_head = |ui: &mut egui::Ui, text: &str| {
                        ui.label(egui::RichText::new(text).strong().size(13.0));
                    };

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            egui::Grid::new("hot_funcs_grid")
                                .striped(true)
                                .min_col_width(40.0)
                                .spacing([6.0, 3.0])
                                .show(ui, |ui| {
                                    col_head(ui, "函数");
                                    col_head(ui, "自身占比");
                                    col_head(ui, "自身 ms/帧");
                                    col_head(ui, "总计占比");
                                    col_head(ui, "总计 ms/帧");
                                    col_head(ui, "采样");
                                    col_head(ui, "位置");
                                    ui.end_row();

                                    for f in funcs.iter().take(30) {
                                        let self_pct = f.self_count as f64 / total * 100.0;
                                        let total_pct = f.total_count as f64 / total * 100.0;
                                        let self_frame = self_pct / 100.0 * frame_ms;
                                        let total_frame = total_pct / 100.0 * frame_ms;
                                        let is_c = f.id.starts_with("[C]:");

                                        let name_color = if is_c {
                                            egui::Color32::from_rgb(130, 175, 235)
                                        } else {
                                            ui.visuals().text_color()
                                        };

                                        ui.label(
                                            egui::RichText::new(&f.name)
                                                .monospace()
                                                .size(13.0)
                                                .color(name_color),
                                        );

                                        pct_bar(ui, self_pct, egui::Color32::from_rgb(230, 115, 60));
                                        ui.label(
                                            egui::RichText::new(format_ms(self_frame))
                                                .monospace()
                                                .size(13.0)
                                                .color(ms_color(self_frame, frame_ms)),
                                        );

                                        pct_bar(ui, total_pct, egui::Color32::from_rgb(100, 160, 230));
                                        ui.label(
                                            egui::RichText::new(format_ms(total_frame))
                                                .monospace()
                                                .size(13.0)
                                                .color(ms_color(total_frame, frame_ms)),
                                        );

                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}/{}",
                                                format_count(f.self_count),
                                                format_count(f.total_count),
                                            ))
                                            .monospace()
                                            .size(13.0),
                                        );
                                        ui.label(
                                            egui::RichText::new(&f.id)
                                                .monospace()
                                                .size(12.0)
                                                .color(DIM),
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                });
        }

        // ── Flame graph (center) ──
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                if self.profiler_room_id.is_none() {
                    empty_hint(ui, "请选择一个房间");
                    return;
                }
                if !room_ok {
                    empty_hint(ui, "房间不存在或 Profiler 不可用");
                    return;
                }
                if !has_data {
                    empty_hint(ui, "暂无采样数据 — 点击 ▶ 开始 启动采样");
                    return;
                }

                let data = profile_data.as_ref().unwrap();

                ui.horizontal(|ui| {
                    section_heading(ui, "调用火焰图");
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Lua 函数")
                            .size(12.0)
                            .background_color(egui::Color32::from_rgb(225, 130, 60))
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("C / API")
                            .size(12.0)
                            .background_color(egui::Color32::from_rgb(85, 140, 210))
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add_space(4.0);

                draw_flame_graph(
                    ui,
                    data,
                    self.profiler_frame_ms as f64,
                    &mut self.profiler_hover_info,
                );
            });
    }
}

// ── Flame graph ──

fn draw_flame_graph(
    ui: &mut egui::Ui,
    data: &ProfileData,
    frame_ms: f64,
    hover_info: &mut String,
) {
    let root = &data.root;
    if root.count == 0 {
        return;
    }

    let depth = max_depth(root, 0);
    let h = (depth as f32) * (BAR_H + BAR_GAP);

    *hover_info = String::new();

    egui::ScrollArea::both()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let w = ui.available_width().max(600.0);
            let (resp, painter) =
                ui.allocate_painter(egui::vec2(w, h), egui::Sense::hover());

            let origin = resp.rect.min;
            let hover_pos = ui.input(|i| i.pointer.hover_pos());

            draw_node(
                &painter,
                root,
                origin.x,
                origin.y,
                w,
                root.count as f64,
                frame_ms,
                hover_pos,
                hover_info,
                true,
            );

            if !hover_info.is_empty() {
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    egui::LayerId::new(egui::Order::Tooltip, ui.id().with("ft")),
                    ui.id().with("ftt"),
                    |ui: &mut egui::Ui| {
                        ui.set_max_width(340.0);
                        ui.label(
                            egui::RichText::new(hover_info.as_str())
                                .monospace()
                                .size(13.0),
                        );
                    },
                );
            }
        });
}

fn draw_node(
    painter: &egui::Painter,
    node: &ProfileNode,
    x: f32,
    y: f32,
    w: f32,
    root_count: f64,
    frame_ms: f64,
    hover_pos: Option<egui::Pos2>,
    hover_info: &mut String,
    is_root: bool,
) {
    if w < MIN_BAR_W || node.count == 0 {
        return;
    }

    let child_y;

    if is_root {
        child_y = y;
    } else {
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, BAR_H));
        let color = node_color(&node.name, &node.id);
        painter.rect_filled(rect, 1.0, color);
        painter.rect_stroke(
            rect,
            1.0,
            egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(0, 0, 0, 60)),
        );

        if w > 32.0 {
            let avail = ((w - 6.0) / 7.0) as usize;
            let label: String = if node.name.chars().count() > avail && avail > 3 {
                let mut s: String = node.name.chars().take(avail - 2).collect();
                s.push_str("..");
                s
            } else {
                node.name.clone()
            };
            let clip = egui::Rect::from_min_size(egui::pos2(x + 2.0, y), egui::vec2(w - 4.0, BAR_H));
            painter.with_clip_rect(clip).text(
                egui::pos2(x + 4.0, y + BAR_H * 0.5),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::new(12.0, egui::FontFamily::Monospace),
                egui::Color32::WHITE,
            );
        }

        if let Some(pos) = hover_pos {
            if rect.contains(pos) && hover_info.is_empty() {
                let pct = node.count as f64 / root_count * 100.0;
                let self_pct = node.self_count as f64 / root_count * 100.0;
                let total_frame = pct / 100.0 * frame_ms;
                let self_frame = self_pct / 100.0 * frame_ms;
                *hover_info = format!(
                    "{}\n─────────────────────────────\nTotal  {:>6}  {:>5.1}%  {}\nSelf   {:>6}  {:>5.1}%  {}\n─────────────────────────────\n{}",
                    node.name,
                    format_count(node.count), pct, format_ms(total_frame),
                    format_count(node.self_count), self_pct, format_ms(self_frame),
                    node.id,
                );
            }
        }

        child_y = y + BAR_H + BAR_GAP;
    }

    let parent_count = node.count as f64;
    let mut cx = x;
    for child in &node.children {
        let cw = (child.count as f64 / parent_count) * w as f64;
        if cw >= MIN_BAR_W as f64 {
            draw_node(
                painter,
                child,
                cx,
                child_y,
                cw as f32,
                root_count,
                frame_ms,
                hover_pos,
                hover_info,
                false,
            );
            cx += cw as f32;
        }
    }
}

fn max_depth(node: &ProfileNode, d: usize) -> usize {
    let mut m = d;
    for c in &node.children {
        m = m.max(max_depth(c, d + 1));
    }
    m
}

fn node_color(name: &str, id: &str) -> egui::Color32 {
    let h = name
        .bytes()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
    if id.starts_with("[C]:") {
        egui::Color32::from_rgb(
            65 + (h % 35) as u8,
            125 + ((h >> 8) % 45) as u8,
            190 + ((h >> 16) % 45) as u8,
        )
    } else {
        egui::Color32::from_rgb(
            210 + (h % 46) as u8,
            100 + ((h >> 8) % 70) as u8,
            40 + ((h >> 16) % 40) as u8,
        )
    }
}

// ── Helpers ──

fn pct_bar(ui: &mut egui::Ui, pct: f64, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(90.0, 16.0), egui::Sense::hover());
    let fill_w = (pct / 100.0).min(1.0) as f32 * rect.width();
    let fill = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
    ui.painter()
        .rect_filled(fill, 2.0, color.linear_multiply(0.25));
    ui.painter().text(
        rect.left_center() + egui::vec2(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        format!("{:.1}%", pct),
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
        ui.visuals().text_color(),
    );
}

fn format_ms(ms: f64) -> String {
    if ms >= 1.0 {
        format!("{:.1} ms", ms)
    } else if ms >= 0.01 {
        format!("{:.2} ms", ms)
    } else {
        format!("{:.0} μs", ms * 1000.0)
    }
}

fn ms_color(ms: f64, frame_ms: f64) -> egui::Color32 {
    let ratio = (ms / frame_ms).min(1.0);
    if ratio > 0.5 {
        egui::Color32::from_rgb(255, 90, 90)
    } else if ratio > 0.2 {
        egui::Color32::from_rgb(255, 200, 80)
    } else {
        egui::Color32::from_rgb(180, 210, 180)
    }
}

fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn empty_hint(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.3);
        ui.label(egui::RichText::new(text).size(15.0).color(DIM));
    });
}

use std::time::Instant;

use egui::{Context, Frame, TopBottomPanel};

use crate::constants::*;

fn rgb(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

pub struct UiState {
    pub show_panel: bool,
    pub show_grid: bool,
    pub pixelated: bool,
    pub model_path: String,
    pub use_texture: bool,
    pub fps: f32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    frame_count: u32,
    fps_timer: Instant,
}

impl UiState {
    pub fn new(model_path: String, _texture_path: String) -> Self {
        Self {
            show_panel: true,
            show_grid: true,
            pixelated: false,
            model_path,
            use_texture: true,
            fps: 0.0,
            vertex_count: 0,
            triangle_count: 0,
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }

    pub fn update_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.fps_timer.elapsed();
        if elapsed.as_secs_f32() >= 0.5 {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.fps_timer = Instant::now();
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        if !self.show_panel {
            return;
        }

        let panel_color = rgb(UI_PANEL_FILL);

        TopBottomPanel::bottom("status_bar")
            .min_height(52.0)
            .frame(Frame {
                fill: panel_color,
                inner_margin: egui::Margin::symmetric(12.0, 6.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("TMV Alpha")
                            .color(rgb(UI_ACCENT))
                            .size(14.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("v0.36")
                            .color(egui::Color32::from_rgb(140, 140, 150))
                            .size(12.0),
                    );

                    ui.separator();

                    ui.label(
                        egui::RichText::new(format!("Model: {}", self.model_path))
                            .size(12.0),
                    );

                    ui.separator();

                    ui.label(
                        egui::RichText::new(format!("Verts: {}", self.vertex_count))
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(format!("Tris: {}", self.triangle_count))
                            .size(12.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let fps_color = if self.fps >= 55.0 {
                            egui::Color32::from_rgb(76, 175, 80)
                        } else if self.fps >= 30.0 {
                            egui::Color32::from_rgb(255, 193, 7)
                        } else {
                            egui::Color32::from_rgb(244, 67, 54)
                        };
                        ui.label(
                            egui::RichText::new(format!("FPS: {:.0}", self.fps))
                                .color(fps_color)
                                .size(12.0)
                                .monospace(),
                        );
                    });
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_texture, "Texture");
                    ui.checkbox(&mut self.pixelated, "Pixelated");
                    ui.checkbox(&mut self.show_grid, "Grid");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("F1 - UI | F2 - Grid")
                                .color(egui::Color32::from_rgb(100, 100, 110))
                                .size(11.0),
                        );
                    });
                });
            });
    }

    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
}

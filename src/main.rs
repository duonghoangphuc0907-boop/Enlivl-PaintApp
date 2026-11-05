#![windows_subsystem = "windows"]
use eframe::egui::{self, Color32, Pos2, Sense, Stroke};
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([950.0, 650.0])
            .with_title("Enlivl PaintApp"),
        ..Default::default()
    };

    eframe::run_native(
        "Enlivl PaintApp",
        native_options,
        Box::new(|_cc| Box::new(PaintApp::default())),
    )
}

// Serializable structs for saving/loading project files
#[derive(Serialize, Deserialize, Clone)]
struct SerializableLine {
    points: Vec<(f32, f32)>,
    color: (u8, u8, u8, u8),
    width: f32,
}

struct StrokeLine {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

struct PaintApp {
    lines: Vec<StrokeLine>,
    current_color: Color32,
    brush_size: f32,
    is_eraser: bool,
    canvas_color: Color32,
    current_file_path: Option<String>,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current_color: Color32::from_rgb(0, 0, 0),
            brush_size: 4.0,
            is_eraser: false,
            canvas_color: Color32::WHITE,
            current_file_path: None,
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Menu Bar for File Operations
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    // New Canvas
                    if ui.button("New").clicked() {
                        self.lines.clear();
                        self.current_file_path = None;
                        ui.close_menu();
                    }

                    // Open File
                    if ui.button("Open...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Paint Project", &["paint", "json"])
                            .pick_file()
                        {
                            if let Ok(contents) = std::fs::read_to_string(&path) {
                                if let Ok(serializable_lines) =
                                    serde_json::from_str::<Vec<SerializableLine>>(&contents)
                                {
                                    self.lines = serializable_lines
                                        .into_iter()
                                        .map(|sl| StrokeLine {
                                            points: sl
                                                .points
                                                .into_iter()
                                                .map(|(x, y)| Pos2::new(x, y))
                                                .collect(),
                                            color: Color32::from_rgba_unmultiplied(
                                                sl.color.0, sl.color.1, sl.color.2, sl.color.3,
                                            ),
                                            width: sl.width,
                                        })
                                        .collect();
                                    self.current_file_path = Some(path.to_string_lossy().to_string());
                                }
                            }
                        }
                        ui.close_menu();
                    }

                    // Save As
                    if ui.button("Save As...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Paint Project", &["paint", "json"])
                            .save_file()
                        {
                            let serializable_lines: Vec<SerializableLine> = self
                                .lines
                                .iter()
                                .map(|line| SerializableLine {
                                    points: line.points.iter().map(|p| (p.x, p.y)).collect(),
                                    color: line.color.to_tuple(),
                                    width: line.width,
                                })
                                .collect();

                            if let Ok(json) = serde_json::to_string_pretty(&serializable_lines) {
                                let _ = std::fs::write(&path, json);
                                self.current_file_path = Some(path.to_string_lossy().to_string());
                            }
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    // Close App
                    if ui.button("Close").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                if let Some(ref path) = self.current_file_path {
                    ui.label(format!("- Editing: {}", path));
                }
            });
        });

        // Toolbar / Drawing Controls
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 Enlivl PaintApp");
                ui.separator();

                // Color Picker
                let mut color = [
                    self.current_color.r() as f32 / 255.0,
                    self.current_color.g() as f32 / 255.0,
                    self.current_color.b() as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    self.current_color = Color32::from_rgb(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    );
                    self.is_eraser = false;
                }

                // Brush Size Slider
                ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=50.0).text("Brush Size"));

                ui.separator();

                // Eraser Toggle
                ui.selectable_value(&mut self.is_eraser, false, "✏️ Draw");
                ui.selectable_value(&mut self.is_eraser, true, "🧹 Eraser");

                ui.separator();

                // Clear Canvas Button
                if ui.button("🗑️ Clear Canvas").clicked() {
                    self.lines.clear();
                }
            });
        });

        // Main Drawing Canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                Sense::drag(),
            );

            let rect = response.rect;
            painter.rect_filled(rect, 0.0, self.canvas_color);

            // Handle Input / Drawing
            if response.dragged() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let active_color = if self.is_eraser {
                        self.canvas_color
                    } else {
                        self.current_color
                    };

                    if let Some(last_line) = self.lines.last_mut() {
                        if response.drag_started()
                            || last_line.color != active_color
                            || last_line.width != self.brush_size
                        {
                            self.lines.push(StrokeLine {
                                points: vec![pointer_pos],
                                color: active_color,
                                width: self.brush_size,
                            });
                        } else {
                            last_line.points.push(pointer_pos);
                        }
                    } else {
                        self.lines.push(StrokeLine {
                            points: vec![pointer_pos],
                            color: active_color,
                            width: self.brush_size,
                        });
                    }
                }
            }

            // Render all stored strokes
            for line in &self.lines {
                if line.points.len() < 2 {
                    if let Some(&p) = line.points.first() {
                        painter.circle_filled(p, line.width / 2.0, line.color);
                    }
                    continue;
                }

                for window in line.points.windows(2) {
                    painter.line_segment(
                        [window[0], window[1]],
                        Stroke::new(line.width, line.color),
                    );
                }
            }
        });
    }
}

//! A lightweight RAD GUI builder for `egui` written in Rust.

use eframe::{egui, egui::pos2, egui::vec2};
use egui::{Align, Color32, Id, Pos2, Rect, Response, Rounding, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("egui RAD GUI Builder", native_options, Box::new(|_cc| Ok(Box::<RadBuilderApp>::default())),)
}

struct RadBuilderApp {
    palette_open: bool,
    project: Project,
    selected: Option<WidgetId>,
    next_id: u64,
    // Drag state for spawning from palette
    spawning: Option<WidgetKind>,
    // Cached generated code
    generated: String,
    // Settings
    grid_size: f32,
}

impl Default for Project {
    fn default() -> Self {
        Self { widgets: Vec::new(), canvas_size: vec2(1200.0, 800.0) }
    }
}

impl Default for RadBuilderApp {
    fn default() -> Self {
        Self {
            palette_open: true,
            project: Project::default(),
            selected: None,
            next_id: 1,
            spawning: None,
            generated: String::new(),
            grid_size: 8.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct WidgetId(u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Project {
    widgets: Vec<Widget>,
    canvas_size: Vec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Widget {
    id: WidgetId,
    kind: WidgetKind,
    pos: Pos2,   // Top-left relative to canvas
    size: Vec2,  // Desired size on canvas
    z: i32,      // draw order
    props: WidgetProps,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum WidgetKind {
    Label,
    Button,
    Checkbox,
    TextEdit,
    Slider,
    ProgressBar,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WidgetProps {
    text: String,         // label/button/textedit placeholder
    checked: bool,        // checkbox
    value: f32,           // slider/progress
    min: f32,
    max: f32,
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self { text: "Label".into(), checked: false, value: 0.5, min: 0.0, max: 1.0 }
    }
}

impl RadBuilderApp {
    fn spawn_widget(&mut self, kind: WidgetKind, at: Pos2) {
        let id = WidgetId(self.next_id);
        self.next_id += 1;
        let (size, props) = match kind {
            WidgetKind::Label => (vec2(140.0, 24.0), WidgetProps { text: "Label".into(), ..Default::default() }),
            WidgetKind::Button => (vec2(160.0, 32.0), WidgetProps { text: "Button".into(), ..Default::default() }),
            WidgetKind::Checkbox => (vec2(160.0, 28.0), WidgetProps { text: "Checkbox".into(), ..Default::default() }),
            WidgetKind::TextEdit => (vec2(220.0, 36.0), WidgetProps { text: "Type here".into(), ..Default::default() }),
            WidgetKind::Slider => (vec2(220.0, 24.0), WidgetProps { text: "Value".into(), min: 0.0, max: 100.0, value: 42.0, checked: false }),
            WidgetKind::ProgressBar => (vec2(220.0, 20.0), WidgetProps { text: "".into(), value: 0.25, min: 0.0, max: 1.0, checked: false }),
        };
        let w = Widget { id, kind, pos: at, size, z: id.0 as i32, props };
        self.project.widgets.push(w);
        self.selected = Some(id);
    }

    fn selected_mut(&mut self) -> Option<&mut Widget> {
        let id = self.selected?;
        self.project.widgets.iter_mut().find(|w| w.id == id)
    }

    fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        // The design canvas area
        let (canvas_resp, _painter) = ui.allocate_painter(self.project.canvas_size, Sense::click_and_drag());
        let canvas_rect = canvas_resp.rect;

        // Spawn from palette drag-preview
        if let Some(kind) = self.spawning.clone() {
            if let Some(mouse) = ui.ctx().pointer_interact_pos() {
                let ghost = Rect::from_min_size(mouse - vec2(50.0, 16.0), vec2(120.0, 32.0));
                let layer = egui::LayerId::new(egui::Order::Tooltip, Id::new("ghost"));
                let painter = ui.ctx().layer_painter(layer);
                painter.rect_filled(ghost, 4.0, Color32::from_gray(40));
                painter.rect_stroke(ghost, Rounding::same(4), Stroke::new(1.0, Color32::LIGHT_BLUE), egui::StrokeKind::Outside);
                painter.text(ghost.center(), egui::Align2::CENTER_CENTER, format!("{:?}", kind), egui::FontId::proportional(14.0), Color32::LIGHT_BLUE);
            }
            // Drop on mouse release inside canvas
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    if canvas_rect.contains(pos) {
                        let local = pos - canvas_rect.min;            // Vec2
                        let snapped = self.snap_pos(pos2(local.x, local.y));
                        self.spawn_widget(kind, snapped);
                    }
                }
                self.spawning = None;
            }
        }

        // Background grid
        self.draw_grid(ui, canvas_rect);

        // Draw all widgets, top-to-bottom by z
        self.project.widgets.sort_by_key(|w| w.z);
        self.project.widgets.sort_by_key(|w| w.z);
        for w in &mut self.project.widgets {
            Self::draw_widget(ui, canvas_rect, self.grid_size, &mut self.selected, w);
        }

        // Click empty space to clear selection
        if canvas_resp.clicked() {
            self.selected = None;
        }
    }

    fn draw_grid(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        let g = self.grid_size.max(4.0);
        let cols = (rect.width() / g) as i32;
        let rows = (rect.height() / g) as i32;
        for c in 0..=cols {
            let x = rect.left() + c as f32 * g;
            painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(1.0, Color32::from_gray(40)));
        }
        for r in 0..=rows {
            let y = rect.top() + r as f32 * g;
            painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(1.0, Color32::from_gray(40)));
        }
    }

    fn draw_widget(
        ui: &mut egui::Ui,
        canvas_rect: Rect,
        grid: f32,
        selected: &mut Option<WidgetId>,
        w: &mut Widget,
    ) {
        let rect = Rect::from_min_size(canvas_rect.min + w.pos.to_vec2(), w.size);

        let move_resp = ui.allocate_rect(rect, Sense::click_and_drag());
        if move_resp.clicked() {
            *selected = Some(w.id);
        }
        if move_resp.dragged() {
            let delta = move_resp.drag_delta();
            w.pos += delta;
            w.pos = snap_pos_with_grid(w.pos, grid);
        }

        let handle_size = 10.0;
        let handle_rect = Rect::from_min_size(rect.max - vec2(handle_size, handle_size), vec2(handle_size, handle_size));
        let handle_resp = ui.allocate_rect(handle_rect, Sense::click_and_drag());
        if handle_resp.dragged() {
            let delta = handle_resp.drag_delta();
            w.size += delta;
            w.size.x = w.size.x.max(20.0);
            w.size.y = w.size.y.max(16.0);
        }

        let painter = ui.painter();
        if *selected == Some(w.id) {
            painter.rect_stroke(
                rect,
                Rounding::same(6),
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                egui::StrokeKind::Outside,
            );
        } else {
            painter.rect_stroke(
                rect,
                Rounding::same(6),
                Stroke::new(1.0, Color32::from_gray(90)),
                egui::StrokeKind::Outside,
            );
        }
        painter.rect_filled(handle_rect, 2.0, Color32::from_rgb(100, 160, 255));

        ui.allocate_ui_at_rect(rect, |ui| {
            match w.kind {
                WidgetKind::Label => {
                    ui.vertical_centered(|ui| { ui.label(&w.props.text); });
                }
                WidgetKind::Button => {
                    ui.add_sized(w.size, egui::Button::new(&w.props.text));
                }
                WidgetKind::Checkbox => {
                    let mut checked = w.props.checked;
                    ui.add_sized(w.size, egui::Checkbox::new(&mut checked, &w.props.text));
                    w.props.checked = checked;
                }
                WidgetKind::TextEdit => {
                    let mut buf = w.props.text.clone();
                    let resp = egui::TextEdit::singleline(&mut buf).hint_text("text");
                    ui.add_sized(w.size, resp);
                    w.props.text = buf;
                }
                WidgetKind::Slider => {
                    let mut v = w.props.value;
                    let slider = egui::Slider::new(&mut v, w.props.min..=w.props.max).text(&w.props.text);
                    ui.add_sized(w.size, slider);
                    w.props.value = v;
                }
                WidgetKind::ProgressBar => {
                    let bar = egui::ProgressBar::new(w.props.value.clamp(0.0, 1.0)).show_percentage();
                    ui.add_sized(w.size, bar);
                }
            }
        });
    }

    fn snap_pos(&self, p: Pos2) -> Pos2 { pos2((p.x / self.grid_size).round() * self.grid_size, (p.y / self.grid_size).round() * self.grid_size) }

    fn palette_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Palette");
        ui.separator();
        ui.label("Drag any control onto the canvas");
        ui.add_space(8.0);

        self.palette_item(ui, "Label", WidgetKind::Label);
        self.palette_item(ui, "Button", WidgetKind::Button);
        self.palette_item(ui, "Checkbox", WidgetKind::Checkbox);
        self.palette_item(ui, "TextEdit", WidgetKind::TextEdit);
        self.palette_item(ui, "Slider", WidgetKind::Slider);
        self.palette_item(ui, "ProgressBar", WidgetKind::ProgressBar);

        ui.separator();
        ui.label("Tips:");
        ui.small("• Click a control to select it\n• Drag to move, drag the corner to resize\n• Snap-to-grid can be changed in Settings");
    }

    fn palette_item(&mut self, ui: &mut egui::Ui, label: &str, kind: WidgetKind) {
        let r = ui.add(egui::Button::new(label).sense(Sense::drag()));
        if r.drag_started() || r.clicked() {
            self.spawning = Some(kind);
        }
    }

    fn inspector_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        ui.separator();
        if let Some(w) = self.selected_mut() {
            ui.label(format!("ID: {:?}", w.id.0));
            ui.add_space(6.0);
            match w.kind {
                WidgetKind::Label | WidgetKind::Button | WidgetKind::TextEdit | WidgetKind::Checkbox | WidgetKind::Slider => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
                WidgetKind::ProgressBar => { /* no text */ }
            }
            match w.kind {
                WidgetKind::Checkbox => {
                    ui.checkbox(&mut w.props.checked, "checked");
                }
                WidgetKind::Slider => {
                    ui.add(egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max).text("value"));
                    ui.add(egui::Slider::new(&mut w.props.min, -1000.0..=w.props.max).text("min"));
                    ui.add(egui::Slider::new(&mut w.props.max, w.props.min..=1000.0).text("max"));
                }
                WidgetKind::ProgressBar => {
                    ui.add(egui::Slider::new(&mut w.props.value, 0.0..=1.0).text("progress"));
                }
                _ => {}
            }
            ui.separator();
            ui.label("Position / Size");
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::DragValue::new(&mut w.pos.x));
                ui.label("y");
                ui.add(egui::DragValue::new(&mut w.pos.y));
            });
            ui.horizontal(|ui| {
                ui.label("w");
                ui.add(egui::DragValue::new(&mut w.size.x).clamp_range(16.0..=2000.0));
                ui.label("h");
                ui.add(egui::DragValue::new(&mut w.size.y).clamp_range(12.0..=2000.0));
            });

            ui.add_space(6.0);
            if ui.button("Delete").clicked() {
                let id = w.id; // capture
                self.project.widgets.retain(|w| w.id != id);
                self.selected = None;
            }
        } else {
            ui.weak("No selection");
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Generate Code").clicked() {
                    self.generated = self.generate_code();
                    ui.close_menu();
                }
                if ui.button("Export JSON").clicked() {
                    if let Ok(s) = serde_json::to_string_pretty(&self.project) {
                        self.generated = s;
                    }
                    ui.close_menu();
                }
                if ui.button("Import JSON (from editor below)").clicked() {
                    if let Ok(p) = serde_json::from_str::<Project>(&self.generated) {
                        self.project = p;
                        self.selected = None;
                    }
                    ui.close_menu();
                }
                if ui.button("Clear Project").clicked() {
                    self.project = Project::default();
                    self.selected = None;
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.palette_open, "Show Palette");
            });

            ui.menu_button("Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Grid");
                    ui.add(egui::DragValue::new(&mut self.grid_size).clamp_range(2.0..=64.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Canvas size");
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.x));
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.y));
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Generate Code").clicked() { self.generated = self.generate_code(); }
                ui.separator();
                ui.strong("egui RAD GUI Builder");
            });
        });
    }

    fn generated_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading("Generated Output");
		ui.label("Rust code (or JSON export) will appear here. Copy-paste into your app.");

		// A scrollable viewport for the generated text:
		egui::ScrollArea::vertical()
			.id_source("generated_output_scroll")
			.max_height(280.0) // tweak to taste
			.auto_shrink([false, false])
			.show(ui, |ui| {
				let editor = egui::TextEdit::multiline(&mut self.generated)
					.code_editor()
					.lock_focus(true)
					.desired_rows(18)
					.desired_width(f32::INFINITY); // fill available width

				ui.add(editor);
			});
	}

    fn generate_code(&self) -> String {
		let mut out = String::new();
		out.push_str("// --- generated by egui RAD GUI Builder ---\n");
		out.push_str("use eframe::egui;\n\n");

		// --- State struct ---
		out.push_str("struct GeneratedState {\n");
		for w in &self.project.widgets {
			match w.kind {
				WidgetKind::TextEdit    => out.push_str(&format!("    text_{}: String,\n",    w.id.0)),
				WidgetKind::Checkbox    => out.push_str(&format!("    checked_{}: bool,\n",   w.id.0)),
				WidgetKind::Slider      => out.push_str(&format!("    value_{}: f32,\n",      w.id.0)),
				WidgetKind::ProgressBar => out.push_str(&format!("    progress_{}: f32,\n",   w.id.0)),
				_ => {}
			}
		}
		out.push_str("}\n\n");

		// --- Default impl with designed initial values ---
		out.push_str("impl Default for GeneratedState {\n");
		out.push_str("    fn default() -> Self {\n");
		out.push_str("        Self {\n");
		for w in &self.project.widgets {
			match w.kind {
				WidgetKind::TextEdit => {
					out.push_str(&format!("            text_{}: \"{}\".to_owned(),\n", w.id.0, escape(&w.props.text)));
				}
				WidgetKind::Checkbox => {
					out.push_str(&format!("            checked_{}: {},\n", w.id.0, if w.props.checked { "true" } else { "false" }));
				}
				WidgetKind::Slider => {
					out.push_str(&format!("            value_{}: {:.3},\n", w.id.0, w.props.value));
				}
				WidgetKind::ProgressBar => {
					// clamp to [0,1] to be safe
					let p = w.props.value.clamp(0.0, 1.0);
					out.push_str(&format!("            progress_{}: {:.3},\n", w.id.0, p));
				}
				_ => {}
			}
		}
		out.push_str("        }\n");
		out.push_str("    }\n");
		out.push_str("}\n\n");

		// --- UI function ---
		out.push_str("fn generated_ui(ui: &mut egui::Ui, state: &mut GeneratedState) {\n");
		out.push_str(&format!(
			"    let canvas = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2({:.1}, {:.1}));\n",
			self.project.canvas_size.x, self.project.canvas_size.y
		));
		out.push_str("    let (_resp, _p) = ui.allocate_painter(canvas.size(), egui::Sense::hover());\n\n");

		for w in &self.project.widgets {
			let pos = w.pos;
			let size = w.size;

			match w.kind {
				WidgetKind::Label => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.label(\"{}\"); }});\n",
						pos.x, pos.y, size.x, size.y, escape(&w.props.text)
					));
				}
				WidgetKind::Button => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::Button::new(\"{}\")); }});\n",
						pos.x, pos.y, size.x, size.y, size.x, size.y, escape(&w.props.text)
					));
				}
				WidgetKind::Checkbox => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.checkbox(&mut state.checked_{}, \"{}\"); }});\n",
						pos.x, pos.y, size.x, size.y, w.id.0, escape(&w.props.text)
					));
				}
				WidgetKind::TextEdit => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::TextEdit::singleline(&mut state.text_{}).hint_text(\"{}\")); }});\n",
						pos.x, pos.y, size.x, size.y, size.x, size.y, w.id.0, escape(&w.props.text)
					));
				}
				WidgetKind::Slider => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::Slider::new(&mut state.value_{}, {:.3}..={:.3}).text(\"{}\")); }});\n",
						pos.x, pos.y, size.x, size.y, size.x, size.y, w.id.0, w.props.min, w.props.max, escape(&w.props.text)
					));
				}
				WidgetKind::ProgressBar => {
					out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::ProgressBar::new(state.progress_{}).show_percentage()); }});\n",
						pos.x, pos.y, size.x, size.y, size.x, size.y, w.id.0
					));
				}
			}
		}

		out.push_str("}\n\n");

		// --- Minimal eframe host app + main ---
		out.push_str("// Example eframe app to host the generated UI\n");
		out.push_str("pub struct GeneratedApp { state: GeneratedState }\n");
		out.push_str("impl Default for GeneratedApp { fn default() -> Self { Self { state: Default::default() } } }\n");
		out.push_str("impl eframe::App for GeneratedApp {\n");
		out.push_str("    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n");
		out.push_str("        egui::CentralPanel::default().show(ctx, |ui| {\n");
		out.push_str("            generated_ui(ui, &mut self.state);\n");
		out.push_str("        });\n");
		out.push_str("    }\n");
		out.push_str("}\n\n");

		out.push_str("fn main() -> eframe::Result<()> {\n");
		out.push_str("    let native_options = eframe::NativeOptions::default();\n");
		out.push_str("    eframe::run_native(\n");
		out.push_str("        \"Generated UI\",\n");
		out.push_str("        native_options,\n");
		out.push_str("        Box::new(|_cc| Ok(Box::new(GeneratedApp::default()))),\n");
		out.push_str("    )\n");
		out.push_str("}\n");

		out
	}

}

fn snap_pos_with_grid(p: Pos2, grid: f32) -> Pos2 {
    pos2((p.x / grid).round() * grid, (p.y / grid).round() * grid)
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl eframe::App for RadBuilderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| self.top_bar(ui));

        if self.palette_open {
            egui::SidePanel::left("palette").resizable(true).show(ctx, |ui| {
                self.palette_ui(ui);
            });
        }

        egui::SidePanel::right("inspector").default_width(260.0).show(ctx, |ui| {
            self.inspector_ui(ui);
            ui.separator();
            self.generated_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
            self.canvas_ui(ui);
        });

        // Cursor hint when spawning
        if self.spawning.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }
    }
}
















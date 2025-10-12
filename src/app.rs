use chrono::{Datelike, NaiveDate};
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Sense, Stroke, UiBuilder, pos2, vec2};
use egui_extras::DatePickerButton;

use crate::{
    project::Project,
    widget::{self, Widget, WidgetId, WidgetKind, WidgetProps, escape, snap_pos_with_grid},
};

pub(crate) struct RadBuilderApp {
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

impl RadBuilderApp {
    fn spawn_widget(&mut self, kind: WidgetKind, at: Pos2) {
        let id = WidgetId::new(self.next_id);
        self.next_id += 1;

        let (size, props) = match kind {
            WidgetKind::Label => (
                vec2(140.0, 24.0),
                WidgetProps {
                    text: "Label".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Button => (
                vec2(160.0, 32.0),
                WidgetProps {
                    text: "Button".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::ImageTextButton => (
                vec2(200.0, 36.0),
                WidgetProps {
                    text: "Button".into(),
                    icon: "🖼️".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Checkbox => (
                vec2(160.0, 28.0),
                WidgetProps {
                    text: "Checkbox".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::TextEdit => (
                vec2(220.0, 36.0),
                WidgetProps {
                    text: "Type here".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Slider => (
                vec2(220.0, 24.0),
                WidgetProps {
                    text: "Value".into(),
                    min: 0.0,
                    max: 100.0,
                    value: 42.0,
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::ProgressBar => (
                vec2(220.0, 20.0),
                WidgetProps {
                    text: "".into(),
                    value: 0.25,
                    min: 0.0,
                    max: 1.0,
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::RadioGroup => {
                let mut p = WidgetProps {
                    text: "Radio Group".into(),
                    ..Default::default()
                };
                p.items = vec!["Option A".into(), "Option B".into(), "Option C".into()];
                p.selected = 0;
                (vec2(200.0, 80.0), p)
            }
            WidgetKind::Link => (
                vec2(160.0, 20.0),
                WidgetProps {
                    text: "Link text".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Hyperlink => (
                vec2(200.0, 20.0),
                WidgetProps {
                    text: "Open website".into(),
                    url: "https://example.com".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::SelectableLabel => (
                vec2(180.0, 24.0),
                WidgetProps {
                    text: "Selectable".into(),
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::ComboBox => {
                let mut p = WidgetProps {
                    text: "Choose one".into(),
                    ..Default::default()
                };
                p.items = vec!["Red".into(), "Green".into(), "Blue".into()];
                p.selected = 0;
                (vec2(220.0, 28.0), p)
            }
            WidgetKind::Separator => (vec2(220.0, 8.0), WidgetProps::default()),
            WidgetKind::CollapsingHeader => (
                vec2(260.0, 80.0),
                WidgetProps {
                    text: "Section".into(),
                    checked: true, // default open
                    ..Default::default()
                },
            ),
            WidgetKind::DatePicker => (
                vec2(200.0, 28.0),
                WidgetProps {
                    text: "Pick a date".into(),
                    year: 2025,
                    month: 1,
                    day: 1,
                    ..Default::default()
                },
            ),
            WidgetKind::AngleSelector => (
                vec2(220.0, 28.0),
                WidgetProps {
                    text: "Angle (deg)".into(),
                    min: 0.0,
                    max: 360.0,
                    value: 45.0,
                    ..Default::default()
                },
            ),
            WidgetKind::Password => (
                vec2(220.0, 36.0),
                WidgetProps {
                    text: "password".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Tree => {
                let mut p = WidgetProps {
                    text: "Tree".into(),
                    ..Default::default()
                };
                // Indentation (two spaces = one level) to define hierarchy:
                p.items = vec![
                    "Animals".into(),
                    "  Mammals".into(),
                    "    Dogs".into(),
                    "    Cats".into(),
                    "  Birds".into(),
                    "Plants".into(),
                    "  Trees".into(),
                    "  Flowers".into(),
                ];
                (vec2(260.0, 200.0), p)
            }
        };

        let mut pos = at - size * 0.5;
        pos = self.snap_pos(pos);

        let w = Widget {
            id,
            kind,
            pos,
            size,
            z: id.as_z(),
            props,
        };
        self.project.widgets.push(w);
        self.selected = Some(id);
    }

    fn selected_mut(&mut self) -> Option<&mut Widget> {
        let id = self.selected?;
        self.project.widgets.iter_mut().find(|w| w.id == id)
    }

    fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        // The design canvas area
        let (canvas_resp, _painter) =
            ui.allocate_painter(self.project.canvas_size, Sense::click_and_drag());
        let canvas_rect = canvas_resp.rect;

        // Spawn from palette drag-preview
        if let Some(kind) = self.spawning.clone() {
            if let Some(mouse) = ui.ctx().pointer_interact_pos() {
                let ghost_size = match kind {
                    WidgetKind::Label => vec2(140.0, 24.0),
                    WidgetKind::Button => vec2(160.0, 32.0),
                    WidgetKind::ImageTextButton => vec2(200.0, 36.0),
                    WidgetKind::Checkbox => vec2(160.0, 28.0),
                    WidgetKind::TextEdit => vec2(220.0, 36.0),
                    WidgetKind::Slider => vec2(220.0, 24.0),
                    WidgetKind::ProgressBar => vec2(220.0, 20.0),
                    WidgetKind::RadioGroup => vec2(200.0, 80.0),
                    WidgetKind::Link => vec2(160.0, 20.0),
                    WidgetKind::Hyperlink => vec2(200.0, 20.0),
                    WidgetKind::SelectableLabel => vec2(180.0, 24.0),
                    WidgetKind::ComboBox => vec2(220.0, 28.0),
                    WidgetKind::Separator => vec2(220.0, 8.0),
                    WidgetKind::CollapsingHeader => vec2(260.0, 80.0),
                    WidgetKind::DatePicker => vec2(200.0, 28.0),
                    WidgetKind::AngleSelector => vec2(220.0, 28.0),
                    WidgetKind::Password => vec2(220.0, 36.0),
                    WidgetKind::Tree => vec2(260.0, 200.0),
                };
                let ghost = Rect::from_center_size(mouse, ghost_size);
                let layer = egui::LayerId::new(egui::Order::Tooltip, Id::new("ghost"));
                let painter = ui.ctx().layer_painter(layer);
                painter.rect_filled(ghost, 4.0, Color32::from_gray(40));
                painter.rect_stroke(
                    ghost,
                    CornerRadius::same(4),
                    Stroke::new(1.0, Color32::LIGHT_BLUE),
                    egui::StrokeKind::Outside,
                );
                painter.text(
                    ghost.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{:?}", kind),
                    egui::FontId::proportional(14.0),
                    Color32::LIGHT_BLUE,
                );
            }
            // Drop on mouse release inside canvas
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(pos) = ui.ctx().pointer_interact_pos()
                    && canvas_rect.contains(pos)
                {
                    let local = pos - canvas_rect.min; // Vec2
                    self.spawn_widget(kind, pos2(local.x, local.y));
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
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(40)),
            );
        }
        for r in 0..=rows {
            let y = rect.top() + r as f32 * g;
            painter.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(40)),
            );
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
        let handle_rect = Rect::from_min_size(
            rect.max - vec2(handle_size, handle_size),
            vec2(handle_size, handle_size),
        );
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
                CornerRadius::same(6),
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                egui::StrokeKind::Outside,
            );
        } else {
            painter.rect_stroke(
                rect,
                CornerRadius::same(6),
                Stroke::new(1.0, Color32::from_gray(90)),
                egui::StrokeKind::Outside,
            );
        }
        painter.rect_filled(handle_rect, 2.0, Color32::from_rgb(100, 160, 255));

        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| match w.kind {
            WidgetKind::Label => {
                ui.vertical_centered(|ui| {
                    ui.label(&w.props.text);
                });
            }
            WidgetKind::Button => {
                ui.add_sized(w.size, egui::Button::new(&w.props.text));
            }
            WidgetKind::ImageTextButton => {
                // We keep it simple: icon + text as the button label.
                // Users can change `icon` to any emoji / short string.
                let label = format!("{}  {}", w.props.icon, w.props.text);
                ui.add_sized(w.size, egui::Button::new(label));
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
                let slider =
                    egui::Slider::new(&mut v, w.props.min..=w.props.max).text(&w.props.text);
                ui.add_sized(w.size, slider);
                w.props.value = v;
            }
            WidgetKind::ProgressBar => {
                let bar = egui::ProgressBar::new(w.props.value.clamp(0.0, 1.0)).show_percentage();
                ui.add_sized(w.size, bar);
            }
            // NEW drawings:
            WidgetKind::RadioGroup => {
                let mut sel = w.props.selected.min(w.props.items.len().saturating_sub(1));
                ui.vertical(|ui| {
                    for (i, it) in w.props.items.iter().enumerate() {
                        if ui.add(egui::RadioButton::new(sel == i, it)).clicked() {
                            sel = i;
                        }
                    }
                });
                w.props.selected = sel;
            }
            WidgetKind::Link => {
                let _ = ui.link(&w.props.text);
            }
            WidgetKind::Hyperlink => {
                ui.hyperlink_to(&w.props.text, &w.props.url);
            }
            WidgetKind::SelectableLabel => {
                let mut on = w.props.checked;
                if ui
                    .add(egui::Button::selectable(on, &w.props.text))
                    .clicked()
                {
                    on = !on;
                }
                w.props.checked = on;
            }
            WidgetKind::ComboBox => {
                let items = if w.props.items.is_empty() {
                    vec!["Item".into()]
                } else {
                    w.props.items.clone()
                };
                let mut sel = w.props.selected.min(items.len() - 1);
                egui::ComboBox::from_id_salt(w.id)
                    .width(w.size.x)
                    .selected_text(items[sel].clone())
                    .show_ui(ui, |ui| {
                        for (i, it) in items.iter().enumerate() {
                            ui.selectable_value(&mut sel, i, it.clone());
                        }
                    });
                w.props.selected = sel;
            }
            WidgetKind::Separator => {
                ui.separator();
            }
            WidgetKind::CollapsingHeader => {
                egui::CollapsingHeader::new(&w.props.text)
                    .default_open(w.props.checked)
                    .show(ui, |ui| {
                        ui.label("… place your inner content here …");
                    });
            }
            WidgetKind::DatePicker => {
                let mut date = NaiveDate::from_ymd_opt(
                    w.props.year,
                    w.props.month.clamp(1, 12),
                    w.props.day.clamp(1, 28), // simple clamp
                )
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
                ui.horizontal(|ui| {
                    ui.label(&w.props.text);
                    ui.add(DatePickerButton::new(&mut date));
                });
                w.props.year = date.year();
                w.props.month = date.month();
                w.props.day = date.day();
            }
            WidgetKind::AngleSelector => {
                // Angle editor as slider in degrees
                let mut v = w.props.value.clamp(w.props.min, w.props.max);
                let slider = egui::Slider::new(&mut v, w.props.min..=w.props.max)
                    .suffix("°")
                    .text(&w.props.text);
                ui.add_sized(w.size, slider);
                w.props.value = v;
            }
            WidgetKind::Password => {
                let mut buf = w.props.text.clone();
                let resp = egui::TextEdit::singleline(&mut buf)
                    .password(true)
                    .hint_text("password");
                ui.add_sized(w.size, resp);
                w.props.text = buf;
            }
            WidgetKind::Tree => {
                // Parse items (two leading spaces per level) into nodes:
                #[derive(Clone)]
                struct Node {
                    label: String,
                    children: Vec<Node>,
                }

                fn parse_nodes(lines: &[String]) -> Vec<Node> {
                    // (indent, label)
                    let mut items: Vec<(usize, String)> = lines
                        .iter()
                        .map(|s| {
                            let indent = s.chars().take_while(|c| *c == ' ').count() / 2;
                            (indent, s.trim().to_string())
                        })
                        .collect();
                    // Remove empties
                    items.retain(|(_, s)| !s.is_empty());

                    fn build<I: Iterator<Item = (usize, String)>>(
                        iter: &mut std::iter::Peekable<I>,
                        level: usize,
                    ) -> Vec<Node> {
                        let mut out = Vec::new();
                        while let Some((ind, _)) = iter.peek().cloned() {
                            if ind < level {
                                break;
                            }
                            if ind > level {
                                // child of previous; let outer loop handle
                                break;
                            }
                            // ind == level
                            let (_, label) = iter.next().unwrap();
                            // gather children (ind + 1)
                            let children = build(iter, level + 1);
                            out.push(Node { label, children });
                        }
                        out
                    }

                    let mut it = items.into_iter().peekable();
                    build(&mut it, 0)
                }

                fn show_nodes(ui: &mut egui::Ui, nodes: &[Node]) {
                    for n in nodes {
                        if n.children.is_empty() {
                            ui.label(&n.label);
                        } else {
                            ui.collapsing(&n.label, |ui| {
                                show_nodes(ui, &n.children);
                            });
                        }
                    }
                }

                let lines = if w.props.items.is_empty() {
                    vec!["Root".into(), "  Child".into()]
                } else {
                    w.props.items.clone()
                };
                let nodes = parse_nodes(&lines);

                // Constrain content to the widget rect:
                egui::Frame::NONE.show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            show_nodes(ui, &nodes);
                        });
                });
            }
        });
    }

    fn snap_pos(&self, p: Pos2) -> Pos2 {
        pos2(
            (p.x / self.grid_size).round() * self.grid_size,
            (p.y / self.grid_size).round() * self.grid_size,
        )
    }

    fn palette_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Palette");
        ui.separator();
        ui.label("Drag any control onto the canvas");
        ui.add_space(8.0);

        self.palette_item(ui, "Label", WidgetKind::Label);
        self.palette_item(ui, "Button", WidgetKind::Button);
        self.palette_item(ui, "Image + Text Button", WidgetKind::ImageTextButton);
        self.palette_item(ui, "Checkbox", WidgetKind::Checkbox);
        self.palette_item(ui, "TextEdit", WidgetKind::TextEdit);
        self.palette_item(ui, "Slider", WidgetKind::Slider);
        self.palette_item(ui, "ProgressBar", WidgetKind::ProgressBar);
        self.palette_item(ui, "Radio Group", WidgetKind::RadioGroup);
        self.palette_item(ui, "Link", WidgetKind::Link);
        self.palette_item(ui, "Hyperlink", WidgetKind::Hyperlink);
        self.palette_item(ui, "Selectable Label", WidgetKind::SelectableLabel);
        self.palette_item(ui, "Combo Box", WidgetKind::ComboBox);
        self.palette_item(ui, "Separator", WidgetKind::Separator);
        self.palette_item(ui, "Collapsing Header", WidgetKind::CollapsingHeader);
        self.palette_item(ui, "Date Picker", WidgetKind::DatePicker);
        self.palette_item(ui, "Angle Selector", WidgetKind::AngleSelector);
        self.palette_item(ui, "Password", WidgetKind::Password);
        self.palette_item(ui, "Tree", WidgetKind::Tree);

        ui.separator();
        ui.label("Tips:");
        ui.small(
            "• Click a control to select it\n• Drag to move, drag the corner to resize\n• Snap-to-grid can be changed in Settings",
        );
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
            ui.label(format!("ID: {:?}", w.id));
            ui.add_space(6.0);
            match w.kind {
                WidgetKind::Label
                | WidgetKind::Button
                | WidgetKind::ImageTextButton
                | WidgetKind::TextEdit
                | WidgetKind::Checkbox
                | WidgetKind::Slider
                | WidgetKind::Link
                | WidgetKind::Hyperlink
                | WidgetKind::SelectableLabel
                | WidgetKind::CollapsingHeader
                | WidgetKind::Password
                | WidgetKind::AngleSelector
                | WidgetKind::DatePicker => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
                WidgetKind::ProgressBar
                | WidgetKind::RadioGroup
                | WidgetKind::ComboBox
                | WidgetKind::Tree
                | WidgetKind::Separator => {}
            }
            match w.kind {
                WidgetKind::ImageTextButton => {
                    ui.label("Icon / Emoji");
                    ui.text_edit_singleline(&mut w.props.icon);
                }
                WidgetKind::Checkbox => {
                    ui.checkbox(&mut w.props.checked, "checked");
                }
                WidgetKind::Slider => {
                    ui.add(
                        egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max)
                            .text("value"),
                    );
                    ui.add(egui::Slider::new(&mut w.props.min, -1000.0..=w.props.max).text("min"));
                    ui.add(egui::Slider::new(&mut w.props.max, w.props.min..=1000.0).text("max"));
                }
                WidgetKind::ProgressBar => {
                    ui.add(egui::Slider::new(&mut w.props.value, 0.0..=1.0).text("progress"));
                }
                WidgetKind::Hyperlink => {
                    ui.label("URL");
                    ui.text_edit_singleline(&mut w.props.url);
                }
                WidgetKind::RadioGroup | WidgetKind::ComboBox | WidgetKind::Tree => {
                    ui.label(match w.kind {
                        WidgetKind::Tree => "Nodes (indent with spaces; 2 spaces per level)",
                        _ => "Items (one per line)",
                    });
                    let mut buf = w.props.items.join("\n");
                    if ui
                        .add(
                            egui::TextEdit::multiline(&mut buf)
                                .desired_rows(8)
                                .desired_width(f32::INFINITY),
                        )
                        .changed()
                    {
                        w.props.items = buf.lines().map(|s| s.to_string()).collect();
                        if w.props.selected >= w.props.items.len() {
                            w.props.selected = w.props.items.len().saturating_sub(1);
                        }
                    }
                    if !matches!(w.kind, WidgetKind::Tree) && !w.props.items.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Selected index");
                            ui.add(
                                egui::DragValue::new(&mut w.props.selected)
                                    .range(0..=w.props.items.len().saturating_sub(1)),
                            );
                        });
                    }
                }
                WidgetKind::CollapsingHeader => {
                    ui.checkbox(&mut w.props.checked, "open by default");
                }
                WidgetKind::DatePicker => {
                    ui.horizontal(|ui| {
                        ui.label("Year");
                        ui.add(egui::DragValue::new(&mut w.props.year));
                        ui.label("Month");
                        ui.add(egui::DragValue::new(&mut w.props.month).range(1..=12));
                        ui.label("Day");
                        ui.add(egui::DragValue::new(&mut w.props.day).range(1..=31));
                    });
                }
                WidgetKind::AngleSelector => {
                    ui.add(
                        egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max)
                            .text("value (deg)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut w.props.min, -1080.0..=w.props.max)
                            .text("min (deg)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut w.props.max, w.props.min..=1080.0).text("max (deg)"),
                    );
                }
                WidgetKind::Password => { /* no extra props */ }
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
                ui.add(egui::DragValue::new(&mut w.size.x).range(16.0..=2000.0));
                ui.label("h");
                ui.add(egui::DragValue::new(&mut w.size.y).range(12.0..=2000.0));
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
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Generate Code").clicked() {
                    self.generated = self.generate_code();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Export JSON").clicked() {
                    if let Ok(s) = serde_json::to_string_pretty(&self.project) {
                        self.generated = s;
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Import JSON (from editor below)").clicked() {
                    if let Ok(p) = serde_json::from_str::<Project>(&self.generated) {
                        self.project = p;
                        self.selected = None;
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Clear Project").clicked() {
                    self.project = Project::default();
                    self.selected = None;
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.palette_open, "Show Palette");
            });

            ui.menu_button("Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Grid");
                    ui.add(egui::DragValue::new(&mut self.grid_size).range(2.0..=64.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Canvas size");
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.x));
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.y));
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Generate Code").clicked() {
                    self.generated = self.generate_code();
                }
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
            .id_salt("generated_output_scroll")
            .max_height(280.0)
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
        out.push_str("use eframe::egui;\n");
        out.push_str("use egui_extras::DatePickerButton;\n");
        out.push_str("use chrono::NaiveDate;\n\n");

        let has_tree = self
            .project
            .widgets
            .iter()
            .any(|w| matches!(w.kind, WidgetKind::Tree));
        if has_tree {
            out.push_str(
                "#[derive(Clone)]\n\
				 struct GenTreeNode { label: String, children: Vec<GenTreeNode> }\n\
				 \n\
				 fn gen_show_tree(ui: &mut egui::Ui, nodes: &[GenTreeNode]) {\n\
				 \tfor n in nodes {\n\
				 \t\tif n.children.is_empty() { ui.label(&n.label); }\n\
				 \t\telse { ui.collapsing(&n.label, |ui| gen_show_tree(ui, &n.children)); }\n\
				 \t}\n\
				 }\n\n",
            );
        }

        out.push_str("struct GeneratedState {\n");
        for w in &self.project.widgets {
            match w.kind {
                WidgetKind::TextEdit => out.push_str(&format!("    text_{}: String,\n", w.id)),
                WidgetKind::Checkbox => out.push_str(&format!("    checked_{}: bool,\n", w.id)),
                WidgetKind::Slider => out.push_str(&format!("    value_{}: f32,\n", w.id)),
                WidgetKind::ProgressBar => out.push_str(&format!("    progress_{}: f32,\n", w.id)),
                WidgetKind::SelectableLabel => out.push_str(&format!("    sel_{}: bool,\n", w.id)),
                WidgetKind::RadioGroup | WidgetKind::ComboBox => {
                    out.push_str(&format!("    sel_{}: usize,\n", w.id))
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!("    open_{}: bool,\n", w.id))
                }
                WidgetKind::DatePicker => out.push_str(&format!("    date_{}: NaiveDate,\n", w.id)),
                WidgetKind::Password => out.push_str(&format!("    pass_{}: String,\n", w.id)),
                WidgetKind::AngleSelector => out.push_str(&format!("    angle_{}: f32,\n", w.id)),
                _ => {}
            }
        }
        out.push_str("}\n\n");

        out.push_str("impl Default for GeneratedState {\n");
        out.push_str("    fn default() -> Self {\n");
        out.push_str("        Self {\n");
        for w in &self.project.widgets {
            match w.kind {
                WidgetKind::TextEdit => {
                    out.push_str(&format!(
                        "            text_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                WidgetKind::Checkbox => {
                    out.push_str(&format!(
                        "            checked_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::Slider => {
                    out.push_str(&format!(
                        "            value_{}: {:.3},\n",
                        w.id, w.props.value
                    ));
                }
                WidgetKind::ProgressBar => {
                    let p = w.props.value.clamp(0.0, 1.0);
                    out.push_str(&format!("            progress_{}: {:.3},\n", w.id, p));
                }
                WidgetKind::SelectableLabel => {
                    out.push_str(&format!(
                        "            sel_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::RadioGroup | WidgetKind::ComboBox => {
                    let sel = if w.props.items.is_empty() {
                        0
                    } else {
                        w.props.selected.min(w.props.items.len() - 1)
                    };
                    out.push_str(&format!("            sel_{}: {},\n", w.id, sel));
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!(
                        "            open_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::DatePicker => {
                    let y = w.props.year;
                    let m = w.props.month.clamp(1, 12);
                    let d = w.props.day.clamp(1, 28);
                    out.push_str(&format!(
                        "            date_{}: NaiveDate::from_ymd_opt({}, {}, {}).unwrap(),\n",
                        w.id, y, m, d
                    ));
                }
                WidgetKind::Password => {
                    out.push_str(&format!(
                        "            pass_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                WidgetKind::AngleSelector => {
                    out.push_str(&format!(
                        "            angle_{}: {:.3},\n",
                        w.id, w.props.value
                    ));
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
        out.push_str(
            "    let (_resp, _p) = ui.allocate_painter(canvas.size(), egui::Sense::hover());\n\n",
        );

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
                WidgetKind::ImageTextButton => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(\
							ui.min_rect().min + egui::vec2({x:.1},{y:.1}), \
							egui::vec2({w:.1},{h:.1})), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::Button::new(format!(\"{{}}  {{}}\", \"{icon}\", \"{text}\")) \
							); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        icon = escape(&w.props.icon),
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::Checkbox => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.checkbox(&mut state.checked_{}, \"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, w.id, escape(&w.props.text)
                    ));
                }
                WidgetKind::TextEdit => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::TextEdit::singleline(&mut state.text_{}).hint_text(\"{}\")); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id, escape(&w.props.text)
                    ));
                }
                WidgetKind::Slider => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::Slider::new(&mut state.value_{}, {:.3}..={:.3}).text(\"{}\")); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id, w.props.min, w.props.max, escape(&w.props.text)
                    ));
                }
                WidgetKind::ProgressBar => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::ProgressBar::new(state.progress_{}).show_percentage()); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id
                    ));
                }
                WidgetKind::RadioGroup => {
                    let items_code = if w.props.items.is_empty() {
                        "\"Item\".to_string()".to_owned()
                    } else {
                        w.props
                            .items
                            .iter()
                            .map(|s| format!("\"{}\".to_string()", escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{\n",
                        pos.x, pos.y, size.x, size.y
                    ));
                    out.push_str(&format!("        let items = vec![{}];\n", items_code));
                    out.push_str(&format!(
                        "        for (i, it) in items.iter().enumerate() {{ if ui.add(egui::RadioButton::new(state.sel_{} == i, it)).clicked() {{ state.sel_{} = i; }} }}\n",
                        w.id, w.id
                    ));
                    out.push_str("    });\n");
                }
                WidgetKind::Link => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.link(\"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text)
                    ));
                }
                WidgetKind::Hyperlink => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.hyperlink_to(\"{}\", \"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), escape(&w.props.url)
                    ));
                }
                WidgetKind::SelectableLabel => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ if ui.add(egui::SelectableLabel::new(state.sel_{}, \"{}\")).clicked() {{ state.sel_{} = !state.sel_{}; }} }});\n",
                        pos.x, pos.y, size.x, size.y, w.id, escape(&w.props.text), w.id, w.id
                    ));
                }
                WidgetKind::ComboBox => {
                    let items_code = if w.props.items.is_empty() {
                        "\"Item\".to_string()".to_owned()
                    } else {
                        w.props
                            .items
                            .iter()
                            .map(|s| format!("\"{}\".to_string()", escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1})), |ui| {{\n",
						x = pos.x, y = pos.y, w = size.x, h = size.y
					));
                    out.push_str(&format!(
                        "        let items = vec![{items}];\n",
                        items = items_code
                    ));
                    out.push_str(&format!(
                        "        egui::ComboBox::from_id_source({id})\n",
                        id = w.id
                    ));
                    out.push_str(&format!("            .width({:.1})\n", size.x));
                    out.push_str(&format!(
						"            .selected_text(items.get(state.sel_{id}).cloned().unwrap_or_else(|| \"\".to_string()))\n",
						id = w.id
					));
                    out.push_str("            .show_ui(ui, |ui| {\n");
                    out.push_str(&format!(
						"                for (i, it) in items.iter().enumerate() {{ ui.selectable_value(&mut state.sel_{id}, i, it.clone()); }}\n",
						id = w.id
					));
                    out.push_str("            });\n");
                    out.push_str("    });\n");
                }
                WidgetKind::Separator => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.separator(); }});\n",
                        pos.x, pos.y, size.x, size.y
                    ));
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ egui::CollapsingHeader::new(\"{}\").default_open(state.open_{}).show(ui, |ui| {{ ui.label(\"… place your inner content here …\"); }}); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), w.id
                    ));
                }
                WidgetKind::DatePicker => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(ui.min_rect().min + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1})), |ui| {{ ui.horizontal(|ui| {{ ui.label(\"{}\"); ui.add(DatePickerButton::new(&mut state.date_{})); }}); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), w.id
                    ));
                }
                WidgetKind::Password => {
                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(\
							ui.min_rect().min + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1})), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::TextEdit::singleline(&mut state.pass_{id}).password(true).hint_text(\"password\") \
							); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id
                    ));
                }
                WidgetKind::AngleSelector => {
                    out.push_str(&format!(
						"    ui.allocate_ui_at_rect(egui::Rect::from_min_size(\
							ui.min_rect().min + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1})), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::Slider::new(&mut state.angle_{id}, {min:.3}..={max:.3}).suffix(\"°\").text(\"{label}\") \
							); \
						}});\n",
						x=pos.x,y=pos.y,w=size.x,h=size.y,id=w.id,
						min=w.props.min, max=w.props.max, label=escape(&w.props.text)
					));
                }
                WidgetKind::Tree => {
                    // Helpers live only in the generator (not emitted), so we can use any Rust we want here:
                    #[derive(Clone)]
                    struct Node {
                        label: String,
                        children: Vec<Node>,
                    }

                    fn parse_nodes(lines: &[String]) -> Vec<Node> {
                        let items: Vec<(usize, String)> = lines
                            .iter()
                            .map(|s| {
                                let indent = s.chars().take_while(|c| *c == ' ').count() / 2;
                                (indent, s.trim().to_string())
                            })
                            .filter(|(_, s)| !s.is_empty())
                            .collect();

                        fn build<I: Iterator<Item = (usize, String)>>(
                            it: &mut std::iter::Peekable<I>,
                            level: usize,
                        ) -> Vec<Node> {
                            let mut out = Vec::new();
                            while let Some((ind, _)) = it.peek().cloned() {
                                if ind < level {
                                    break;
                                }
                                if ind > level {
                                    break;
                                }
                                let (_, label) = it.next().unwrap();
                                let children = build(it, level + 1);
                                out.push(Node { label, children });
                            }
                            out
                        }

                        let mut it = items.into_iter().peekable();
                        build(&mut it, 0)
                    }

                    fn nodes_to_literal(nodes: &[Node]) -> String {
                        fn one(n: &Node) -> String {
                            let kids = if n.children.is_empty() {
                                "vec![]".to_string()
                            } else {
                                format!(
                                    "vec![{}]",
                                    n.children.iter().map(one).collect::<Vec<_>>().join(", ")
                                )
                            };
                            format!(
                                "GenTreeNode {{ label: \"{}\".to_string(), children: {} }}",
                                crate::widget::escape(&n.label),
                                kids
                            )
                        }
                        format!(
                            "vec![{}]",
                            nodes.iter().map(one).collect::<Vec<_>>().join(", ")
                        )
                    }

                    let items = if w.props.items.is_empty() {
                        vec!["Root".into(), "  Child".into()]
                    } else {
                        w.props.items.clone()
                    };

                    let nodes_literal = {
                        let nodes = parse_nodes(&items);
                        nodes_to_literal(&nodes)
                    };

                    out.push_str(&format!(
                        "    ui.allocate_ui_at_rect(egui::Rect::from_min_size(\
							ui.min_rect().min + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1})), |ui| {{ \
							let nodes: Vec<GenTreeNode> = {nodes}; \
							egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {{ \
								gen_show_tree(ui, &nodes); \
							}}); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        nodes = nodes_literal
                    ));
                }
            }
        }

        out.push_str("}\n\n");
        out.push_str("// Example eframe app to host the generated UI\n");
        out.push_str("pub struct GeneratedApp { state: GeneratedState }\n");
        out.push_str("impl Default for GeneratedApp { fn default() -> Self { Self { state: Default::default() } } }\n");
        out.push_str("impl eframe::App for GeneratedApp {\n");
        out.push_str(
            "    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n",
        );
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

impl eframe::App for RadBuilderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| self.top_bar(ui));

        if self.palette_open {
            egui::SidePanel::left("palette")
                .resizable(true)
                .show(ctx, |ui| {
                    self.palette_ui(ui);
                });
        }

        egui::SidePanel::right("inspector")
            .default_width(260.0)
            .show(ctx, |ui| {
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

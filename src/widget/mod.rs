use std::fmt;

use egui::{Pos2, Vec2, pos2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct WidgetId(u64);

impl WidgetId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_z(&self) -> i32 {
        self.0 as i32
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Widget {
    pub(crate) id: WidgetId,
    pub(crate) kind: WidgetKind,
    pub(crate) pos: Pos2,  // Top-left relative to canvas
    pub(crate) size: Vec2, // Desired size on canvas
    pub(crate) z: i32,     // draw order
    pub(crate) props: WidgetProps,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub(crate) enum WidgetKind {
    Label,
    Button,
    Checkbox,
    TextEdit,
    Slider,
    ProgressBar,
    RadioGroup,
    Link,
    Hyperlink,
    SelectableLabel,
    ComboBox,
    Separator,
    CollapsingHeader,
    DatePicker,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WidgetProps {
    pub(crate) text: String,  // label/button/textedit placeholder
    pub(crate) checked: bool, // checkbox
    pub(crate) value: f32,    // slider/progress
    pub(crate) min: f32,
    pub(crate) max: f32,
    // lists (for radio/combobox)
    pub(crate) items: Vec<String>,
    pub(crate) selected: usize,
    // hyperlinks
    pub(crate) url: String,
    // date (stored as y/m/d to avoid chrono serde feature requirements)
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self {
            text: "Label".into(),
            checked: false,
            value: 0.5,
            min: 0.0,
            max: 1.0,
            items: vec![],
            selected: 0,
            url: "https://example.com".into(),
            year: 2024,
            month: 1,
            day: 1,
        }
    }
}

pub(crate) fn snap_pos_with_grid(p: Pos2, grid: f32) -> Pos2 {
    pos2((p.x / grid).round() * grid, (p.y / grid).round() * grid)
}

pub(crate) fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

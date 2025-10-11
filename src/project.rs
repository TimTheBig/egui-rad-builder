use egui::{Vec2, vec2};
use serde::{Deserialize, Serialize};

use crate::widget::Widget;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) widgets: Vec<Widget>,
    pub(crate) canvas_size: Vec2,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            canvas_size: vec2(1200.0, 800.0),
        }
    }
}

//! A lightweight RAD GUI builder for `egui` written in Rust.

mod app;
mod project;
mod widget;

use crate::app::RadBuilderApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui RAD GUI Builder",
        native_options,
        Box::new(|_cc| Ok(Box::<RadBuilderApp>::default())),
    )
}

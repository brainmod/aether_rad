mod app;
mod compiler;
mod model;
mod syntax;
mod theme;
mod ui;
mod validator;
mod widgets;

use crate::app::AetherApp;

fn main() -> eframe::Result {
    // Define native window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Aether RAD")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Aether RAD",
        native_options,
        Box::new(|cc| Ok(Box::new(AetherApp::new(cc)))),
    )
}

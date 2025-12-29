use super::EditorContext;
use crate::compiler::Compiler;
use crate::syntax;
use crate::theme;
use egui::{Color32, CornerRadius, RichText, Ui};

pub fn render_code_preview(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);
    ui.label(theme::heading("Generated Code"));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Live preview of output")
            .size(11.0)
            .color(theme::muted_color(ui.ctx())),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Cargo.toml
        render_code_section(
            ui,
            "📦 Cargo.toml",
            &Compiler::generate_cargo_toml(&ctx.project_state.project_name),
        );

        ui.add_space(8.0);

        // main.rs
        render_code_section(ui, "🚀 src/main.rs", &Compiler::generate_main_rs());

        ui.add_space(8.0);

        // app.rs
        render_code_section(
            ui,
            "⚙️ src/app.rs",
            &Compiler::generate_app_rs(&ctx.project_state),
        );
    });
}

/// Render a code section with header and syntax highlighting
fn render_code_section(ui: &mut Ui, title: &str, code: &str) {
    theme::section_frame(ui.ctx()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(title)
                    .size(12.0)
                    .strong()
                    .color(theme::accent_light_color(ui.ctx())),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} lines", code.lines().count()))
                        .size(10.0)
                        .color(theme::muted_color(ui.ctx())),
                );
            });
        });
        ui.add_space(6.0);

        // Code display with syntax highlighting
        let is_light = !ui.ctx().style().visuals.dark_mode;
        let code_bg = if is_light {
            Color32::from_rgb(245, 245, 248)
        } else {
            Color32::from_rgb(25, 25, 30)
        };

        egui::Frame::new()
            .fill(code_bg)
            .inner_margin(egui::Margin::same(8))
            .corner_radius(CornerRadius::same(4))
            .show(ui, |ui| {
                // Choose highlighter based on file type
                let mut highlighted_code = if title.contains("Cargo.toml") {
                    syntax::highlight_toml(code, is_light)
                } else {
                    // Rust files
                    syntax::highlight_rust(code, is_light)
                };

                // Set font size for the layout job
                for section in &mut highlighted_code.sections {
                    section.format.font_id.size = 11.0;
                }

                ui.add(egui::Label::new(highlighted_code).wrap());
            });
    });
}

use super::EditorContext;
use crate::theme;
use egui::{RichText, Ui};

pub fn render_assets(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);
    ui.label(theme::heading("Project Assets"));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Manage images and other resources")
            .size(11.0)
            .color(theme::muted_color(ui.ctx())),
    );
    ui.add_space(8.0);

    // Add asset button
    if ui
        .add(egui::Button::new(
            RichText::new("+ Add Image").color(theme::success_color(ui.ctx())),
        ))
        .clicked()
    {
        // Open file picker to select an image
        if let Some(path) = crate::io::pick_file("Images") {
            // Extract filename as asset name
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("asset_{}", ctx.project_state.assets.assets.len()));

            // Add asset to project
            ctx.project_state.assets.add_asset(
                name,
                crate::model::AssetType::Image,
                path,
            );
        }
    }

    ui.add_space(8.0);

    // Asset list
    if ctx.project_state.assets.assets.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                RichText::new("No assets yet")
                    .size(12.0)
                    .color(theme::muted_color(ui.ctx())),
            );
            ui.label(
                RichText::new("Click 'Add Image' to import assets")
                    .size(11.0)
                    .color(theme::muted_color(ui.ctx())),
            );
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_remove = None;

            for (name, asset) in &ctx.project_state.assets.assets {
                theme::section_frame(ui.ctx()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).strong());
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("✕").color(theme::error_color(ui.ctx())),
                                        )
                                        .small(),
                                    )
                                    .clicked()
                                {
                                    to_remove = Some(name.clone());
                                }
                            },
                        );
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Type: {}", asset.asset_type))
                            .size(10.0)
                            .color(theme::muted_color(ui.ctx())),
                    );
                    ui.label(
                        RichText::new(format!("Path: {}", asset.path.display()))
                            .size(10.0)
                            .color(theme::muted_color(ui.ctx())),
                    );
                });
            }

            if let Some(name) = to_remove {
                ctx.project_state.assets.remove_asset(&name);
            }
        });
    }
}

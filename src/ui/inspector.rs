use super::EditorContext;
use crate::theme;
use egui::{RichText, Ui};

pub fn render_inspector(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);

    // Check if multiple widgets are selected
    let selection_count = ctx.project_state.selection.len();

    if selection_count == 0 {
        ui.label(theme::heading("Inspector"));
        ui.add_space(8.0);
        ui.label(RichText::new("No widget selected").color(theme::muted_color(ui.ctx())));
    } else if selection_count > 1 {
        // Multi-selection mode
        ui.label(theme::heading(&format!("{} Widgets Selected", selection_count)));
        ui.add_space(8.0);
        ui.label(
            RichText::new("Multi-selection: Edit properties for all selected widgets")
                .color(theme::muted_color(ui.ctx()))
                .size(11.0),
        );
        ui.add_space(8.0);

        // Show common actions for all selected widgets
        theme::section_frame(ui.ctx()).show(ui, |ui| {
            ui.label(theme::subheading("Bulk Actions"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("Delete All").clicked() {
                    for id in ctx.project_state.selection.clone() {
                        ctx.project_state.delete_widget(id);
                    }
                }
            });
        });
    } else if let Some(id) = ctx.project_state.selection.iter().next().cloned() {
        let known_vars: Vec<String> = ctx.project_state.variables.keys().cloned().collect();
        // Build (name, filename) pairs for asset selection
        let known_assets: Vec<(String, String)> = ctx.project_state.assets.assets.values()
            .map(|asset| {
                let filename = asset.path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(&asset.name)
                    .to_string();
                (asset.name.clone(), filename)
            })
            .collect();
        let is_root = id == ctx.project_state.root_node.id();

        // Get widget name for header
        let widget_name = if let Some(node) = ctx.project_state.find_node_mut(id) {
            node.name().to_string()
        } else {
            "Unknown".to_string()
        };

        // Header with widget label
        ui.label(theme::heading(&format!(
            "{} Properties",
            widget_name
        )));
        ui.add_space(4.0);

        // If root layout is selected, show layout type switcher
        if is_root {
            theme::section_frame(ui.ctx()).show(ui, |ui| {
                ui.label(theme::subheading("Root Layout Type"));
                ui.add_space(4.0);

                let current_type = ctx.project_state.root_layout_type();
                let mut selected_type = current_type.to_string();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut selected_type,
                        "Vertical Layout".to_string(),
                        "⬇ Vertical",
                    );
                    ui.selectable_value(
                        &mut selected_type,
                        "Horizontal Layout".to_string(),
                        "➡ Horizontal",
                    );
                    ui.selectable_value(
                        &mut selected_type,
                        "Grid Layout".to_string(),
                        "⊞ Grid",
                    );
                });

                if selected_type != current_type {
                    ctx.project_state.set_root_layout_type(&selected_type);
                }
            });
            ui.add_space(8.0);
        }

        // Widget properties
        if let Some(node) = ctx.project_state.find_node_mut(id) {
            theme::section_frame(ui.ctx()).show(ui, |ui| {
                node.inspect(ui, &known_vars, &known_assets);
            });

            // Widget actions (not for root)
            if !is_root {
                ui.add_space(8.0);
                theme::section_frame(ui.ctx()).show(ui, |ui| {
                    ui.label(theme::subheading("Actions"));
                    ui.add_space(6.0);

                    // Reorder buttons
                    ui.horizontal(|ui| {
                        if ui.button("⬆ Move Up").clicked() {
                            ctx.project_state.move_widget_up(id);
                        }
                        if ui.button("⬇ Move Down").clicked() {
                            ctx.project_state.move_widget_down(id);
                        }
                    });

                    ui.add_space(8.0);

                    // Delete button with warning color
                    if ui
                        .add(egui::Button::new(
                            RichText::new("✕ Delete Widget").color(theme::error_color(ui.ctx())),
                        ))
                        .clicked()
                    {
                        if ctx.project_state.delete_widget(id) {
                            ctx.project_state.selection.clear();
                        }
                    }
                });
            }

            return;
        }
    }

    // No selection state
    ui.label(theme::heading("Inspector"));
    ui.add_space(8.0);
    theme::section_frame(ui.ctx()).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("↓").size(32.0));
            ui.add_space(8.0);
            ui.label(RichText::new("Select a widget").color(theme::muted_color(ui.ctx())));
            ui.label(
                RichText::new("to view its properties")
                    .size(11.0)
                    .color(theme::muted_color(ui.ctx())),
            );
            ui.add_space(20.0);
        });
    });
}

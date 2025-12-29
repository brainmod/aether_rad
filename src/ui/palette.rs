use super::EditorContext;
use crate::theme::{self, AetherColors};
use egui::{Color32, CornerRadius, RichText, Ui};

pub fn render_palette(ui: &mut Ui, _ctx: &mut EditorContext) {
    ui.add_space(4.0);
    ui.label(theme::heading("Widget Palette"));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Drag widgets to the canvas")
            .size(11.0)
            .color(theme::muted_color(ui.ctx())),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Layout widgets
        render_widget_category(
            ui,
            "Layouts",
            &["Vertical Layout", "Horizontal Layout", "Grid Layout", "Freeform Layout", "Scroll Area", "Tab Container", "Window"],
            AetherColors::LAYOUT_COLOR,
        );

        ui.add_space(8.0);

        // Input widgets
        render_widget_category(
            ui,
            "Inputs",
            &["Button", "Checkbox", "Slider", "Text Edit", "ComboBox"],
            AetherColors::INPUT_COLOR,
        );

        ui.add_space(8.0);

        // Display widgets
        render_widget_category(
            ui,
            "Display",
            &[
                "Label",
                "Table",
                "Plot",
                "Progress Bar",
                "Image",
                "Separator",
                "Spinner",
                "Hyperlink",
                "Color Picker",
            ],
            AetherColors::DISPLAY_COLOR,
        );
    });
}

/// Render a categorized widget section in the palette
fn render_widget_category(ui: &mut Ui, category: &str, widgets: &[&str], accent_color: Color32) {
    let header = egui::CollapsingHeader::new(
        RichText::new(category)
            .size(13.0)
            .strong()
            .color(accent_color),
    )
    .default_open(true);

    header.show(ui, |ui| {
        for widget_type in widgets {
            let label = theme::WidgetLabels::get(widget_type);
            let id = egui::Id::new("palette").with(*widget_type);

            // Check if we're currently dragging this widget
            let is_being_dragged = ui.ctx().is_being_dragged(id);

            if is_being_dragged {
                // Set the dragged widget type in memory for the canvas to read
                ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("dragged_widget_type"), widget_type.to_string()));

                // Show a ghost/preview at the cursor position
                egui::Area::new(egui::Id::new("drag_preview").with(*widget_type))
                    .order(egui::Order::Tooltip)
                    .fixed_pos(ui.ctx().pointer_hover_pos().unwrap_or_default())
                    .show(ui.ctx(), |ui| {
                        egui::Frame::new()
                            .fill(ui.style().visuals.window_fill)
                            .stroke(egui::Stroke::new(2.0, accent_color))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(egui::Margin::same(8))
                            .shadow(egui::Shadow::NONE)
                            .show(ui, |ui| {
                                // Show a preview of what the widget looks like
                                crate::widgets::render_widget_preview(ui, widget_type, accent_color);
                            });
                    });
            }

            ui.dnd_drag_source(id, widget_type.to_string(), |ui| {
                let response = ui.add(
                    egui::Button::new(
                        RichText::new(label)
                            .color(accent_color)
                    )
                    .min_size(egui::vec2(ui.available_width() - 8.0, 28.0)),
                );

                // Show drag hint on hover
                response.on_hover_text("Drag to canvas to add");
            });
            ui.add_space(4.0);
        }
    });
}



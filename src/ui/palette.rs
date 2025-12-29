use super::EditorContext;
use crate::theme::{self, AetherColors};
use crate::widgets;
use egui::{Color32, CornerRadius, RichText, Ui};

pub fn render_palette(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);
    ui.label(theme::heading("Widget Palette"));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Click or drag to add widgets")
            .size(11.0)
            .color(theme::muted_color(ui.ctx())),
    );
    ui.add_space(8.0);

    // Track if any widget was clicked for adding
    let mut widget_to_add: Option<String> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Layout widgets
        if let Some(w) = render_widget_category(
            ui,
            "Layouts",
            &["Vertical Layout", "Horizontal Layout", "Grid Layout", "Freeform Layout", "Scroll Area", "Tab Container", "Window"],
            AetherColors::LAYOUT_COLOR,
        ) {
            widget_to_add = Some(w);
        }

        ui.add_space(8.0);

        // Input widgets
        if let Some(w) = render_widget_category(
            ui,
            "Inputs",
            &["Button", "Checkbox", "Slider", "Text Edit", "ComboBox"],
            AetherColors::INPUT_COLOR,
        ) {
            widget_to_add = Some(w);
        }

        ui.add_space(8.0);

        // Display widgets
        if let Some(w) = render_widget_category(
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
        ) {
            widget_to_add = Some(w);
        }
    });

    // Handle click-to-add: add widget to root or selected container
    if let Some(widget_type) = widget_to_add {
        if let Some(new_widget) = widgets::create_widget_by_name(&widget_type) {
            // Try to add to selected container, or fall back to root
            let target_id = ctx.project_state.selection.iter().next().cloned();

            if let Some(id) = target_id {
                // Check if selected widget is a container
                if let Some(node) = ctx.project_state.find_node_mut(id) {
                    if let Some(children) = node.children_mut() {
                        children.push(new_widget);
                        return;
                    }
                }
            }

            // Fall back to root
            if let Some(children) = ctx.project_state.root_node.children_mut() {
                children.push(new_widget);
            }
        }
    }
}

/// Render a categorized widget section in the palette
/// Returns the widget type name if a widget was clicked for adding
fn render_widget_category(ui: &mut Ui, category: &str, widgets: &[&str], accent_color: Color32) -> Option<String> {
    let mut clicked_widget: Option<String> = None;

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

            let dnd_response = ui.dnd_drag_source(id, widget_type.to_string(), |ui| {
                let response = ui.add(
                    egui::Button::new(
                        RichText::new(label)
                            .color(accent_color)
                    )
                    .min_size(egui::vec2(ui.available_width() - 8.0, 28.0)),
                );

                // Show hint on hover
                response.on_hover_text("Click to add, or drag to canvas");
            });

            // Check for click (not drag) to add widget
            if dnd_response.response.clicked() {
                clicked_widget = Some(widget_type.to_string());
            }

            ui.add_space(4.0);
        }
    });

    clicked_widget
}



use super::EditorContext;
use crate::theme::AetherColors;
use egui::{Color32, CornerRadius, RichText, Ui};

pub fn render_canvas(ui: &mut Ui, ctx: &mut EditorContext) {
    // CENTER: The main visual editor with styled frame
    let is_light = !ui.ctx().style().visuals.dark_mode;
    let outer_bg = if is_light {
        Color32::from_rgb(250, 250, 252)
    } else {
        Color32::from_rgb(30, 30, 35)
    };
    let muted_color = if is_light {
        Color32::from_rgb(120, 120, 130)
    } else {
        AetherColors::MUTED
    };

    egui::Frame::new()
        .fill(outer_bg)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            // Canvas header with zoom controls
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("■ Design Canvas")
                        .size(12.0)
                        .color(muted_color),
                );

                ui.separator();

                // Zoom controls
                if ui.button("−").clicked() {
                    *ctx.canvas_zoom = (*ctx.canvas_zoom - 0.05).max(0.25);
                }
                ui.add(
                    egui::Slider::new(ctx.canvas_zoom, 0.25..=3.0)
                        .step_by(0.05)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .custom_parser(|s| s.trim_end_matches('%').parse::<f64>().ok().map(|v| v / 100.0)),
                );
                if ui.button("+").clicked() {
                    *ctx.canvas_zoom = (*ctx.canvas_zoom + 0.05).min(3.0);
                }
                if ui.button("100%").clicked() {
                    *ctx.canvas_zoom = 1.0;
                }
                if ui.button("Fit").clicked() {
                    *ctx.canvas_zoom = 1.0;
                    *ctx.canvas_pan = egui::Vec2::ZERO;
                }

                // Show current zoom percentage
                ui.label(
                    RichText::new(format!("{}%", (*ctx.canvas_zoom * 100.0) as i32))
                        .size(11.0)
                        .color(muted_color),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Root: {}",
                            ctx.project_state.root_layout_type()
                        ))
                        .size(11.0)
                        .color(muted_color),
                    );
                });
            });
            ui.add_space(8.0);

            // Canvas content area with scroll and zoom
            let canvas_bg = if is_light {
                Color32::from_rgb(240, 240, 245)
            } else {
                Color32::from_rgb(40, 40, 48)
            };
            let canvas_stroke = if is_light {
                Color32::from_rgb(220, 220, 225)
            } else {
                Color32::from_rgb(55, 55, 65)
            };

            let canvas_rect = egui::Frame::new()
                .fill(canvas_bg)
                .inner_margin(egui::Margin::same(12))
                .corner_radius(CornerRadius::same(8))
                .stroke(egui::Stroke::new(1.0, canvas_stroke))
                .show(ui, |ui| {
                    let zoom = *ctx.canvas_zoom;

                    // Draw subtle grid pattern for visual reference
                    let grid_spacing = 20.0 * zoom;
                    let canvas_area = ui.available_rect_before_wrap();
                    let grid_color = if is_light {
                        Color32::from_rgba_unmultiplied(0, 0, 0, 15)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
                    };

                    // Draw grid lines
                    let painter = ui.painter();
                    let start_x = canvas_area.left() - (canvas_area.left() % grid_spacing);
                    let start_y = canvas_area.top() - (canvas_area.top() % grid_spacing);

                    let mut x = start_x;
                    while x < canvas_area.right() {
                        painter.line_segment(
                            [egui::pos2(x, canvas_area.top()), egui::pos2(x, canvas_area.bottom())],
                            egui::Stroke::new(1.0, grid_color),
                        );
                        x += grid_spacing;
                    }

                    let mut y = start_y;
                    while y < canvas_area.bottom() {
                        painter.line_segment(
                            [egui::pos2(canvas_area.left(), y), egui::pos2(canvas_area.right(), y)],
                            egui::Stroke::new(1.0, grid_color),
                        );
                        y += grid_spacing;
                    }

                    // Scroll area for panning with offset
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .scroll_offset(*ctx.canvas_pan)
                        .show(ui, |ui| {
                            // Apply zoom by scaling the UI
                            ui.style_mut().spacing.item_spacing *= zoom;
                            ui.style_mut().spacing.button_padding *= zoom;
                            ui.style_mut().spacing.indent *= zoom;

                            // Scale text sizes for widgets (with safety floor to prevent panic)
                            let original_text_style = ui.style().text_styles.clone();
                            for (_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                                font_id.size = (font_id.size * zoom).max(4.0);
                            }

                            // Add some padding at the scaled level
                            ui.add_space(8.0 * zoom);

                            // Render the widget tree
                            ctx.project_state
                                .root_node
                                .render_editor(ui, &mut ctx.project_state.selection);

                            // Restore original text styles
                            ui.style_mut().text_styles = original_text_style;
                        });
                }).response.rect;

            // Handle Ctrl+scroll wheel for zooming
            let canvas_response = ui.interact(canvas_rect, egui::Id::new("canvas_zoom_pan"), egui::Sense::hover());
            if canvas_response.hovered() {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta);
                let modifiers = ui.input(|i| i.modifiers);

                if modifiers.ctrl && scroll_delta.y != 0.0 {
                    // Ctrl + scroll = zoom
                    let zoom_delta = scroll_delta.y * 0.001;
                    *ctx.canvas_zoom = (*ctx.canvas_zoom + zoom_delta).clamp(0.25, 3.0);
                }
            }

            // Handle middle-mouse drag for panning
            let pan_response = ui.interact(canvas_rect, egui::Id::new("canvas_pan_drag"), egui::Sense::drag());
            if pan_response.dragged_by(egui::PointerButton::Middle) {
                *ctx.canvas_pan -= pan_response.drag_delta();
            }
        });
}

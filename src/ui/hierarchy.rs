use super::EditorContext;
use crate::model::WidgetNode;
use crate::theme::{self, AetherColors};
use egui::{Color32, CornerRadius, RichText, Ui};
use std::collections::HashSet;
use uuid::Uuid;

/// Payload for hierarchy drag-and-drop
#[derive(Clone)]
#[allow(dead_code)]
struct HierarchyDragPayload {
    widget_id: Uuid,
    widget_name: String,
}

/// Position for drop operations
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum DropPosition {
    Before,
    After,
    Into,
}

pub fn render_hierarchy(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);
    ui.label(theme::heading("Widget Tree"));
    ui.add_space(4.0);

    // Keyboard navigation hint
    ui.label(
        RichText::new("↑↓ Nav • Ctrl+↑↓ Reorder • Drag to move")
            .size(10.0)
            .color(theme::muted_color(ui.ctx())),
    );
    ui.add_space(8.0);

    // Keyboard navigation for hierarchy
    if ui.ui_contains_pointer() {
        let all_ids = ctx.project_state.get_all_widget_ids();
        let current_selected = ctx.project_state.selection.iter().next().cloned();

        // Check for Ctrl key held (for reordering)
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowUp) {
                if let Some(current) = current_selected {
                    if ctrl_held {
                        // Ctrl+Up = Move widget up in parent
                        ctx.project_state.move_widget_up(current);
                    } else {
                        // Just navigate up
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx > 0 {
                                ctx.project_state.selection.clear();
                                ctx.project_state.selection.insert(all_ids[current_idx - 1]);
                            }
                        }
                    }
                } else if !all_ids.is_empty() {
                    ctx.project_state.selection.insert(all_ids[0]);
                }
            }

            if i.key_pressed(egui::Key::ArrowDown) {
                if let Some(current) = current_selected {
                    if ctrl_held {
                        // Ctrl+Down = Move widget down in parent
                        ctx.project_state.move_widget_down(current);
                    } else {
                        // Just navigate down
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx < all_ids.len() - 1 {
                                ctx.project_state.selection.clear();
                                ctx.project_state.selection.insert(all_ids[current_idx + 1]);
                            }
                        }
                    }
                } else if !all_ids.is_empty() {
                    ctx.project_state.selection.insert(all_ids[0]);
                }
            }

            if i.key_pressed(egui::Key::Escape) {
                ctx.project_state.selection.clear();
            }

            // Delete key to remove selected widget
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if let Some(current) = current_selected {
                    if current != ctx.project_state.root_node.id() {
                        ctx.project_state.delete_widget(current);
                        ctx.project_state.selection.clear();
                    }
                }
            }
        });
    }

    // Track pending drop operation
    let mut pending_drop: Option<(Uuid, Uuid, DropPosition)> = None;

    // Tree view with styled frame
    theme::section_frame(ui.ctx()).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            draw_hierarchy_node_styled(
                ui,
                ctx.project_state.root_node.as_ref(),
                &mut ctx.project_state.selection,
                0,
                &mut pending_drop,
            );
        });
    });

    // Handle any pending drop operations
    if let Some((source_id, target_id, position)) = pending_drop {
        // Get the currently selected widget as the drag source if source_id is nil
        let actual_source_id = if source_id == Uuid::nil() {
            ctx.project_state.selection.iter().next().cloned()
        } else {
            Some(source_id)
        };

        if let Some(src_id) = actual_source_id {
            match position {
                DropPosition::Before => {
                    ctx.project_state.move_widget_before(src_id, target_id);
                }
                DropPosition::After => {
                    ctx.project_state.move_widget_after(src_id, target_id);
                }
                DropPosition::Into => {
                    // Move into container at end
                    ctx.project_state.reparent_widget(src_id, target_id, usize::MAX);
                }
            }
        }
    }
}

/// Styled hierarchy node rendering with icons, depth indication, and DnD support
fn draw_hierarchy_node_styled(
    ui: &mut Ui,
    node: &dyn WidgetNode,
    selection: &mut HashSet<Uuid>,
    depth: usize,
    pending_drop: &mut Option<(Uuid, Uuid, DropPosition)>,
) {
    let id = node.id();
    let is_selected = selection.contains(&id);
    let label = theme::WidgetLabels::get(node.name());
    let category_color = theme::widget_category_color(node.name());

    let children = node.children();
    let has_children = children.map_or(false, |c| !c.is_empty());
    let is_container = node.children().is_some();

    // Indent based on depth
    let indent = depth as f32 * 12.0;

    // Read modifiers BEFORE entering any nested closures to avoid deadlock
    let cmd_held = ui.input(|i| i.modifiers.command);

    // Create drag payload
    let drag_id = egui::Id::new("hierarchy_drag").with(id);
    let payload = HierarchyDragPayload {
        widget_id: id,
        widget_name: label.to_string(),
    };

    // Check if this widget is being dragged
    let is_being_dragged = ui.ctx().is_being_dragged(drag_id);

    // Don't render if being dragged (will be shown as floating preview)
    if is_being_dragged {
        // Show dragged item preview at cursor
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            egui::Area::new(egui::Id::new("hierarchy_drag_preview"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos + egui::vec2(10.0, 10.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(ui.style().visuals.window_fill)
                        .stroke(egui::Stroke::new(2.0, category_color))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(egui::Margin::same(4))
                        .shadow(egui::Shadow::NONE)
                        .show(ui, |ui| {
                            ui.label(RichText::new(&payload.widget_name).color(category_color));
                        });
                });
        }
    }

    // Main hierarchy item rendering
    if has_children {
        let display_text = label.to_string();
        let text_color = if is_selected {
            AetherColors::ACCENT_LIGHT
        } else {
            category_color
        };

        // Drop zone for inserting BEFORE this container
        let _before_drop_id = egui::Id::new("drop_before").with(id);
        let (before_rect, before_response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 2.0),
            egui::Sense::hover(),
        );

        // Check for drop on "before" zone
        if let Some(dragged_payload) = ui.ctx().dragged_id().and_then(|did| {
            if before_response.hovered() && did.with("hierarchy_drag") != drag_id {
                // Visual drop indicator
                ui.painter().rect_filled(before_rect.expand2(egui::vec2(0.0, 2.0)), 0.0, AetherColors::ACCENT);
                Some(true)
            } else {
                None
            }
        }) {
            if dragged_payload && ui.input(|i| i.pointer.any_released()) {
                // Get the dragged widget ID from any currently dragged item
                // We need to track this differently
            }
        }

        ui.horizontal(|ui| {
            ui.add_space(indent);

            let state_id = ui.make_persistent_id(id);
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), state_id, true);

            let _ = state.show_header(ui, |ui| {
                // Make ONLY the header content draggable
                ui.dnd_drag_source(drag_id, payload.clone(), |ui| {
                    let response = ui.selectable_label(
                        is_selected,
                        RichText::new(&display_text).color(text_color).strong(),
                    );

                    // Handle selection
                    if response.clicked() {
                        if cmd_held {
                            if selection.contains(&id) {
                                selection.remove(&id);
                            } else {
                                selection.insert(id);
                            }
                        } else {
                            selection.clear();
                            selection.insert(id);
                        }
                    }

                     // Selection indicator
                    if is_selected {
                        let rect = response.rect;
                        ui.painter().rect_stroke(
                            rect.expand(2.0),
                            4.0,
                            egui::Stroke::new(2.0, AetherColors::ACCENT),
                            egui::StrokeKind::Outside,
                        );
                    }
                });
            });
        });

        // Manually render body if open (outside of horizontal layout)
        let state_id = ui.make_persistent_id(id);
        if egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), state_id, true).is_open() {
            // Container drop zone (for dropping INTO this container)
            if is_container {
                let (drop_rect, drop_response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 8.0),
                    egui::Sense::hover(),
                );

                // Visual indicator for drop target
                if drop_response.hovered() && ui.ctx().dragged_id().is_some() {
                    ui.painter().rect_filled(
                        drop_rect,
                        2.0,
                        Color32::from_rgba_unmultiplied(100, 200, 100, 100),
                    );
                    ui.painter().rect_stroke(
                        drop_rect,
                        2.0,
                        egui::Stroke::new(2.0, AetherColors::ACCENT),
                        egui::StrokeKind::Inside,
                    );

                    // Handle drop
                    if ui.input(|i| i.pointer.any_released()) {
                        *pending_drop = Some((Uuid::nil(), id, DropPosition::Into));
                    }
                }
            }

            if let Some(children) = children {
                for child in children {
                    draw_hierarchy_node_styled(ui, child.as_ref(), selection, depth + 1, pending_drop);
                }
            }
        }
    } else {
        // Leaf node - simpler rendering with drop zones
        ui.horizontal(|ui| {
            ui.add_space(indent + 16.0);

            let display_text = label;
            let text_color = if is_selected {
                AetherColors::ACCENT_LIGHT
            } else {
                category_color
            };

            // Make the leaf draggable
            ui.dnd_drag_source(drag_id, payload, |ui| {
                let response = ui.selectable_label(
                    is_selected,
                    RichText::new(display_text).color(text_color),
                );

                if response.clicked() {
                    if cmd_held {
                        if selection.contains(&id) {
                            selection.remove(&id);
                        } else {
                            selection.insert(id);
                        }
                    } else {
                        selection.clear();
                        selection.insert(id);
                    }
                }

                // Drop indicator below this item
                if response.hovered() && ui.ctx().dragged_id().is_some() {
                    let rect = response.rect;
                    ui.painter().line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        egui::Stroke::new(2.0, AetherColors::ACCENT),
                    );

                    if ui.input(|i| i.pointer.any_released()) {
                        *pending_drop = Some((Uuid::nil(), id, DropPosition::After));
                    }
                }
            });
        });
    }
}

use super::EditorContext;
use crate::model::WidgetNode;
use crate::theme::{self, AetherColors};
use egui::{Color32, RichText, Ui};
use std::collections::HashSet;
use uuid::Uuid;

/// Payload for hierarchy drag-and-drop - contains the widget ID being dragged
#[derive(Clone, PartialEq, Eq)]
struct HierarchyDragPayload {
    widget_id: Uuid,
}

/// Position for drop operations
#[derive(Clone, Copy, Debug)]
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
        // Only proceed if we have a valid source (not the root)
        if source_id != ctx.project_state.root_node.id() && source_id != target_id {
            match position {
                DropPosition::Before => {
                    ctx.project_state.move_widget_before(source_id, target_id);
                }
                DropPosition::After => {
                    ctx.project_state.move_widget_after(source_id, target_id);
                }
                DropPosition::Into => {
                    // Move into container at end
                    ctx.project_state.reparent_widget(source_id, target_id, usize::MAX);
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

    // Create drag payload with just the widget ID
    let drag_id = egui::Id::new("hierarchy_drag").with(id);
    let payload = HierarchyDragPayload { widget_id: id };

    // Main hierarchy item rendering
    if has_children {
        let display_text = label.to_string();
        let text_color = if is_selected {
            AetherColors::ACCENT_LIGHT
        } else {
            category_color
        };

        ui.horizontal(|ui| {
            ui.add_space(indent);

            let state_id = ui.make_persistent_id(id);
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), state_id, true);

            let _ = state.show_header(ui, |ui| {
                // Make ONLY the header content draggable
                let drag_response = ui.dnd_drag_source(drag_id, payload.clone(), |ui| {
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
                }).response;

                // Check for hover during drag - show insertion indicator
                // Note: For containers, "Into" is handled by the dedicated drop zone below
                // Header only handles Before/After to avoid duplicate drop targets
                if let (Some(pointer), Some(hovered_payload)) = (
                    ui.input(|i| i.pointer.interact_pos()),
                    drag_response.dnd_hover_payload::<HierarchyDragPayload>(),
                ) {
                    let rect = drag_response.rect;
                    let stroke = egui::Stroke::new(2.0, AetherColors::ACCENT);

                    // Don't show indicator if dragging over self
                    if hovered_payload.widget_id != id {
                        if pointer.y < rect.center().y {
                            // Insert before
                            ui.painter().hline(rect.x_range(), rect.top() - 1.0, stroke);
                            if let Some(released) = drag_response.dnd_release_payload::<HierarchyDragPayload>() {
                                *pending_drop = Some((released.widget_id, id, DropPosition::Before));
                            }
                        } else {
                            // Insert after
                            ui.painter().hline(rect.x_range(), rect.bottom() + 1.0, stroke);
                            if let Some(released) = drag_response.dnd_release_payload::<HierarchyDragPayload>() {
                                *pending_drop = Some((released.widget_id, id, DropPosition::After));
                            }
                        }
                    }
                }
            });
        });

        // Manually render body if open (outside of horizontal layout)
        let state_id = ui.make_persistent_id(id);
        if egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), state_id, true).is_open() {
            // Container drop zone (for dropping INTO this container)
            if is_container {
                // Only show drop zone if something is being dragged
                let is_dragging = ui.ctx().dragged_id().is_some();
                let zone_height = if is_dragging { 16.0 } else { 4.0 };

                let (drop_rect, drop_response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), zone_height),
                    egui::Sense::hover(),
                );

                // Check for hover payload
                if let Some(hovered_payload) = drop_response.dnd_hover_payload::<HierarchyDragPayload>() {
                    // Don't allow dropping into self
                    if hovered_payload.widget_id != id {
                        ui.painter().rect_filled(
                            drop_rect,
                            2.0,
                            Color32::from_rgba_unmultiplied(100, 200, 100, 120),
                        );
                        ui.painter().rect_stroke(
                            drop_rect,
                            2.0,
                            egui::Stroke::new(2.0, AetherColors::ACCENT),
                            egui::StrokeKind::Inside,
                        );

                        // Show hint text
                        let text = format!("▸ Drop into {}", label);
                        ui.painter().text(
                            drop_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(9.0),
                            AetherColors::ACCENT,
                        );

                        // Handle drop
                        if let Some(released) = drop_response.dnd_release_payload::<HierarchyDragPayload>() {
                            *pending_drop = Some((released.widget_id, id, DropPosition::Into));
                        }
                    }
                } else if is_dragging {
                    // Show faint hint when dragging but not hovering
                    ui.painter().rect_stroke(
                        drop_rect,
                        2.0,
                        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 100, 80)),
                        egui::StrokeKind::Inside,
                    );
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
            let drag_response = ui.dnd_drag_source(drag_id, payload, |ui| {
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
            }).response;

            // Check for hover during drag - show insertion indicator
            if let (Some(pointer), Some(hovered_payload)) = (
                ui.input(|i| i.pointer.interact_pos()),
                drag_response.dnd_hover_payload::<HierarchyDragPayload>(),
            ) {
                // Don't show indicator if dragging over self
                if hovered_payload.widget_id != id {
                    let rect = drag_response.rect;
                    let stroke = egui::Stroke::new(2.0, AetherColors::ACCENT);

                    if pointer.y < rect.center().y {
                        // Insert before
                        ui.painter().hline(rect.x_range(), rect.top() - 1.0, stroke);
                        if let Some(released) = drag_response.dnd_release_payload::<HierarchyDragPayload>() {
                            *pending_drop = Some((released.widget_id, id, DropPosition::Before));
                        }
                    } else {
                        // Insert after
                        ui.painter().hline(rect.x_range(), rect.bottom() + 1.0, stroke);
                        if let Some(released) = drag_response.dnd_release_payload::<HierarchyDragPayload>() {
                            *pending_drop = Some((released.widget_id, id, DropPosition::After));
                        }
                    }
                }
            }
        });
    }
}

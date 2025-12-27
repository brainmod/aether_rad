use crate::compiler::Compiler;
use crate::model::{ProjectState, Variable, VariableType, WidgetNode};
use egui::{Ui, WidgetText};
use egui_dock::TabViewer;
use std::collections::HashSet;
use uuid::Uuid;

// Identifiers for the different panels in the IDE
#[derive(Clone, Debug, PartialEq)]
pub enum AetherTab {
    Canvas,
    Palette,
    Hierarchy,
    Inspector,
    Output,
    Variables,
    CodePreview,
}

// The "Viewer" handles the actual rendering of each tab.
// It holds a mutable reference to the ProjectState so tabs can modify it.
pub struct AetherTabViewer<'a> {
    pub project_state: &'a mut ProjectState,
}

impl<'a> TabViewer for AetherTabViewer<'a> {
    type Tab = AetherTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{:?}", tab).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            AetherTab::Canvas => self.render_canvas(ui),
            AetherTab::Palette => self.render_palette(ui),
            AetherTab::Hierarchy => self.render_hierarchy(ui),
            AetherTab::Inspector => self.render_inspector(ui),
            AetherTab::Output => self.render_output(ui),
            AetherTab::Variables => self.render_variables(ui),
            AetherTab::CodePreview => self.render_code_preview(ui),
        }
    }
}

impl<'a> AetherTabViewer<'a> {
    fn render_canvas(&mut self, ui: &mut Ui) {
        // CENTER: The main visual editor
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // In Phase 2, we just direct render the root node
            self.project_state
                .root_node
                .render_editor(ui, &mut self.project_state.selection);
        });
    }

    fn render_palette(&mut self, ui: &mut Ui) {
        ui.label("Drafting Board");
        ui.separator();

        let widgets = vec![
            "Button",
            "Label",
            "Text Edit",
            "Checkbox",
            "Slider",
            "Progress Bar",
            "ComboBox",
            "Image",
            "Vertical Layout",
            "Horizontal Layout",
            "Grid Layout",
        ];

        for widget_type in widgets {
            let id = egui::Id::new("palette").with(widget_type);
            ui.dnd_drag_source(id, widget_type.to_string(), |ui| {
                let _ = ui.button(widget_type);
            });
        }
    }

    fn render_hierarchy(&mut self, ui: &mut Ui) {
        ui.heading("Tree View");
        let ps = &mut self.project_state;

        // Keyboard navigation for hierarchy
        if ui.ui_contains_pointer() {
            ui.input(|i| {
                let all_ids = ps.get_all_widget_ids();

                // Get currently selected widget (if any)
                let current_selected = ps.selection.iter().next().cloned();

                // Arrow Up - Navigate to previous widget
                if i.key_pressed(egui::Key::ArrowUp) {
                    if let Some(current) = current_selected {
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx > 0 {
                                let prev_id = all_ids[current_idx - 1];
                                ps.selection.clear();
                                ps.selection.insert(prev_id);
                            }
                        }
                    } else if !all_ids.is_empty() {
                        // No selection, select first widget
                        ps.selection.insert(all_ids[0]);
                    }
                }

                // Arrow Down - Navigate to next widget
                if i.key_pressed(egui::Key::ArrowDown) {
                    if let Some(current) = current_selected {
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx < all_ids.len() - 1 {
                                let next_id = all_ids[current_idx + 1];
                                ps.selection.clear();
                                ps.selection.insert(next_id);
                            }
                        }
                    } else if !all_ids.is_empty() {
                        // No selection, select first widget
                        ps.selection.insert(all_ids[0]);
                    }
                }

                // Enter - Confirm selection (already selected, but provides feedback)
                if i.key_pressed(egui::Key::Enter) {
                    // Selection already exists, this could be used for editing later
                }

                // Escape - Clear selection
                if i.key_pressed(egui::Key::Escape) {
                    ps.selection.clear();
                }
            });
        }

        // Track if any reordering happened
        let mut reorder_action: Option<(Uuid, Uuid)> = None;

        draw_hierarchy_node(ui, ps.root_node.as_ref(), &mut ps.selection, &mut reorder_action);

        // Store pending reorder operation for processing in app update (with undo)
        if let Some((source_id, target_id)) = reorder_action {
            ps.pending_reorder = Some((source_id, target_id));
        }
    }

    fn render_inspector(&mut self, ui: &mut Ui) {
        // RIGHT: Property editor

        if let Some(id) = self.project_state.selection.iter().next().cloned() {
            let known_vars: Vec<String> = self.project_state.variables.keys().cloned().collect();

            if let Some(node) = self.project_state.find_node_mut(id) {
                node.inspect(ui, &known_vars);

                ui.separator();

                // Add Delete button
                ui.horizontal(|ui| {
                    if ui.button("🗑 Delete Widget").clicked() {
                        if self.project_state.delete_widget(id) {
                            self.project_state.selection.clear();
                        }
                    }
                });

                return;
            }
        }

        ui.label("No widget selected.");
    }

    fn render_output(&mut self, ui: &mut Ui) {
        ui.heading("Compilation Output");
        ui.separator();

        // Project name editor
        ui.horizontal(|ui| {
            ui.label("Project Name:");
            ui.text_edit_singleline(&mut self.project_state.project_name);
        });

        ui.separator();

        if ui.button("Generate Code (stdout)").clicked() {
            // Print to stdout for quick debugging
            let code = Compiler::generate_app_rs(&self.project_state);
            println!("--- Generated app.rs ---\n{}", code);
            println!("------------------------");
        }

        ui.separator();

        if ui.button("📁 Export Project").clicked() {
            // Pick a directory to export the project
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                // Create src subdirectory
                let src_dir = folder.join("src");
                if let Err(e) = std::fs::create_dir_all(&src_dir) {
                    eprintln!("Failed to create src directory: {}", e);
                    return;
                }

                // Write Cargo.toml
                let cargo_toml_path = folder.join("Cargo.toml");
                let cargo_toml = Compiler::generate_cargo_toml(&self.project_state.project_name);
                if let Err(e) = std::fs::write(&cargo_toml_path, cargo_toml) {
                    eprintln!("Failed to write Cargo.toml: {}", e);
                    return;
                }

                // Write src/main.rs
                let main_rs_path = src_dir.join("main.rs");
                let main_rs = Compiler::generate_main_rs();
                if let Err(e) = std::fs::write(&main_rs_path, main_rs) {
                    eprintln!("Failed to write main.rs: {}", e);
                    return;
                }

                // Write src/app.rs
                let app_rs_path = src_dir.join("app.rs");
                let app_rs = Compiler::generate_app_rs(&self.project_state);
                if let Err(e) = std::fs::write(&app_rs_path, app_rs) {
                    eprintln!("Failed to write app.rs: {}", e);
                    return;
                }

                println!("✓ Project '{}' exported successfully to: {}", self.project_state.project_name, folder.display());
            }
        }

        ui.separator();
        ui.label("Export your project to build and run with cargo.");
    }

    fn render_variables(&mut self, ui: &mut Ui) {
        ui.heading("Application State");
        ui.separator();

        // 1. Add New Variable
        ui.horizontal(|ui| {
            if ui.button("+ Add Variable").clicked() {
                let name = format!("var_{}", self.project_state.variables.len());
                self.project_state.variables.insert(
                    name.clone(),
                    Variable {
                        name,
                        v_type: VariableType::String,
                        value: "".to_string(),
                    },
                );
            }
        });
        ui.separator();

        // 2. List Variables
        let mut keys: Vec<String> = self.project_state.variables.keys().cloned().collect();
        keys.sort(); // Stable order

        let mut to_remove = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for key in keys {
                ui.group(|ui| {
                    if let Some(var) = self.project_state.variables.get_mut(&key) {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut var.name);
                            if ui.button("🗑").clicked() {
                                to_remove = Some(key.clone());
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt(format!("type_{}", key))
                                .selected_text(format!("{}", var.v_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut var.v_type,
                                        VariableType::String,
                                        "String",
                                    );
                                    ui.selectable_value(
                                        &mut var.v_type,
                                        VariableType::Integer,
                                        "Integer",
                                    );
                                    ui.selectable_value(
                                        &mut var.v_type,
                                        VariableType::Boolean,
                                        "Boolean",
                                    );
                                    ui.selectable_value(
                                        &mut var.v_type,
                                        VariableType::Float,
                                        "Float",
                                    );
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut var.value);
                        });
                    }
                });
            }
        });

        if let Some(key) = to_remove {
            self.project_state.variables.remove(&key);
        }
    }

    fn render_code_preview(&mut self, ui: &mut Ui) {
        ui.heading("Code Preview");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.label("Cargo.toml");
                let cargo_toml = Compiler::generate_cargo_toml(&self.project_state.project_name);
                ui.code(&cargo_toml);
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("src/main.rs");
                let main_rs = Compiler::generate_main_rs();
                ui.code(&main_rs);
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("src/app.rs");
                let app_rs = Compiler::generate_app_rs(&self.project_state);
                ui.code(&app_rs);
            });
        });
    }
}

fn draw_hierarchy_node(
    ui: &mut Ui,
    node: &dyn WidgetNode,
    selection: &mut HashSet<Uuid>,
    reorder_action: &mut Option<(Uuid, Uuid)>,
) {
    let id = node.id();
    let is_selected = selection.contains(&id);

    let children = node.children();
    let has_children = children.map_or(false, |c| !c.is_empty());

    if has_children {
        let id_source = id;
        let drag_id = egui::Id::new("hierarchy_drag").with(id);

        // Wrap the collapsing header in a drag source
        let header_response = ui.dnd_drag_source(drag_id, id, |ui| {
            let color = if is_selected {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().text_color()
            };
            let title = egui::RichText::new(node.name()).color(color);

            let header = egui::CollapsingHeader::new(title)
                .id_salt(id_source)
                .default_open(true);

            header.show(ui, |ui| {
                if let Some(children) = children {
                    for child in children {
                        draw_hierarchy_node(ui, child.as_ref(), selection, reorder_action);
                    }
                }
            })
        });

        // Drop zone for containers
        let (drop_inner, dropped_payload) = ui.dnd_drop_zone::<Uuid, ()>(egui::Frame::NONE, |_ui| {});

        if drop_inner.response.dnd_hover_payload::<Uuid>().is_some() {
            ui.painter().rect_filled(
                header_response.response.rect,
                0.0,
                ui.visuals().selection.bg_fill.gamma_multiply(0.5),
            );
        }

        if let Some(dragged_id) = dropped_payload {
            if *dragged_id != id {
                *reorder_action = Some((*dragged_id, id));
            }
        }

        if header_response.response.clicked() {
            selection.clear();
            selection.insert(id);
        }
    } else {
        // Leaf widget - add drag source and drop target
        let drag_id = egui::Id::new("hierarchy_drag").with(id);

        ui.horizontal(|ui| {
            // Drag source
            let response = ui.dnd_drag_source(drag_id, id, |ui| {
                ui.selectable_label(is_selected, node.name())
            }).response;

            // Check for drop - egui's dnd_drop_zone returns (InnerResponse, Option<payload>)
            // The second element is the released payload
            let (drop_inner, dropped_payload) = ui.dnd_drop_zone::<Uuid, ()>(egui::Frame::NONE, |_ui| {
                // Empty drop zone visualization
            });

            // Visual feedback on hover
            if drop_inner.response.dnd_hover_payload::<Uuid>().is_some() {
                ui.painter().rect_filled(
                    response.rect,
                    0.0,
                    ui.visuals().selection.bg_fill.gamma_multiply(0.5),
                );
            }

            // Handle drop
            if let Some(dragged_id) = dropped_payload {
                // Dropped! Record the reorder action
                if *dragged_id != id {
                    *reorder_action = Some((*dragged_id, id));
                }
            }

            if response.clicked() {
                selection.clear();
                selection.insert(id);
            }
        });
    }
}

// Helper to set up the default "Qt Designer" layout
pub fn default_layout() -> egui_dock::DockState<AetherTab> {
    let mut dock_state = egui_dock::DockState::new(vec![AetherTab::Canvas]);

    let tree = dock_state.main_surface_mut();

    // 1. Split Left for Palette
    let [_left, right] =
        tree.split_left(egui_dock::NodeIndex::root(), 0.2, vec![AetherTab::Palette]);

    // 2. Split Right for Inspector & Hierarchy
    let [_center, right_panel] = tree.split_right(right, 0.75, vec![AetherTab::Hierarchy]);

    // 3. Add Inspector to the same right panel (tabbed or split)
    // Let's split the right panel vertically: Top = Hierarchy, Bottom = Inspector
    tree.split_below(right_panel, 0.5, vec![AetherTab::Inspector]);

    // 4. Split Left (Palette) to add Variables below it
    tree.split_below(_left, 0.6, vec![AetherTab::Variables]);

    // 5. Split Bottom of Canvas (center) for Output and CodePreview (tabbed together)
    // We need to find the node containing the Canvas again since indices change.
    // For simplicity, we can just split the root's first child's second child...
    // Or easier: split the "right" node (which contains canvas) from step 1?
    // Actually, 'right' in step 1 was the center area.
    // But 'right' was split in step 2. The left part of that split is the new center (Canvas).
    tree.split_below(_center, 0.8, vec![AetherTab::Output, AetherTab::CodePreview]);

    dock_state
}

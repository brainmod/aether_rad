use crate::compiler::Compiler;
use crate::model::{ProjectState, Variable, VariableType, WidgetNode};
use crate::theme::{self, AetherColors, WidgetIcons};
use egui::{Ui, WidgetText, RichText, Color32, CornerRadius};
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
        let (icon, name) = match tab {
            AetherTab::Canvas => ("🎨", "Canvas"),
            AetherTab::Palette => ("🧰", "Palette"),
            AetherTab::Hierarchy => ("🌳", "Hierarchy"),
            AetherTab::Inspector => ("🔧", "Inspector"),
            AetherTab::Output => ("📤", "Output"),
            AetherTab::Variables => ("📊", "Variables"),
            AetherTab::CodePreview => ("📝", "Code"),
        };
        format!("{} {}", icon, name).into()
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
        // CENTER: The main visual editor with styled frame
        egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 35))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                // Canvas header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📐 Design Canvas").size(12.0).color(AetherColors::MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("Root: {}", self.project_state.root_layout_type()))
                            .size(11.0)
                            .color(AetherColors::MUTED));
                    });
                });
                ui.add_space(8.0);

                // Canvas content area
                egui::Frame::new()
                    .fill(Color32::from_rgb(40, 40, 48))
                    .inner_margin(egui::Margin::same(12))
                    .corner_radius(CornerRadius::same(8))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 65)))
                    .show(ui, |ui| {
                        self.project_state
                            .root_node
                            .render_editor(ui, &mut self.project_state.selection);
                    });
            });
    }

    fn render_palette(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("Widget Palette"));
        ui.add_space(4.0);
        ui.label(RichText::new("Drag widgets to the canvas").size(11.0).color(AetherColors::MUTED));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Layout widgets
            render_widget_category(ui, "Layouts", &[
                "Vertical Layout",
                "Horizontal Layout",
                "Grid Layout",
            ], AetherColors::LAYOUT_COLOR);

            ui.add_space(8.0);

            // Input widgets
            render_widget_category(ui, "Inputs", &[
                "Button",
                "Checkbox",
                "Slider",
                "Text Edit",
                "ComboBox",
            ], AetherColors::INPUT_COLOR);

            ui.add_space(8.0);

            // Display widgets
            render_widget_category(ui, "Display", &[
                "Label",
                "Progress Bar",
                "Image",
            ], AetherColors::DISPLAY_COLOR);
        });
    }

    fn render_hierarchy(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("Widget Tree"));
        ui.add_space(4.0);

        let ps = &mut self.project_state;

        // Keyboard navigation hint
        ui.label(RichText::new("↑↓ Navigate • Esc Clear").size(10.0).color(AetherColors::MUTED));
        ui.add_space(8.0);

        // Keyboard navigation for hierarchy
        if ui.ui_contains_pointer() {
            ui.input(|i| {
                let all_ids = ps.get_all_widget_ids();
                let current_selected = ps.selection.iter().next().cloned();

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
                        ps.selection.insert(all_ids[0]);
                    }
                }

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
                        ps.selection.insert(all_ids[0]);
                    }
                }

                if i.key_pressed(egui::Key::Escape) {
                    ps.selection.clear();
                }
            });
        }

        // Tree view with styled frame
        theme::section_frame(ui).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                draw_hierarchy_node_styled(ui, ps.root_node.as_ref(), &mut ps.selection, 0);
            });
        });
    }

    fn render_inspector(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);

        if let Some(id) = self.project_state.selection.iter().next().cloned() {
            let known_vars: Vec<String> = self.project_state.variables.keys().cloned().collect();
            let is_root = id == self.project_state.root_node.id();

            // Get widget name for header
            let widget_name = if let Some(node) = self.project_state.find_node_mut(id) {
                node.name().to_string()
            } else {
                "Unknown".to_string()
            };

            // Header with icon
            let icon = WidgetIcons::get(&widget_name);
            ui.label(theme::heading(&format!("{} {} Properties", icon, widget_name)));
            ui.add_space(4.0);

            // If root layout is selected, show layout type switcher
            if is_root {
                theme::section_frame(ui).show(ui, |ui| {
                    ui.label(theme::subheading("Root Layout Type"));
                    ui.add_space(4.0);

                    let current_type = self.project_state.root_layout_type();
                    let mut selected_type = current_type.to_string();

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut selected_type, "Vertical Layout".to_string(), "⬇ Vertical");
                        ui.selectable_value(&mut selected_type, "Horizontal Layout".to_string(), "➡ Horizontal");
                        ui.selectable_value(&mut selected_type, "Grid Layout".to_string(), "⊞ Grid");
                    });

                    if selected_type != current_type {
                        self.project_state.set_root_layout_type(&selected_type);
                    }
                });
                ui.add_space(8.0);
            }

            // Widget properties
            if let Some(node) = self.project_state.find_node_mut(id) {
                theme::section_frame(ui).show(ui, |ui| {
                    node.inspect(ui, &known_vars);
                });

                // Widget actions (not for root)
                if !is_root {
                    ui.add_space(8.0);
                    theme::section_frame(ui).show(ui, |ui| {
                        ui.label(theme::subheading("Actions"));
                        ui.add_space(6.0);

                        // Reorder buttons
                        ui.horizontal(|ui| {
                            if ui.button("⬆ Move Up").clicked() {
                                self.project_state.move_widget_up(id);
                            }
                            if ui.button("⬇ Move Down").clicked() {
                                self.project_state.move_widget_down(id);
                            }
                        });

                        ui.add_space(8.0);

                        // Delete button with warning color
                        if ui.add(egui::Button::new(
                            RichText::new("🗑 Delete Widget").color(AetherColors::ERROR)
                        )).clicked() {
                            if self.project_state.delete_widget(id) {
                                self.project_state.selection.clear();
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
        theme::section_frame(ui).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("👆").size(32.0));
                ui.add_space(8.0);
                ui.label(RichText::new("Select a widget").color(AetherColors::MUTED));
                ui.label(RichText::new("to view its properties").size(11.0).color(AetherColors::MUTED));
                ui.add_space(20.0);
            });
        });
    }

    fn render_output(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("Project Output"));
        ui.add_space(8.0);

        // Project settings
        theme::section_frame(ui).show(ui, |ui| {
            ui.label(theme::subheading("Project Settings"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(egui::TextEdit::singleline(&mut self.project_state.project_name)
                    .desired_width(150.0));
            });
        });

        ui.add_space(8.0);

        // Export actions
        theme::section_frame(ui).show(ui, |ui| {
            ui.label(theme::subheading("Export"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(
                    RichText::new("📁 Export Project").color(AetherColors::ACCENT_LIGHT)
                )).clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        let src_dir = folder.join("src");
                        if let Err(e) = std::fs::create_dir_all(&src_dir) {
                            eprintln!("Failed to create src directory: {}", e);
                            return;
                        }

                        let cargo_toml_path = folder.join("Cargo.toml");
                        let cargo_toml = Compiler::generate_cargo_toml(&self.project_state.project_name);
                        if let Err(e) = std::fs::write(&cargo_toml_path, cargo_toml) {
                            eprintln!("Failed to write Cargo.toml: {}", e);
                            return;
                        }

                        let main_rs_path = src_dir.join("main.rs");
                        let main_rs = Compiler::generate_main_rs();
                        if let Err(e) = std::fs::write(&main_rs_path, main_rs) {
                            eprintln!("Failed to write main.rs: {}", e);
                            return;
                        }

                        let app_rs_path = src_dir.join("app.rs");
                        let app_rs = Compiler::generate_app_rs(&self.project_state);
                        if let Err(e) = std::fs::write(&app_rs_path, app_rs) {
                            eprintln!("Failed to write app.rs: {}", e);
                            return;
                        }

                        println!("✓ Project '{}' exported to: {}", self.project_state.project_name, folder.display());
                    }
                }

                if ui.button("🖨 Print to Console").clicked() {
                    let code = Compiler::generate_app_rs(&self.project_state);
                    println!("--- Generated app.rs ---\n{}\n------------------------", code);
                }
            });

            ui.add_space(4.0);
            ui.label(RichText::new("Export creates a complete Cargo project")
                .size(11.0).color(AetherColors::MUTED));
        });
    }

    fn render_variables(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("State Variables"));
        ui.add_space(4.0);
        ui.label(RichText::new("Define app state for bindings").size(11.0).color(AetherColors::MUTED));
        ui.add_space(8.0);

        // Add variable button
        if ui.add(egui::Button::new(
            RichText::new("+ Add Variable").color(AetherColors::SUCCESS)
        )).clicked() {
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

        ui.add_space(8.0);

        // Variable list
        let mut keys: Vec<String> = self.project_state.variables.keys().cloned().collect();
        keys.sort();

        let mut to_remove = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for key in keys {
                theme::section_frame(ui).show(ui, |ui| {
                    if let Some(var) = self.project_state.variables.get_mut(&key) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&var.name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(
                                    RichText::new("🗑").color(AetherColors::ERROR)
                                ).small()).clicked() {
                                    to_remove = Some(key.clone());
                                }
                            });
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Type:").size(11.0).color(AetherColors::MUTED));
                            egui::ComboBox::from_id_salt(format!("type_{}", key))
                                .selected_text(format!("{}", var.v_type))
                                .width(80.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut var.v_type, VariableType::String, "String");
                                    ui.selectable_value(&mut var.v_type, VariableType::Integer, "Integer");
                                    ui.selectable_value(&mut var.v_type, VariableType::Boolean, "Boolean");
                                    ui.selectable_value(&mut var.v_type, VariableType::Float, "Float");
                                });

                            ui.label(RichText::new("Value:").size(11.0).color(AetherColors::MUTED));
                            ui.add(egui::TextEdit::singleline(&mut var.value).desired_width(80.0));
                        });
                    }
                });
                ui.add_space(4.0);
            }
        });

        if let Some(key) = to_remove {
            self.project_state.variables.remove(&key);
        }
    }

    fn render_code_preview(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("Generated Code"));
        ui.add_space(4.0);
        ui.label(RichText::new("Live preview of output").size(11.0).color(AetherColors::MUTED));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Cargo.toml
            render_code_section(ui, "📦 Cargo.toml", &Compiler::generate_cargo_toml(&self.project_state.project_name));

            ui.add_space(8.0);

            // main.rs
            render_code_section(ui, "🚀 src/main.rs", &Compiler::generate_main_rs());

            ui.add_space(8.0);

            // app.rs
            render_code_section(ui, "⚙️ src/app.rs", &Compiler::generate_app_rs(&self.project_state));
        });
    }
}

/// Render a categorized widget section in the palette
fn render_widget_category(ui: &mut Ui, category: &str, widgets: &[&str], accent_color: Color32) {
    let header = egui::CollapsingHeader::new(
        RichText::new(format!("{} {}", WidgetIcons::get_category_icon(category), category))
            .size(13.0)
            .color(accent_color)
    )
    .default_open(true);

    header.show(ui, |ui| {
        for widget_type in widgets {
            let icon = WidgetIcons::get(widget_type);
            let id = egui::Id::new("palette").with(*widget_type);

            ui.dnd_drag_source(id, widget_type.to_string(), |ui| {
                let response = ui.add(
                    egui::Button::new(RichText::new(format!("{} {}", icon, widget_type)))
                        .min_size(egui::vec2(ui.available_width() - 8.0, 28.0))
                );

                // Show drag hint on hover
                response.on_hover_text("Drag to canvas to add");
            });
            ui.add_space(2.0);
        }
    });
}

/// Render a code section with header
fn render_code_section(ui: &mut Ui, title: &str, code: &str) {
    theme::section_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).size(12.0).strong().color(AetherColors::ACCENT_LIGHT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{} lines", code.lines().count()))
                    .size(10.0).color(AetherColors::MUTED));
            });
        });
        ui.add_space(6.0);

        // Code display with monospace font
        egui::Frame::new()
            .fill(Color32::from_rgb(25, 25, 30))
            .inner_margin(egui::Margin::same(8))
            .corner_radius(CornerRadius::same(4))
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(code).monospace().size(11.0)).wrap());
            });
    });
}

/// Styled hierarchy node rendering with icons and depth indication
fn draw_hierarchy_node_styled(
    ui: &mut Ui,
    node: &dyn WidgetNode,
    selection: &mut HashSet<Uuid>,
    depth: usize,
) {
    let id = node.id();
    let is_selected = selection.contains(&id);
    let icon = WidgetIcons::get(node.name());
    let category_color = theme::widget_category_color(node.name());

    let children = node.children();
    let has_children = children.map_or(false, |c| !c.is_empty());

    // Indent based on depth
    let indent = depth as f32 * 12.0;

    if has_children {
        let display_text = format!("{} {}", icon, node.name());
        let text_color = if is_selected {
            AetherColors::ACCENT_LIGHT
        } else {
            category_color
        };

        ui.horizontal(|ui| {
            ui.add_space(indent);

            let header = egui::CollapsingHeader::new(
                RichText::new(&display_text).color(text_color).strong()
            )
            .id_salt(id)
            .default_open(true);

            let response = header.show(ui, |ui| {
                if let Some(children) = children {
                    for child in children {
                        draw_hierarchy_node_styled(ui, child.as_ref(), selection, depth + 1);
                    }
                }
            });

            if response.header_response.clicked() {
                selection.clear();
                selection.insert(id);
            }

            // Selection indicator
            if is_selected {
                let rect = response.header_response.rect;
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    4.0,
                    egui::Stroke::new(2.0, AetherColors::ACCENT),
                    egui::StrokeKind::Outside,
                );
            }
        });
    } else {
        ui.horizontal(|ui| {
            ui.add_space(indent + 16.0); // Extra indent for leaf nodes

            let display_text = format!("{} {}", icon, node.name());
            let text_color = if is_selected {
                AetherColors::ACCENT_LIGHT
            } else {
                category_color
            };

            let response = ui.selectable_label(
                is_selected,
                RichText::new(&display_text).color(text_color)
            );

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
        tree.split_left(egui_dock::NodeIndex::root(), 0.18, vec![AetherTab::Palette]);

    // 2. Split Right for Inspector & Hierarchy
    let [_center, right_panel] = tree.split_right(right, 0.78, vec![AetherTab::Hierarchy]);

    // 3. Add Inspector to the same right panel (tabbed or split)
    tree.split_below(right_panel, 0.5, vec![AetherTab::Inspector]);

    // 4. Split Left (Palette) to add Variables below it
    tree.split_below(_left, 0.6, vec![AetherTab::Variables]);

    // 5. Split Bottom of Canvas (center) for Output and CodePreview (tabbed together)
    tree.split_below(_center, 0.75, vec![AetherTab::Output, AetherTab::CodePreview]);

    dock_state
}

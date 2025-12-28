use crate::compiler::Compiler;
use crate::model::{ProjectState, Variable, VariableType, WidgetNode};
use crate::theme::{self, AetherColors, ThemeMode};
use crate::validator::{CodeValidator, ValidationStatus};
use egui::{Color32, CornerRadius, RichText, Ui, WidgetText};
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
    Assets,
    CodePreview,
}

// The "Viewer" handles the actual rendering of each tab.
// It holds a mutable reference to the ProjectState so tabs can modify it.
pub struct AetherTabViewer<'a> {
    pub project_state: &'a mut ProjectState,
    pub validation_status: &'a mut ValidationStatus,
    pub theme_mode: &'a mut ThemeMode,
    pub canvas_zoom: &'a mut f32,
    pub canvas_pan: &'a mut egui::Vec2,
}

impl<'a> TabViewer for AetherTabViewer<'a> {
    type Tab = AetherTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        let name = match tab {
            AetherTab::Canvas => "Canvas",
            AetherTab::Palette => "Palette",
            AetherTab::Hierarchy => "Hierarchy",
            AetherTab::Inspector => "Inspector",
            AetherTab::Output => "Output",
            AetherTab::Variables => "Variables",
            AetherTab::Assets => "Assets",
            AetherTab::CodePreview => "Code",
        };
        name.into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            AetherTab::Canvas => self.render_canvas(ui),
            AetherTab::Palette => self.render_palette(ui),
            AetherTab::Hierarchy => self.render_hierarchy(ui),
            AetherTab::Inspector => self.render_inspector(ui),
            AetherTab::Output => self.render_output(ui),
            AetherTab::Variables => self.render_variables(ui),
            AetherTab::Assets => self.render_assets(ui),
            AetherTab::CodePreview => self.render_code_preview(ui),
        }
    }
}

impl<'a> AetherTabViewer<'a> {
    fn render_canvas(&mut self, ui: &mut Ui) {
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
                        *self.canvas_zoom = (*self.canvas_zoom - 0.1).max(0.1);
                    }
                    ui.add(
                        egui::Slider::new(self.canvas_zoom, 0.1..=3.0)
                            .fixed_decimals(0)
                            .text("%")
                            .show_value(true),
                    );
                    if ui.button("+").clicked() {
                        *self.canvas_zoom = (*self.canvas_zoom + 0.1).min(3.0);
                    }
                    if ui.button("100%").clicked() {
                        *self.canvas_zoom = 1.0;
                    }
                    if ui.button("Fit").clicked() {
                        *self.canvas_zoom = 1.0;
                        *self.canvas_pan = egui::Vec2::ZERO;
                    }

                    // Show current zoom percentage
                    ui.label(
                        RichText::new(format!("{}%", (*self.canvas_zoom * 100.0) as i32))
                            .size(11.0)
                            .color(muted_color),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Root: {}",
                                self.project_state.root_layout_type()
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
                        let zoom = *self.canvas_zoom;

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
                            .scroll_offset(*self.canvas_pan)
                            .show(ui, |ui| {
                                // Apply zoom by scaling the UI
                                ui.style_mut().spacing.item_spacing *= zoom;
                                ui.style_mut().spacing.button_padding *= zoom;
                                ui.style_mut().spacing.indent *= zoom;

                                // Scale text sizes for widgets
                                let original_text_style = ui.style().text_styles.clone();
                                for (_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                                    font_id.size *= zoom;
                                }

                                // Add some padding at the scaled level
                                ui.add_space(8.0 * zoom);

                                // Render the widget tree
                                self.project_state
                                    .root_node
                                    .render_editor(ui, &mut self.project_state.selection);

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
                        *self.canvas_zoom = (*self.canvas_zoom + zoom_delta).clamp(0.1, 3.0);
                    }
                }

                // Handle middle-mouse drag for panning
                let pan_response = ui.interact(canvas_rect, egui::Id::new("canvas_pan_drag"), egui::Sense::drag());
                if pan_response.dragged_by(egui::PointerButton::Middle) {
                    *self.canvas_pan -= pan_response.drag_delta();
                }
            });
    }

    fn render_palette(&mut self, ui: &mut Ui) {
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

    fn render_hierarchy(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("Widget Tree"));
        ui.add_space(4.0);

        // Keyboard navigation hint
        ui.label(
            RichText::new("↑ ↓ Navigate • Drag to reorder")
                .size(10.0)
                .color(theme::muted_color(ui.ctx())),
        );
        ui.add_space(8.0);

        // Keyboard navigation for hierarchy
        if ui.ui_contains_pointer() {
            let all_ids = self.project_state.get_all_widget_ids();
            let current_selected = self.project_state.selection.iter().next().cloned();

            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowUp) {
                    if let Some(current) = current_selected {
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx > 0 {
                                self.project_state.selection.clear();
                                self.project_state.selection.insert(all_ids[current_idx - 1]);
                            }
                        }
                    } else if !all_ids.is_empty() {
                        self.project_state.selection.insert(all_ids[0]);
                    }
                }

                if i.key_pressed(egui::Key::ArrowDown) {
                    if let Some(current) = current_selected {
                        if let Some(current_idx) = all_ids.iter().position(|id| *id == current) {
                            if current_idx < all_ids.len() - 1 {
                                self.project_state.selection.clear();
                                self.project_state.selection.insert(all_ids[current_idx + 1]);
                            }
                        }
                    } else if !all_ids.is_empty() {
                        self.project_state.selection.insert(all_ids[0]);
                    }
                }

                if i.key_pressed(egui::Key::Escape) {
                    self.project_state.selection.clear();
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
                    self.project_state.root_node.as_ref(),
                    &mut self.project_state.selection,
                    0,
                    &mut pending_drop,
                );
            });
        });

        // Handle any pending drop operations
        if let Some((source_id, target_id, position)) = pending_drop {
            // Get the currently selected widget as the drag source if source_id is nil
            let actual_source_id = if source_id == Uuid::nil() {
                self.project_state.selection.iter().next().cloned()
            } else {
                Some(source_id)
            };

            if let Some(src_id) = actual_source_id {
                match position {
                    DropPosition::Before => {
                        self.project_state.move_widget_before(src_id, target_id);
                    }
                    DropPosition::After => {
                        self.project_state.move_widget_after(src_id, target_id);
                    }
                    DropPosition::Into => {
                        // Move into container at end
                        self.project_state.reparent_widget(src_id, target_id, usize::MAX);
                    }
                }
            }
        }
    }

    fn render_inspector(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);

        // Check if multiple widgets are selected
        let selection_count = self.project_state.selection.len();

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
                        for id in self.project_state.selection.clone() {
                            self.project_state.delete_widget(id);
                        }
                    }
                });
            });
        } else if let Some(id) = self.project_state.selection.iter().next().cloned() {
            let known_vars: Vec<String> = self.project_state.variables.keys().cloned().collect();
            // Build (name, filename) pairs for asset selection
            let known_assets: Vec<(String, String)> = self.project_state.assets.assets.values()
                .map(|asset| {
                    let filename = asset.path.file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&asset.name)
                        .to_string();
                    (asset.name.clone(), filename)
                })
                .collect();
            let is_root = id == self.project_state.root_node.id();

            // Get widget name for header
            let widget_name = if let Some(node) = self.project_state.find_node_mut(id) {
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

                    let current_type = self.project_state.root_layout_type();
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
                        self.project_state.set_root_layout_type(&selected_type);
                    }
                });
                ui.add_space(8.0);
            }

            // Widget properties
            if let Some(node) = self.project_state.find_node_mut(id) {
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
                                self.project_state.move_widget_up(id);
                            }
                            if ui.button("⬇ Move Down").clicked() {
                                self.project_state.move_widget_down(id);
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

    fn render_output(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);

        // Header with theme toggle
        ui.horizontal(|ui| {
            ui.label(theme::heading("Project Output"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_icon = match *self.theme_mode {
                    ThemeMode::Dark => "☀️",  // Sun when in dark mode (to switch to light)
                    ThemeMode::Light => "🌙", // Moon when in light mode (to switch to dark)
                };

                if ui
                    .button(theme_icon)
                    .on_hover_text("Toggle theme")
                    .clicked()
                {
                    self.theme_mode.toggle();
                }
            });
        });
        ui.add_space(8.0);

        // New Project templates
        theme::section_frame(ui.ctx()).show(ui, |ui| {
            ui.label(theme::subheading("New Project"));
            ui.add_space(6.0);

            ui.label(
                RichText::new("Create from template:")
                    .size(11.0)
                    .color(theme::muted_color(ui.ctx())),
            );
            ui.add_space(4.0);

            if ui.button("Empty Project").clicked() {
                *self.project_state = crate::model::ProjectState::empty();
            }

            if ui.button("Counter App").clicked() {
                *self.project_state = crate::model::ProjectState::template_counter_app();
            }

            if ui.button("Contact Form").clicked() {
                *self.project_state = crate::model::ProjectState::template_form();
            }

            if ui.button("Dashboard").clicked() {
                *self.project_state = crate::model::ProjectState::template_dashboard();
            }
        });

        ui.add_space(8.0);

        // Project settings
        theme::section_frame(ui.ctx()).show(ui, |ui| {
            ui.label(theme::subheading("Project Settings"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.project_state.project_name)
                        .desired_width(150.0),
                );
            });
        });

        ui.add_space(8.0);

        // Validation section
        theme::section_frame(ui.ctx()).show(ui, |ui| {
            ui.label(theme::subheading("Code Validation"));
            ui.add_space(6.0);

            let is_checking = self.validation_status.is_checking();

            if ui
                .add_enabled(
                    !is_checking,
                    egui::Button::new(
                        RichText::new("✓ Check Code").color(theme::accent_light_color(ui.ctx())),
                    ),
                )
                .clicked()
            {
                *self.validation_status = ValidationStatus::Checking;
                let project_state_clone = self.project_state.clone();
                match CodeValidator::validate(&project_state_clone) {
                    Ok(_) => {
                        *self.validation_status = ValidationStatus::Success;
                    }
                    Err(err) => {
                        *self.validation_status = ValidationStatus::Failed(err);
                    }
                }
            }

            ui.add_space(8.0);

            // Display validation status with progress animation
            ui.horizontal(|ui| {
                // Show spinner if checking
                if is_checking {
                    ui.add(egui::Spinner::new());
                }

                let status_text = self.validation_status.display_text();
                let status_color = if self.validation_status.is_success() {
                    theme::success_color(ui.ctx())
                } else if matches!(self.validation_status, ValidationStatus::Failed(_)) {
                    theme::error_color(ui.ctx())
                } else {
                    theme::muted_color(ui.ctx())
                };

                ui.label(RichText::new(status_text).size(11.0).color(status_color));
            });
        });

        ui.add_space(8.0);

        // Export actions
        theme::section_frame(ui.ctx()).show(ui, |ui| {
            ui.label(theme::subheading("Export"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(
                        RichText::new("⬇ Export Project").color(theme::accent_light_color(ui.ctx())),
                    ))
                    .clicked()
                {
                    if let Some(folder) = crate::io::pick_folder() {
                        let src_dir = folder.join("src");
                        if let Err(e) = std::fs::create_dir_all(&src_dir) {
                            eprintln!("Failed to create src directory: {}", e);
                            return;
                        }

                        let cargo_toml_path = folder.join("Cargo.toml");
                        let cargo_toml =
                            Compiler::generate_cargo_toml(&self.project_state.project_name);
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

                        // Copy assets to assets directory
                        if !self.project_state.assets.assets.is_empty() {
                            let assets_dir = folder.join("assets");
                            if let Err(e) = std::fs::create_dir_all(&assets_dir) {
                                eprintln!("Failed to create assets directory: {}", e);
                                return;
                            }

                            for asset in self.project_state.assets.assets.values() {
                                let source_path = &asset.path;
                                if source_path.exists() {
                                    if let Some(filename) = source_path.file_name() {
                                        let dest_path = assets_dir.join(filename);
                                        if let Err(e) = std::fs::copy(source_path, &dest_path) {
                                            eprintln!("Failed to copy asset '{}': {}", asset.name, e);
                                        } else {
                                            println!("  ✓ Copied asset: {}", asset.name);
                                        }
                                    }
                                } else {
                                    eprintln!("  ⚠ Asset file not found: {}", source_path.display());
                                }
                            }
                        }

                        println!(
                            "✓ Project '{}' exported to: {}",
                            self.project_state.project_name,
                            folder.display()
                        );
                    }
                }

                if ui.button("🖨 Print to Console").clicked() {
                    let code = Compiler::generate_app_rs(&self.project_state);
                    println!(
                        "--- Generated app.rs ---\n{}\n------------------------",
                        code
                    );
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("Export creates a complete Cargo project")
                    .size(11.0)
                    .color(theme::muted_color(ui.ctx())),
            );
        });
    }

    fn render_variables(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.label(theme::heading("State Variables"));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Define app state for bindings")
                .size(11.0)
                .color(theme::muted_color(ui.ctx())),
        );
        ui.add_space(8.0);

        // Add variable button
        if ui
            .add(egui::Button::new(
                RichText::new("+ Add Variable").color(theme::success_color(ui.ctx())),
            ))
            .clicked()
        {
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
                theme::section_frame(ui.ctx()).show(ui, |ui| {
                    if let Some(var) = self.project_state.variables.get_mut(&key) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&var.name).strong());
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
                                        to_remove = Some(key.clone());
                                    }
                                },
                            );
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Type:").size(11.0).color(theme::muted_color(ui.ctx())));
                            egui::ComboBox::from_id_salt(format!("type_{}", key))
                                .selected_text(format!("{}", var.v_type))
                                .width(80.0)
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

                            ui.label(
                                RichText::new("Value:")
                                    .size(11.0)
                                    .color(theme::muted_color(ui.ctx())),
                            );
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

    fn render_assets(&mut self, ui: &mut Ui) {
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
                    .unwrap_or_else(|| format!("asset_{}", self.project_state.assets.assets.len()));

                // Add asset to project
                self.project_state.assets.add_asset(
                    name,
                    crate::model::AssetType::Image,
                    path,
                );
            }
        }

        ui.add_space(8.0);

        // Asset list
        if self.project_state.assets.assets.is_empty() {
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

                for (name, asset) in &self.project_state.assets.assets {
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
                    self.project_state.assets.remove_asset(&name);
                }
            });
        }
    }

    fn render_code_preview(&mut self, ui: &mut Ui) {
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
                &Compiler::generate_cargo_toml(&self.project_state.project_name),
            );

            ui.add_space(8.0);

            // main.rs
            render_code_section(ui, "🚀 src/main.rs", &Compiler::generate_main_rs());

            ui.add_space(8.0);

            // app.rs
            render_code_section(
                ui,
                "⚙️ src/app.rs",
                &Compiler::generate_app_rs(&self.project_state),
            );
        });
    }
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
                                render_widget_preview(ui, widget_type, accent_color);
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

/// Render a preview of a widget for drag-and-drop visualization
fn render_widget_preview(ui: &mut Ui, widget_type: &str, accent_color: Color32) {
    ui.set_max_width(150.0);

    match widget_type {
        "Button" => {
            ui.button("Click Me");
        }
        "Label" => {
            ui.label("Label Text");
        }
        "Text Edit" => {
            let mut preview_text = "Enter text...".to_string();
            ui.add(egui::TextEdit::singleline(&mut preview_text).desired_width(120.0));
        }
        "Checkbox" => {
            let mut checked = true;
            ui.checkbox(&mut checked, "Checkbox");
        }
        "Slider" => {
            let mut value = 0.5f32;
            ui.add(egui::Slider::new(&mut value, 0.0..=1.0).show_value(false));
        }
        "Progress Bar" => {
            ui.add(egui::ProgressBar::new(0.6).desired_width(120.0));
        }
        "ComboBox" => {
            let mut selected = "Option 1".to_string();
            egui::ComboBox::from_id_salt("preview_combo")
                .selected_text(&selected)
                .width(100.0)
                .show_ui(ui, |_ui| {});
        }
        "Image" => {
            egui::Frame::new()
                .fill(Color32::from_gray(100))
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.label(RichText::new("🖼").size(24.0));
                });
        }
        "Vertical Layout" | "Horizontal Layout" | "Grid Layout" => {
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(RichText::new(theme::WidgetLabels::get(widget_type)).small());
                });
        }
        "Separator" => {
            ui.separator();
        }
        "Spinner" => {
            ui.add(egui::Spinner::new());
        }
        "Hyperlink" => {
            ui.hyperlink_to("Link", "");
        }
        "Color Picker" => {
            let mut color = [0.3f32, 0.6, 0.9];
            ui.color_edit_button_rgb(&mut color);
        }
        "Freeform Layout" => {
            egui::Frame::new()
                .fill(Color32::from_gray(40))
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    // Draw a small grid preview
                    let (_id, rect) = ui.allocate_space(egui::vec2(60.0, 40.0));
                    let painter = ui.painter();
                    for i in 0..4 {
                        let x = rect.left() + (i as f32) * 15.0;
                        painter.line_segment(
                            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                            egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                    }
                    for i in 0..3 {
                        let y = rect.top() + (i as f32) * 15.0;
                        painter.line_segment(
                            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                            egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                    }
                });
        }
        "Scroll Area" | "Tab Container" | "Window" => {
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(RichText::new(theme::WidgetLabels::get(widget_type)).small());
                });
        }
        _ => {
            ui.label(widget_type);
        }
    }
}

/// Render a code section with header and syntax highlighting
fn render_code_section(ui: &mut Ui, title: &str, code: &str) {
    use crate::syntax;

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

/// Payload for hierarchy drag-and-drop
#[derive(Clone)]
struct HierarchyDragPayload {
    widget_id: Uuid,
    widget_name: String,
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
        let before_drop_id = egui::Id::new("drop_before").with(id);
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

            // Make the header draggable
            ui.dnd_drag_source(drag_id, payload.clone(), |ui| {
                let header = egui::CollapsingHeader::new(
                    RichText::new(&display_text).color(text_color).strong(),
                )
                .id_salt(id)
                .default_open(true);

                let response = header.show(ui, |ui| {
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
                });

                // Handle multi-selection with Ctrl/Cmd support
                if response.header_response.clicked() {
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
                    let rect = response.header_response.rect;
                    ui.painter().rect_stroke(
                        rect.expand(2.0),
                        4.0,
                        egui::Stroke::new(2.0, AetherColors::ACCENT),
                        egui::StrokeKind::Outside,
                    );
                }
            });
        });
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

/// Position for drop operations
#[derive(Clone, Copy, Debug)]
enum DropPosition {
    Before,
    After,
    Into,
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

    // 4. Split Left (Palette) to add Variables and Assets below it
    let [variables_panel, _] = tree.split_below(_left, 0.6, vec![AetherTab::Variables]);
    tree.split_below(variables_panel, 0.5, vec![AetherTab::Assets]);

    // 5. Split Bottom of Canvas (center) for Output and CodePreview (tabbed together)
    tree.split_below(
        _center,
        0.75,
        vec![AetherTab::Output, AetherTab::CodePreview],
    );

    dock_state
}

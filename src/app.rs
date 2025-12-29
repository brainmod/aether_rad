use crate::compiler::Compiler;
use crate::model::ProjectState;
use crate::theme::{self, ThemeMode};
use crate::ui::{
    assets, canvas, code_preview, hierarchy, inspector, palette, variables, EditorContext,
};
use crate::validator::{CodeValidator, ValidationStatus};
use crate::widgets::{ButtonWidget, LabelWidget, VerticalLayout};
use eframe::App;
use egui::RichText;

pub struct AetherApp {
    // UI State for panels
    ui_state: UiState,

    // The data state of the user's project (SOM)
    project_state: ProjectState,

    // Undo/Redo history
    undo_stack: Vec<ProjectState>,
    redo_stack: Vec<ProjectState>,

    // Clipboard for copy/paste
    clipboard: Option<String>, // JSON representation of copied widget

    // Code validation status
    validation_status: ValidationStatus,

    // Theme mode (Light or Dark)
    theme_mode: ThemeMode,

    // Theme initialized flag
    theme_initialized: bool,

    // Canvas zoom level (1.0 = 100%)
    canvas_zoom: f32,

    // Canvas pan offset
    canvas_pan: egui::Vec2,
}

// UI State implementation
pub struct UiState {
    pub left_panel_expanded: bool,
    pub right_panel_expanded: bool,
    pub bottom_panel_expanded: bool, // Renamed to mean status bar visibility
    
    // Tab selection
    pub left_tab: LeftTab,
    pub right_bottom_tab: RightBottomTab, // New split for Inspector/Variables

    pub show_code_preview: bool,
    pub show_project_settings: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel_expanded: true,
            right_panel_expanded: true,
            bottom_panel_expanded: true, // Shows status bar by default
            left_tab: LeftTab::Palette,
            right_bottom_tab: RightBottomTab::Inspector,
            show_code_preview: false,
            show_project_settings: false,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum LeftTab {
    Palette,
    Assets,
}

#[derive(PartialEq, Clone, Copy)]
pub enum RightBottomTab {
    Inspector,
    Variables,
}

impl AetherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize with a demo project
        let mut root = VerticalLayout::default();
        root.children.push(Box::new(LabelWidget {
            text: "Welcome to Aether RAD".into(),
            ..Default::default()
        }));
        root.children.push(Box::new(ButtonWidget {
            text: "Click Me".into(),
            ..Default::default()
        }));
        root.children.push(Box::new(ButtonWidget {
            text: "Edit Me".into(),
            ..Default::default()
        }));

        Self {
            ui_state: UiState::default(),
            project_state: ProjectState::new(Box::new(root)),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            validation_status: ValidationStatus::NotRun,
            theme_mode: ThemeMode::Dark,
            theme_initialized: false,
            canvas_zoom: 1.0,
            canvas_pan: egui::Vec2::ZERO,
        }
    }

    /// Push the current state onto the undo stack before making a change
    fn push_undo(&mut self) {
        self.undo_stack.push(self.project_state.clone());
        // Clear redo stack when making a new change
        self.redo_stack.clear();

        // Limit undo stack size to prevent excessive memory usage
        const MAX_UNDO_STEPS: usize = 50;
        if self.undo_stack.len() > MAX_UNDO_STEPS {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last action
    fn undo(&mut self) {
        if let Some(previous_state) = self.undo_stack.pop() {
            // Push current state to redo stack
            self.redo_stack.push(self.project_state.clone());
            // Restore previous state
            self.project_state = previous_state;
        }
    }

    /// Redo the last undone action
    fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            // Push current state to undo stack
            self.undo_stack.push(self.project_state.clone());
            // Restore next state
            self.project_state = next_state;
        }
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Copy selected widget to clipboard
    fn copy_widget(&mut self) {
        if let Some(id) = self.project_state.selection.iter().next().cloned() {
            if let Some(node) = self.project_state.find_node_mut(id) {
                // Serialize the widget to JSON
                if let Ok(json) = serde_json::to_string(node) {
                    self.clipboard = Some(json);
                }
            }
        }
    }

    /// Paste widget from clipboard (with new UUID)
    fn paste_widget(&mut self) {
        if let Some(json) = &self.clipboard {
            // Deserialize the widget
            if let Ok(mut widget) = serde_json::from_str::<Box<dyn crate::model::WidgetNode>>(json)
            {
                self.push_undo();

                // Regenerate UUID to make it unique
                widget = regenerate_widget_ids(widget);

                // Try to add to root if it's a layout
                if let Some(children) = self.project_state.root_node.children_mut() {
                    children.push(widget);
                }
            }
        }
    }
}

/// Recursively regenerate all UUIDs in a widget tree
fn regenerate_widget_ids(
    widget: Box<dyn crate::model::WidgetNode>,
) -> Box<dyn crate::model::WidgetNode> {
    // Clone the widget and serialize/deserialize to get a fresh copy
    // Then we can modify it
    let cloned = widget.clone_box();

    // This is a simple approach: serialize and deserialize, which will generate new UUIDs
    // if we modify the JSON. For now, just return the cloned widget.
    // A more sophisticated approach would traverse and regenerate IDs explicitly.
    cloned
}

impl App for AetherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme on every frame (to support real-time toggling)
        theme::configure_aether_theme(ctx, self.theme_mode);

        if !self.theme_initialized {
            self.theme_initialized = true;
        }

        // Process pending reorder operation with undo support
        if let Some((source_id, target_id)) = self.project_state.pending_reorder.take() {
            self.push_undo();

            // Find the parent and apply reorder
            if let Some(parent) = self.project_state.find_parent_mut(source_id) {
                if let Some(children) = parent.children() {
                    if let Some(target_idx) = children.iter().position(|c| c.id() == target_id) {
                        self.project_state.reorder_widget(source_id, target_idx);
                    }
                }
            }
        }

        // Handle keyboard shortcuts
        ctx.input(|i| {
            // Copy: Ctrl+C (Cmd+C on Mac)
            if i.modifiers.command && i.key_pressed(egui::Key::C) && !i.modifiers.shift {
                self.copy_widget();
            }
            // Paste: Ctrl+V (Cmd+V on Mac)
            else if i.modifiers.command && i.key_pressed(egui::Key::V) {
                self.paste_widget();
            }
            // Undo: Ctrl+Z (Cmd+Z on Mac)
            else if i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
                self.undo();
            }
            // Redo: Ctrl+Shift+Z or Ctrl+Y (Cmd+Shift+Z or Cmd+Y on Mac)
            else if (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
                || (i.modifiers.command && i.key_pressed(egui::Key::Y))
            {
                self.redo();
            }
            // Delete widget
            else if i.key_pressed(egui::Key::Delete) {
                if let Some(id) = self.project_state.selection.iter().next().cloned() {
                    self.push_undo();
                    if self.project_state.delete_widget(id) {
                        self.project_state.selection.clear();
                    }
                }
            }
        });

        // --- TOP PANEL (Menu Bar) ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("New Project...", |ui| {
                        if ui.button("Empty Project").clicked() {
                            self.project_state = crate::model::ProjectState::empty();
                            ui.close();
                        }
                        if ui.button("Counter App").clicked() {
                            self.project_state = crate::model::ProjectState::template_counter_app();
                            ui.close();
                        }
                        if ui.button("Contact Form").clicked() {
                            self.project_state = crate::model::ProjectState::template_form();
                            ui.close();
                        }
                        if ui.button("Dashboard").clicked() {
                            self.project_state = crate::model::ProjectState::template_dashboard();
                            ui.close();
                        }
                    });
                    
                    ui.separator();

                    if ui.button("Save Project").clicked() {
                        self.push_undo();
                        if let Some(path) = crate::io::save_file("project.json") {
                            if let Ok(file) = std::fs::File::create(path) {
                                let _ = serde_json::to_writer_pretty(file, &self.project_state);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Load Project").clicked() {
                        self.push_undo();
                        if let Some(path) = crate::io::pick_file("Aether Project") {
                            if let Ok(file) = std::fs::File::open(path) {
                                let reader = std::io::BufReader::new(file);
                                if let Ok(state) = serde_json::from_reader(reader) {
                                    self.project_state = state;
                                } else {
                                    eprintln!("Failed to parse project file");
                                }
                            }
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Project Settings...").clicked() {
                        self.ui_state.show_project_settings = true;
                        ui.close();
                    }

                    ui.menu_button("Export...", |ui| {
                        if ui.button("Export to Folder").clicked() {
                            if let Some(folder) = crate::io::pick_folder() {
                                // Re-using export logic
                                let src_dir = folder.join("src");
                                let _ = std::fs::create_dir_all(&src_dir);
                                
                                let cargo_toml_path = folder.join("Cargo.toml");
                                let cargo_toml = Compiler::generate_cargo_toml(&self.project_state.project_name);
                                let _ = std::fs::write(&cargo_toml_path, cargo_toml);

                                let main_rs_path = src_dir.join("main.rs");
                                let main_rs = Compiler::generate_main_rs();
                                let _ = std::fs::write(&main_rs_path, main_rs);

                                let app_rs_path = src_dir.join("app.rs");
                                let app_rs = Compiler::generate_app_rs(&self.project_state);
                                let _ = std::fs::write(&app_rs_path, app_rs);
                                
                                // Copy assets...
                                if !self.project_state.assets.assets.is_empty() {
                                    let assets_dir = folder.join("assets");
                                    let _ = std::fs::create_dir_all(&assets_dir);
                                    for asset in self.project_state.assets.assets.values() {
                                        if let Some(name) = asset.path.file_name() {
                                            let _ = std::fs::copy(&asset.path, assets_dir.join(name));
                                        }
                                    }
                                }
                            }
                            ui.close();
                        }
                        if ui.button("Print App Code").clicked() {
                            println!("{}", Compiler::generate_app_rs(&self.project_state));
                            ui.close();
                        }
                    });
                });

                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(self.can_undo(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.undo();
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.can_redo(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.redo();
                        ui.close();
                    }

                    ui.separator();

                    let has_selection = !self.project_state.selection.is_empty();
                    if ui
                        .add_enabled(has_selection, egui::Button::new("Copy"))
                        .clicked()
                    {
                        self.copy_widget();
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.clipboard.is_some(), egui::Button::new("Paste"))
                        .clicked()
                    {
                        self.paste_widget();
                        ui.close();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.ui_state.left_panel_expanded, "Left Panel").clicked() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.ui_state.right_panel_expanded, "Right Panel").clicked() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.ui_state.bottom_panel_expanded, "Status Bar").clicked() {
                        ui.close();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.ui_state.show_code_preview, "Code Preview Window").clicked() {
                         ui.close();
                    }
                    ui.separator();
                    // Theme toggle inside View menu
                    let theme_name = match self.theme_mode {
                        ThemeMode::Dark => "Switch to Light Mode",
                        ThemeMode::Light => "Switch to Dark Mode",
                    };
                    if ui.button(theme_name).clicked() {
                        self.theme_mode.toggle();
                        ui.close();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Validate Code").clicked() {
                        let project_state_clone = self.project_state.clone();
                        match CodeValidator::validate(&project_state_clone) {
                            Ok(_) => {
                                self.validation_status = ValidationStatus::Success;
                            }
                            Err(err) => {
                                self.validation_status = ValidationStatus::Failed(err);
                            }
                        }
                        ui.close();
                    }
                });
            });
        });

        // Create the editor context AFTER top panel (avoids borrow conflict with push_undo)
        let mut editor_ctx = EditorContext {
            project_state: &mut self.project_state,
            validation_status: &mut self.validation_status,
            theme_mode: &mut self.theme_mode,
            canvas_zoom: &mut self.canvas_zoom,
            canvas_pan: &mut self.canvas_pan,
            clipboard: &mut self.clipboard,
        };

        // --- BOTTOM STATUS BAR ---
        if self.ui_state.bottom_panel_expanded {
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .min_height(24.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let is_checking = editor_ctx.validation_status.is_checking();
                        if is_checking {
                            ui.add(egui::Spinner::new().size(12.0));
                        }
                        
                        let status_text = editor_ctx.validation_status.display_text();
                        let status_color = if editor_ctx.validation_status.is_success() {
                            theme::success_color(ui.ctx())
                        } else if matches!(editor_ctx.validation_status, ValidationStatus::Failed(_)) {
                            theme::error_color(ui.ctx())
                        } else {
                            theme::muted_color(ui.ctx())
                        };
                        
                        ui.label(RichText::new(status_text).size(11.0).color(status_color));
                    });
                });
        }

        // --- LEFT PANEL ---
        if self.ui_state.left_panel_expanded {
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.ui_state.left_tab == LeftTab::Palette, "Palette").clicked() {
                            self.ui_state.left_tab = LeftTab::Palette;
                        }
                         if ui.selectable_label(self.ui_state.left_tab == LeftTab::Assets, "Assets").clicked() {
                            self.ui_state.left_tab = LeftTab::Assets;
                        }
                    });
                    ui.separator();
                    
                    match self.ui_state.left_tab {
                        LeftTab::Palette => palette::render_palette(ui, &mut editor_ctx),
                        LeftTab::Assets => assets::render_assets(ui, &mut editor_ctx),
                    }
                });
        }

        // --- RIGHT PANEL ---
        if self.ui_state.right_panel_expanded {
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    // Split vertically: Hierarchy on top, [Inspector | Variables] on bottom
                    let available_height = ui.available_height();
                    let half_height = available_height / 2.0;
                    
                    egui::ScrollArea::vertical()
                        .max_height(half_height)
                        .id_salt("hierarchy_scroll")
                        .show(ui, |ui| {
                            hierarchy::render_hierarchy(ui, &mut editor_ctx);
                        });
                        
                    ui.separator();
                    
                    // Tab switcher for bottom half
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.ui_state.right_bottom_tab == RightBottomTab::Inspector, "Inspector").clicked() {
                            self.ui_state.right_bottom_tab = RightBottomTab::Inspector;
                        }
                        if ui.selectable_label(self.ui_state.right_bottom_tab == RightBottomTab::Variables, "Variables").clicked() {
                            self.ui_state.right_bottom_tab = RightBottomTab::Variables;
                        }
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("inspector_vars_scroll")
                        .show(ui, |ui| {
                            match self.ui_state.right_bottom_tab {
                                RightBottomTab::Inspector => inspector::render_inspector(ui, &mut editor_ctx),
                                RightBottomTab::Variables => variables::render_variables(ui, &mut editor_ctx),
                            }
                        });
                });
        }

        // --- CENTRAL PANEL (Canvas) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas::render_canvas(ui, &mut editor_ctx);
        });

        // --- WINDOWS ---
        if self.ui_state.show_code_preview {
            egui::Window::new("Code Preview")
                .open(&mut self.ui_state.show_code_preview)
                .default_size([600.0, 500.0])
                .show(ctx, |ui| {
                    code_preview::render_code_preview(ui, &mut editor_ctx);
                });
        }

        if self.ui_state.show_project_settings {
            egui::Window::new("Project Settings")
                .open(&mut self.ui_state.show_project_settings)
                .default_size([300.0, 150.0])
                .show(ctx, |ui| {
                    ui.label("Project Configuration");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Project Name:");
                        ui.text_edit_singleline(&mut editor_ctx.project_state.project_name);
                    });
                    // Future settings can go here
                });
        }
    }
}
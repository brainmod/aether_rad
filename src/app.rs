use crate::model::ProjectState;
use crate::theme::{self, ThemeMode};
use crate::ui::{default_layout, AetherTab, AetherTabViewer};
use crate::validator::ValidationStatus;
use crate::widgets::{ButtonWidget, LabelWidget, VerticalLayout};
use eframe::App;
use egui_dock::{DockArea, DockState};

pub struct AetherApp {
    // The visual state of the docking area (layout, tabs, sizes)
    dock_state: DockState<AetherTab>,

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
            dock_state: default_layout(),
            project_state: ProjectState::new(Box::new(root)),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            validation_status: ValidationStatus::NotRun,
            theme_mode: ThemeMode::Dark,
            theme_initialized: false,
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Project").clicked() {
                        self.push_undo();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aether Project", &["json"])
                            .save_file()
                        {
                            if let Ok(file) = std::fs::File::create(path) {
                                let _ = serde_json::to_writer_pretty(file, &self.project_state);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Load Project").clicked() {
                        self.push_undo();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aether Project", &["json"])
                            .pick_file()
                        {
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
            });
        });

        // Create the viewer, passing mutable access to the project data
        let mut viewer = AetherTabViewer {
            project_state: &mut self.project_state,
            validation_status: &mut self.validation_status,
            theme_mode: &mut self.theme_mode,
        };

        // Render the docking area
        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut viewer);
    }
}

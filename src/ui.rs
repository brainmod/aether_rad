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

        let widgets = vec!["Button", "Vertical Layout"];

        for widget_type in widgets {
            let id = egui::Id::new("palette").with(widget_type);
            ui.dnd_drag_source(id, widget_type.to_string(), |ui| {
                ui.button(widget_type);
            });
        }
    }

    fn render_hierarchy(&mut self, ui: &mut Ui) {
        ui.heading("Tree View");
        let ps = &mut self.project_state;
        draw_hierarchy_node(ui, ps.root_node.as_ref(), &mut ps.selection);
    }

    fn render_inspector(&mut self, ui: &mut Ui) {
        // RIGHT: Property editor

        if let Some(id) = self.project_state.selection.iter().next().cloned() {
            let known_vars: Vec<String> = self.project_state.variables.keys().cloned().collect();

            if let Some(node) = self.project_state.find_node_mut(id) {
                node.inspect(ui, &known_vars);

                return;
            }
        }

        ui.label("No widget selected.");
    }

    fn render_output(&mut self, ui: &mut Ui) {
        ui.label("Compilation Output / Logs");
        ui.code("Waiting for build...");
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
}

fn draw_hierarchy_node(ui: &mut Ui, node: &dyn WidgetNode, selection: &mut HashSet<Uuid>) {
    let id = node.id();
    let is_selected = selection.contains(&id);

    let children = node.children();
    let has_children = children.map_or(false, |c| !c.is_empty());

    if has_children {
        let id_source = id;
        let color = if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().text_color()
        };
        let title = egui::RichText::new(node.name()).color(color);

        let header = egui::CollapsingHeader::new(title)
            .id_salt(id_source)
            .default_open(true);

        let body_response = header.show(ui, |ui| {
            if let Some(children) = children {
                for child in children {
                    draw_hierarchy_node(ui, child.as_ref(), selection);
                }
            }
        });

        if body_response.header_response.clicked() {
            selection.clear();
            selection.insert(id);
        }
    } else {
        if ui.selectable_label(is_selected, node.name()).clicked() {
            selection.clear();
            selection.insert(id);
        }
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

    dock_state
}

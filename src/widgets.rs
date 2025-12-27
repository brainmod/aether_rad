use crate::model::WidgetNode;
use egui::Ui;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// ... existing imports
// Ensure you import ButtonWidget and the necessary macros

/// A container that arranges children vertically.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerticalLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub spacing: f32,
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for VerticalLayout {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Vertical Layout"
    }

    // RECURSION: Render children inside a vertical layout
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing.y = self.spacing;
                for child in &mut self.children {
                    child.render_editor(ui, selection);
                }
            })
            .response;

        // Interaction & Gizmo for the container itself
        let is_selected = selection.contains(&self.id);

        // Gizmo (Outline)
        if is_selected {
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)),
                egui::StrokeKind::Outside,
            );
        }

        // Drop Zone
        let (_response, payload_option) = ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
            ui.label("Drag widget here to add...");
        });

        if let Some(payload) = payload_option {
            // Check if dropped
            if ui.input(|i| i.pointer.any_released()) {
                let new_widget: Box<dyn WidgetNode> = match payload.as_str() {
                    "Button" => Box::new(ButtonWidget::default()),
                    "Vertical Layout" => Box::new(VerticalLayout::default()),
                    _ => return,
                };
                self.children.push(new_widget);
            }
        }
    }
    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String]) {
        ui.heading("Vertical Layout Settings");
        ui.label(format!("ID: {}", self.id));
        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.1));
        });

        ui.label(format!("Children count: {}", self.children.len()));
    }

    // RECURSION: Generate code for the layout and all children
    fn codegen(&self) -> proc_macro2::TokenStream {
        // 1. Generate token streams for all children
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();

        // 2. Wrap them in the egui vertical builder
        quote! {
            ui.vertical(|ui| {
                #(#child_streams)*
            });
        }
    }

    // Expose children for the Hierarchy View (Tree Walker)
    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A concrete implementation of a Button.
/// Annotated with typetag to register it with the serialization system.
/// [cite: 59]
#[derive(Debug, Serialize, Deserialize)]
pub struct ButtonWidget {
    pub id: Uuid,
    pub text: String,
    pub clicked_code: String, // Simulating a basic event action

    // Maps property name (e.g. "text") to variable name (e.g. "counter")
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ButtonWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "Click Me".to_string(),
            clicked_code: String::new(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ButtonWidget {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Button"
    }

    // Render logic for the Editor Canvas
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        // In the editor, we just draw the button.
        // Later (Phase 3), this will be wrapped in interaction interceptors.
        // [cite: 107]
        let response = ui.button(&self.text);

        if response.clicked() {
            selection.clear();
            selection.insert(self.id);
        }

        if selection.contains(&self.id) {
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)),
                egui::StrokeKind::Outside,
            );
        }
    }

    // The "Inspectable" pattern: The widget defines its own property UI.
    // [cite: 137]
    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Button Properties");
        ui.label(format!("ID: {}", self.id));

        ui.horizontal(|ui| {
            ui.label("Label Text:");

            // Check binding status
            let is_bound = self.bindings.contains_key("text");
            let mut bound_mode = is_bound;

            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    // Switch to bound: set default if empty
                    if !known_variables.is_empty() {
                        self.bindings
                            .insert("text".to_string(), known_variables[0].clone());
                    } else {
                        self.bindings.insert("text".to_string(), "".to_string());
                    }
                } else {
                    self.bindings.remove("text");
                }
            }

            if bound_mode {
                let current_var = self.bindings.get("text").cloned().unwrap_or_default();
                let mut selected_var = current_var.clone();

                egui::ComboBox::from_id_salt("btn_txt_bind")
                    .selected_text(&selected_var)
                    .show_ui(ui, |ui| {
                        for var in known_variables {
                            ui.selectable_value(&mut selected_var, var.clone(), var);
                        }
                    });

                if selected_var != current_var {
                    self.bindings.insert("text".to_string(), selected_var);
                }
            } else {
                ui.text_edit_singleline(&mut self.text);
            }
        });

        ui.label("On Click Code:");
        ui.code_editor(&mut self.clicked_code);
    }

    // Generating the AST for the final Rust application.
    // [cite: 184]
    fn codegen(&self) -> proc_macro2::TokenStream {
        let label = &self.text;
        quote! {
            if ui.button(#label).clicked() {
                // Logic would be injected here
            }
        }
    }
}

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
                    "Label" => Box::new(LabelWidget::default()),
                    "Text Edit" => Box::new(TextEditWidget::default()),
                    "Checkbox" => Box::new(CheckboxWidget::default()),
                    "Slider" => Box::new(SliderWidget::default()),
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
        let label_tokens = if let Some(var_name) = self.bindings.get("text") {
            let ident = quote::format_ident!("{}", var_name);
            quote! { &self.#ident }
        } else {
            let text = &self.text;
            quote! { #text }
        };

        quote! {
            if ui.button(#label_tokens).clicked() {
                // Logic would be injected here
            }
        }
    }
}

// ===================== NEW WIDGETS =====================

// --- Label ---
#[derive(Debug, Serialize, Deserialize)]
pub struct LabelWidget {
    pub id: Uuid,
    pub text: String,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for LabelWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "Label".to_string(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for LabelWidget {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Label"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.label(&self.text);
        let response = response.interact(egui::Sense::click()); // Labels aren't clickable by default

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

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Label Properties");
        ui.horizontal(|ui| {
            ui.label("Text:");
            // Simplified binding logic for prototype
            let is_bound = self.bindings.contains_key("text");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("text".to_string(), first);
                } else {
                    self.bindings.remove("text");
                }
            }
            if bound_mode {
                let current = self.bindings.get("text").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("lbl_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("text".to_string(), selected);
                }
            } else {
                ui.text_edit_singleline(&mut self.text);
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let content = if let Some(var) = self.bindings.get("text") {
            let ident = quote::format_ident!("{}", var);
            quote! { &self.#ident }
        } else {
            let t = &self.text;
            quote! { #t }
        };
        quote! { ui.label(#content); }
    }
}

// --- TextEdit ---
#[derive(Debug, Serialize, Deserialize)]
pub struct TextEditWidget {
    pub id: Uuid,
    pub text: String, // Fallback if not bound
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for TextEditWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "".to_string(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for TextEditWidget {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Text Edit"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.text_edit_singleline(&mut self.text); // In editor, it's just local state
                                                                // TextEdit captures click, so we might need a frame or sense logic.
                                                                // For now, assume clicking it selects it.
        if response.clicked() || response.has_focus() {
            if !selection.contains(&self.id) {
                selection.clear();
                selection.insert(self.id);
            }
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

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Text Edit Properties");
        ui.label("Note: Binds to String variables.");
        ui.horizontal(|ui| {
            ui.label("Bind Value:");
            let is_bound = self.bindings.contains_key("value");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("value".to_string(), first);
                } else {
                    self.bindings.remove("value");
                }
            }
            if bound_mode {
                let current = self.bindings.get("value").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("txt_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("value".to_string(), selected);
                }
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        if let Some(var) = self.bindings.get("value") {
            let ident = quote::format_ident!("{}", var);
            quote! { ui.text_edit_singleline(&mut self.#ident); }
        } else {
            // If not bound, it's just a placeholder or needs local state we don't track well in codegen yet
            quote! { ui.label("Unbound TextEdit"); }
        }
    }
}

// --- Checkbox ---
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckboxWidget {
    pub id: Uuid,
    pub label: String,
    pub checked: bool,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for CheckboxWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            label: "Check me".to_string(),
            checked: false,
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for CheckboxWidget {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Checkbox"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.checkbox(&mut self.checked, &self.label);
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

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Checkbox Properties");
        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.text_edit_singleline(&mut self.label);
        });
        ui.horizontal(|ui| {
            ui.label("Bind Checked (Bool):");
            let is_bound = self.bindings.contains_key("checked");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("checked".to_string(), first);
                } else {
                    self.bindings.remove("checked");
                }
            }
            if bound_mode {
                let current = self.bindings.get("checked").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("chk_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("checked".to_string(), selected);
                }
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let label = &self.label;
        if let Some(var) = self.bindings.get("checked") {
            let ident = quote::format_ident!("{}", var);
            quote! { ui.checkbox(&mut self.#ident, #label); }
        } else {
            let val = self.checked;
            quote! {
                let mut temp = #val;
                ui.checkbox(&mut temp, #label);
            }
        }
    }
}

// --- Slider ---
#[derive(Debug, Serialize, Deserialize)]
pub struct SliderWidget {
    pub id: Uuid,
    pub min: f64,
    pub max: f64,
    pub value: f64,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for SliderWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            min: 0.0,
            max: 100.0,
            value: 50.0,
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for SliderWidget {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Slider"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.add(egui::Slider::new(&mut self.value, self.min..=self.max));
        if response.clicked() || response.dragged() {
            // Sliders are dragged
            if !selection.contains(&self.id) {
                selection.clear();
                selection.insert(self.id);
            }
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

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Slider Properties");
        ui.horizontal(|ui| {
            ui.label("Min:");
            ui.add(egui::DragValue::new(&mut self.min));
            ui.label("Max:");
            ui.add(egui::DragValue::new(&mut self.max));
        });
        ui.horizontal(|ui| {
            ui.label("Bind Value (Num):");
            let is_bound = self.bindings.contains_key("value");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("value".to_string(), first);
                } else {
                    self.bindings.remove("value");
                }
            }
            if bound_mode {
                let current = self.bindings.get("value").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("sld_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("value".to_string(), selected);
                }
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let min = self.min;
        let max = self.max;
        if let Some(var) = self.bindings.get("value") {
            let ident = quote::format_ident!("{}", var);
            // We assume the variable is numeric. In a real compiler we'd cast.
            quote! { ui.add(egui::Slider::new(&mut self.#ident, #min..=#max)); }
        } else {
            let val = self.value;
            quote! {
                let mut temp = #val;
                ui.add(egui::Slider::new(&mut temp, #min..=#max));
            }
        }
    }
}

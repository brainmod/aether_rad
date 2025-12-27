use crate::model::WidgetNode;
use egui::Ui;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// ... existing imports
// Ensure you import ButtonWidget and the necessary macros

// === Gizmo Helper Functions ===

const GIZMO_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);
const HANDLE_SIZE: f32 = 8.0;

/// Draw a selection gizmo (orange outline) around a widget
fn draw_gizmo(ui: &egui::Ui, rect: egui::Rect) {
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, GIZMO_COLOR),
        egui::StrokeKind::Outside,
    );
}

/// Draw resize handles at the corners and edges of a rect
fn draw_resize_handles(ui: &egui::Ui, rect: egui::Rect) {
    let handles = [
        (rect.left_top(), "nw"),
        (rect.center_top(), "n"),
        (rect.right_top(), "ne"),
        (rect.right_center(), "e"),
        (rect.right_bottom(), "se"),
        (rect.center_bottom(), "s"),
        (rect.left_bottom(), "sw"),
        (rect.left_center(), "w"),
    ];

    for (pos, _label) in handles {
        let handle_rect = egui::Rect::from_center_size(pos, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
        ui.painter().rect_filled(handle_rect, 0.0, GIZMO_COLOR);
        ui.painter().rect_stroke(
            handle_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
}

/// Check if mouse is hovering over a resize handle and return which one
fn check_resize_handle(_ui: &egui::Ui, rect: egui::Rect, mouse_pos: egui::Pos2) -> Option<&'static str> {
    let handles = [
        (rect.left_top(), "nw"),
        (rect.center_top(), "n"),
        (rect.right_top(), "ne"),
        (rect.right_center(), "e"),
        (rect.right_bottom(), "se"),
        (rect.center_bottom(), "s"),
        (rect.left_bottom(), "sw"),
        (rect.left_center(), "w"),
    ];

    for (pos, label) in handles {
        let handle_rect = egui::Rect::from_center_size(pos, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
        if handle_rect.contains(mouse_pos) {
            return Some(label);
        }
    }
    None
}

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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            spacing: self.spacing,
        })
    }

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
            draw_gizmo(ui, response.rect);
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
                    "Progress Bar" => Box::new(ProgressBarWidget::default()),
                    "ComboBox" => Box::new(ComboBoxWidget::default()),
                    "Image" => Box::new(ImageWidget::default()),
                    "Vertical Layout" => Box::new(VerticalLayout::default()),
                    "Horizontal Layout" => Box::new(HorizontalLayout::default()),
                    "Grid Layout" => Box::new(GridLayout::default()),
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

    // ... VerticalLayout ...

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

/// A container that arranges children horizontally.
#[derive(Debug, Serialize, Deserialize)]
pub struct HorizontalLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub spacing: f32,
}

impl Default for HorizontalLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for HorizontalLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            spacing: self.spacing,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Horizontal Layout"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = self.spacing;
                for child in &mut self.children {
                    child.render_editor(ui, selection);
                }

                // Drop Zone inside Horizontal Layout?
                // A bit tricky visually in horizontal, but let's try adding it at the end.
                let (_response, payload_option) =
                    ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
                        ui.label(" + ");
                    });

                if let Some(payload) = payload_option {
                    if ui.input(|i| i.pointer.any_released()) {
                        let new_widget: Box<dyn WidgetNode> = match payload.as_str() {
                            "Button" => Box::new(ButtonWidget::default()),
                            "Label" => Box::new(LabelWidget::default()),
                            "Text Edit" => Box::new(TextEditWidget::default()),
                            "Checkbox" => Box::new(CheckboxWidget::default()),
                            "Slider" => Box::new(SliderWidget::default()),
                            "Progress Bar" => Box::new(ProgressBarWidget::default()),
                            "ComboBox" => Box::new(ComboBoxWidget::default()),
                            "Image" => Box::new(ImageWidget::default()),
                            "Vertical Layout" => Box::new(VerticalLayout::default()),
                            "Horizontal Layout" => Box::new(HorizontalLayout::default()),
                            "Grid Layout" => Box::new(GridLayout::default()),
                            _ => return,
                        };
                        self.children.push(new_widget);
                    }
                }
            })
            .response;

        let is_selected = selection.contains(&self.id);
        if is_selected {
            draw_gizmo(ui, response.rect);
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String]) {
        ui.heading("Horizontal Layout Settings");
        ui.label(format!("ID: {}", self.id));
        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.1));
        });
        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();
        quote! {
            ui.horizontal(|ui| {
                #(#child_streams)*
            });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A container that arranges children in a grid with a specified number of columns.
#[derive(Debug, Serialize, Deserialize)]
pub struct GridLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub columns: usize,
    pub spacing: f32,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            columns: 2, // Default to 2 columns
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for GridLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            columns: self.columns,
            spacing: self.spacing,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Grid Layout"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(self.spacing, self.spacing);

                // Render children in grid format
                let mut row_children = Vec::new();
                let total_children = self.children.len();
                for (idx, child) in self.children.iter_mut().enumerate() {
                    row_children.push(child);

                    // When we reach the column count or the last child, render the row
                    if (idx + 1) % self.columns == 0 || idx == total_children - 1 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = self.spacing;
                            for child in row_children.drain(..) {
                                child.render_editor(ui, selection);
                            }
                        });
                    }
                }

                // Drop Zone for adding widgets to grid
                let (_response, payload_option) =
                    ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
                        ui.label("Drag widget here to add...");
                    });

                if let Some(payload) = payload_option {
                    if ui.input(|i| i.pointer.any_released()) {
                        let new_widget: Box<dyn WidgetNode> = match payload.as_str() {
                            "Button" => Box::new(ButtonWidget::default()),
                            "Label" => Box::new(LabelWidget::default()),
                            "Text Edit" => Box::new(TextEditWidget::default()),
                            "Checkbox" => Box::new(CheckboxWidget::default()),
                            "Slider" => Box::new(SliderWidget::default()),
                            "Progress Bar" => Box::new(ProgressBarWidget::default()),
                            "ComboBox" => Box::new(ComboBoxWidget::default()),
                            "Image" => Box::new(ImageWidget::default()),
                            "Vertical Layout" => Box::new(VerticalLayout::default()),
                            "Horizontal Layout" => Box::new(HorizontalLayout::default()),
                            "Grid Layout" => Box::new(GridLayout::default()),
                            _ => return,
                        };
                        self.children.push(new_widget);
                    }
                }
            })
            .response;

        let is_selected = selection.contains(&self.id);
        if is_selected {
            draw_gizmo(ui, response.rect);
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String]) {
        ui.heading("Grid Layout Settings");
        ui.label(format!("ID: {}", self.id));
        ui.horizontal(|ui| {
            ui.label("Columns:");
            ui.add(egui::DragValue::new(&mut self.columns).speed(1.0).range(1..=10));
        });
        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.1));
        });
        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let columns = self.columns;

        // Group children into rows
        let mut row_streams = Vec::new();
        let mut current_row = Vec::new();

        for (idx, child) in self.children.iter().enumerate() {
            current_row.push(child.codegen());

            // When we reach the column count or the last child, complete the row
            if (idx + 1) % columns == 0 || idx == self.children.len() - 1 {
                let row_widgets = current_row.drain(..).collect::<Vec<_>>();
                row_streams.push(quote! {
                    ui.horizontal(|ui| {
                        #(#row_widgets)*
                    });
                });
            }
        }

        quote! {
            ui.vertical(|ui| {
                #(#row_streams)*
            });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A concrete implementation of a Button.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

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
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip on hover with widget properties
        response.on_hover_text(format!("Button: {}\nID: {}", self.text, self.id));
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

        ui.separator();
        ui.heading("On Click Action");
        ui.label("Write Rust code to execute when button is clicked:");
        ui.label("Example: self.counter += 1;");

        let code_editor = egui::TextEdit::multiline(&mut self.clicked_code)
            .code_editor()
            .desired_rows(5)
            .desired_width(f32::INFINITY);
        ui.add(code_editor);

        if !self.clicked_code.trim().is_empty() {
            // Show validation feedback
            if self.clicked_code.parse::<proc_macro2::TokenStream>().is_ok() {
                ui.colored_label(egui::Color32::GREEN, "✓ Valid Rust syntax");
            } else {
                ui.colored_label(egui::Color32::RED, "✗ Invalid Rust syntax");
            }
        }
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

        // Parse and inject clicked_code if present
        let action_code = if !self.clicked_code.trim().is_empty() {
            // Parse the code string as Rust tokens
            match self.clicked_code.parse::<proc_macro2::TokenStream>() {
                Ok(tokens) => tokens,
                Err(_) => {
                    // If parsing fails, insert a comment indicating the error
                    quote! { /* Invalid Rust code in clicked_code */ }
                }
            }
        } else {
            quote! { /* No action code */ }
        };

        quote! {
            if ui.button(#label_tokens).clicked() {
                #action_code
            }
        }
    }
}

// ===================== NEW WIDGETS =====================

// --- Label ---
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

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
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Label: {}\nID: {}", self.text, self.id));
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

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
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Text Edit: {}\nID: {}", self.text, self.id));
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

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
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Checkbox: {} ({})\nID: {}", self.label, if self.checked { "✓" } else { "☐" }, self.id));
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

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
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Slider: {} ({}-{})\nID: {}", self.value, self.min, self.max, self.id));
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
            // Use `as _` to allow the compiler to infer the correct numeric type (f64, f32, i32, etc)
            // for the range limits based on the variable's type.
            quote! { ui.add(egui::Slider::new(&mut self.#ident, (#min as _)..=(#max as _))); }
        } else {
            let val = self.value;
            quote! {
                let mut temp = #val;
                ui.add(egui::Slider::new(&mut temp, #min..=#max));
            }
        }
    }
}

// --- ProgressBar ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressBarWidget {
    pub id: Uuid,
    pub value: f32, // 0.0 to 1.0
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ProgressBarWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            value: 0.5,
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ProgressBarWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Progress Bar"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.add(egui::ProgressBar::new(self.value).show_percentage());
        let response = response.interact(egui::Sense::click());

        if response.clicked() {
            selection.clear();
            selection.insert(self.id);
        }
        if selection.contains(&self.id) {
            draw_gizmo(ui, response.rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Progress Bar: {:.0}%\nID: {}", self.value * 100.0, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("Progress Bar Properties");
        ui.horizontal(|ui| {
            ui.label("Progress (0.0 - 1.0):");
            ui.add(egui::Slider::new(&mut self.value, 0.0..=1.0));
        });
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
                egui::ComboBox::from_id_salt("progress_bind")
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
            quote! { ui.add(egui::ProgressBar::new(self.#ident).show_percentage()); }
        } else {
            let val = self.value;
            quote! { ui.add(egui::ProgressBar::new(#val).show_percentage()); }
        }
    }
}

// --- ComboBox ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComboBoxWidget {
    pub id: Uuid,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ComboBoxWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            label: "Select:".to_string(),
            options: vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()],
            selected: 0,
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ComboBoxWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "ComboBox"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let outer_response = ui.horizontal(|ui| {
            ui.label(&self.label);

            let selected_text = self.options.get(self.selected).map(|s| s.as_str()).unwrap_or("");
            let response = egui::ComboBox::from_id_salt(format!("combo_{}", self.id))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (idx, opt) in self.options.iter().enumerate() {
                        ui.selectable_value(&mut self.selected, idx, opt);
                    }
                });

            if response.response.clicked() {
                selection.clear();
                selection.insert(self.id);
            }

            if selection.contains(&self.id) {
                draw_gizmo(ui, response.response.rect);
            }

            response.response
        });

        // Show tooltip
        let selected_text = self.options.get(self.selected).map(|s| s.as_str()).unwrap_or("");
        outer_response.response.on_hover_text(format!("ComboBox: {}\nSelected: {}\nID: {}", self.label, selected_text, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) {
        ui.heading("ComboBox Properties");

        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.text_edit_singleline(&mut self.label);
        });

        ui.separator();
        ui.label("Options:");

        let mut to_remove = None;
        for (idx, opt) in self.options.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(opt);
                if ui.button("🗑").clicked() {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            self.options.remove(idx);
            if self.selected >= self.options.len() && !self.options.is_empty() {
                self.selected = self.options.len() - 1;
            }
        }

        if ui.button("+ Add Option").clicked() {
            self.options.push(format!("Option {}", self.options.len() + 1));
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Bind Selected Index:");
            let is_bound = self.bindings.contains_key("selected");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("selected".to_string(), first);
                } else {
                    self.bindings.remove("selected");
                }
            }
            if bound_mode {
                let current = self.bindings.get("selected").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("combo_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("selected".to_string(), selected);
                }
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let label = &self.label;
        let options: Vec<_> = self.options.iter().map(|s| s.as_str()).collect();

        if let Some(var) = self.bindings.get("selected") {
            let ident = quote::format_ident!("{}", var);
            quote! {
                ui.horizontal(|ui| {
                    ui.label(#label);
                    let options = vec![#(#options),*];
                    let selected_text = options.get(self.#ident).unwrap_or(&"");
                    egui::ComboBox::from_label("")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (idx, opt) in options.iter().enumerate() {
                                ui.selectable_value(&mut self.#ident, idx, opt);
                            }
                        });
                });
            }
        } else {
            let selected = self.selected;
            quote! {
                ui.horizontal(|ui| {
                    ui.label(#label);
                    let mut selected = #selected;
                    let options = vec![#(#options),*];
                    let selected_text = options.get(selected).unwrap_or(&"");
                    egui::ComboBox::from_label("")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (idx, opt) in options.iter().enumerate() {
                                ui.selectable_value(&mut selected, idx, opt);
                            }
                        });
                });
            }
        }
    }
}

// --- Image ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageWidget {
    pub id: Uuid,
    pub path: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            path: "".to_string(),
            width: Some(100.0),
            height: None, // Maintain aspect ratio
        }
    }
}

#[typetag::serde]
impl WidgetNode for ImageWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Image"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = if self.path.is_empty() {
            ui.label("[No Image]")
        } else {
            ui.label(format!("🖼 {}", self.path.split('/').last().unwrap_or(&self.path)))
        };

        let response = response.interact(egui::Sense::click());

        if response.clicked() {
            selection.clear();
            selection.insert(self.id);
        }

        let is_selected = selection.contains(&self.id);
        if is_selected {
            draw_gizmo(ui, response.rect);
            draw_resize_handles(ui, response.rect);

            // Handle resize dragging
            if let Some(hover_pos) = ui.ctx().pointer_hover_pos() {
                if let Some(handle) = check_resize_handle(ui, response.rect, hover_pos) {
                    // Show resize cursor based on handle direction
                    let cursor = match handle {
                        "nw" | "se" => egui::CursorIcon::ResizeNwSe,
                        "ne" | "sw" => egui::CursorIcon::ResizeNeSw,
                        "n" | "s" => egui::CursorIcon::ResizeVertical,
                        "e" | "w" => egui::CursorIcon::ResizeHorizontal,
                        _ => egui::CursorIcon::Default,
                    };
                    ui.ctx().set_cursor_icon(cursor);

                    // If dragging, resize the image
                    if ui.input(|i| i.pointer.primary_down()) {
                        let delta = ui.input(|i| i.pointer.delta());

                        // Handle horizontal resize
                        match handle {
                            "e" | "ne" | "se" => {
                                if let Some(w) = self.width.as_mut() {
                                    *w = (*w + delta.x).max(10.0);
                                }
                            }
                            "w" | "nw" | "sw" => {
                                if let Some(w) = self.width.as_mut() {
                                    *w = (*w - delta.x).max(10.0);
                                }
                            }
                            _ => {}
                        }

                        // Handle vertical resize
                        match handle {
                            "s" | "se" | "sw" => {
                                if let Some(h) = self.height.as_mut() {
                                    *h = (*h + delta.y).max(10.0);
                                }
                            }
                            "n" | "nw" | "ne" => {
                                if let Some(h) = self.height.as_mut() {
                                    *h = (*h - delta.y).max(10.0);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Show tooltip with image info
        let size_info = match (self.width, self.height) {
            (Some(w), Some(h)) => format!(" ({}x{})", w, h),
            (Some(w), None) => format!(" (w:{})", w),
            (None, Some(h)) => format!(" (h:{})", h),
            (None, None) => "".to_string(),
        };
        response.on_hover_text(format!("Image{}\nPath: {}\nID: {}", size_info, self.path, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String]) {
        ui.heading("Image Properties");

        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut self.path);
        });

        if ui.button("📁 Browse...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    self.path = path_str.to_string();
                }
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Width:");
            let mut has_width = self.width.is_some();
            if ui.checkbox(&mut has_width, "").changed() {
                self.width = if has_width { Some(100.0) } else { None };
            }
            if let Some(ref mut w) = self.width {
                ui.add(egui::DragValue::new(w).speed(1.0).range(10.0..=1000.0));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            let mut has_height = self.height.is_some();
            if ui.checkbox(&mut has_height, "").changed() {
                self.height = if has_height { Some(100.0) } else { None };
            }
            if let Some(ref mut h) = self.height {
                ui.add(egui::DragValue::new(h).speed(1.0).range(10.0..=1000.0));
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let path = &self.path;

        let size_tokens = match (self.width, self.height) {
            (Some(w), Some(h)) => quote! { .max_size(egui::vec2(#w, #h)) },
            (Some(w), None) => quote! { .max_width(#w) },
            (None, Some(h)) => quote! { .max_height(#h) },
            (None, None) => quote! {},
        };

        quote! {
            ui.add(
                egui::Image::new(#path)
                    #size_tokens
            );
        }
    }
}

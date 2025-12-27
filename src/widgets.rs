use crate::model::WidgetNode;
use egui::Ui;
use quote::quote;
use serde::{Deserialize, Serialize};

// ... existing imports
// Ensure you import ButtonWidget and the necessary macros

/// A container that arranges children vertically.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerticalLayout {
    pub children: Vec<Box<dyn WidgetNode>>,
    pub spacing: f32,
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for VerticalLayout {
    fn name(&self) -> &str {
        "Vertical Layout"
    }

    // RECURSION: Render children inside a vertical layout
    fn render_editor(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = self.spacing;
            for child in &mut self.children {
                child.render_editor(ui);
            }
        });
    }

    fn inspect(&mut self, ui: &mut Ui) {
        ui.heading("Vertical Layout Settings");
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
    pub text: String,
    pub clicked_code: String, // Simulating a basic event action
}

impl Default for ButtonWidget {
    fn default() -> Self {
        Self {
            text: "Click Me".to_string(),
            clicked_code: String::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ButtonWidget {
    fn name(&self) -> &str {
        "Button"
    }

    // Render logic for the Editor Canvas
    fn render_editor(&mut self, ui: &mut Ui) {
        // In the editor, we just draw the button.
        // Later (Phase 3), this will be wrapped in interaction interceptors.
        // [cite: 107]
        ui.button(&self.text);
    }

    // The "Inspectable" pattern: The widget defines its own property UI.
    // [cite: 137]
    fn inspect(&mut self, ui: &mut Ui) {
        ui.heading("Button Properties");

        ui.horizontal(|ui| {
            ui.label("Label Text:");
            ui.text_edit_singleline(&mut self.text);
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

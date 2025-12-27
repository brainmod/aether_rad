use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use egui::Ui;
use proc_macro2::TokenStream;

/// The contract for any element that can exist in the designer.
/// Uses typetag to allow for polymorphic serialization of trait objects.
/// [cite: 47, 55]
#[typetag::serde(tag = "type")]
pub trait WidgetNode: std::fmt::Debug {
    /// Distinct behavior 1: Editor Visualization
    /// How the widget renders itself inside the designer canvas.
    /// [cite: 50]
    fn render_editor(&mut self, ui: &mut Ui);

    /// Distinct behavior 2: Property Introspection
    /// How the widget exposes configurable fields to the Inspector.
    /// [cite: 51, 134]
    fn inspect(&mut self, ui: &mut Ui);

    /// Distinct behavior 3: Code Generation
    /// Synthesizes the Rust code required to instantiate this widget.
    /// [cite: 52, 184]
    fn codegen(&self) -> TokenStream {
        quote::quote! { /* Default no-op */ }
    }

    /// Helper to get the display name for the Hierarchy View
    fn name(&self) -> &str;

    /// Helper to get children (if container) for tree traversal
    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> { None }

    /// Mutable access to children for drag-and-drop re-parenting
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> { None }
}

/// The root container for the entire application definition.
/// [cite: 61]
#[derive(Serialize, Deserialize)]
pub struct ProjectState {
    /// The entry point of the UI tree.
    ///
    pub root_node: Box<dyn WidgetNode>,

    /// Selection set for gizmos and inspector.
    ///
    pub selection: HashSet<Uuid>,

    /// Application state variables (e.g., "counter: i32").
    ///
    pub variables: HashMap<String, String>,
}

impl ProjectState {
    pub fn new(root: Box<dyn WidgetNode>) -> Self {
        Self {
            root_node: root,
            selection: HashSet::new(),
            variables: HashMap::new(),
        }
    }

    /// Serialize the entire project state to JSON.
    /// [cite: 64]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

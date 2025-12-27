use egui::Ui;
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Types supported by the variable store
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableType {
    String,
    Integer,
    Boolean,
    Float,
}

impl std::fmt::Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A variable in the global application state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub v_type: VariableType,
    pub value: String, // Stored as string for simplicity in prototype
}

/// The contract for any element that can exist in the designer.
/// Uses typetag to allow for polymorphic serialization of trait objects.
/// [cite: 47, 55]
#[typetag::serde(tag = "type")]
pub trait WidgetNode: std::fmt::Debug {
    /// Distinct behavior 1: Editor Visualization
    /// How the widget renders itself inside the designer canvas.
    /// [cite: 50]
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>);

    /// Distinct behavior 2: Property Introspection
    /// How the widget exposes configurable fields to the Inspector.
    /// [cite: 51, 134]
    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]);

    /// Distinct behavior 3: Code Generation
    /// Synthesizes the Rust code required to instantiate this widget.
    /// [cite: 52, 184]
    fn codegen(&self) -> TokenStream {
        quote::quote! { /* Default no-op */ }
    }

    /// Unique identifier for the widget instance.
    fn id(&self) -> Uuid;

    /// Helper to get the display name for the Hierarchy View
    fn name(&self) -> &str;

    /// Helper to get children (if container) for tree traversal
    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        None
    }

    /// Mutable access to children for drag-and-drop re-parenting
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        None
    }
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
    pub variables: HashMap<String, Variable>,
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

    /// Recursively find a node by its UUID.
    pub fn find_node_mut(&mut self, id: Uuid) -> Option<&mut dyn WidgetNode> {
        find_node_recursive_mut(self.root_node.as_mut(), id)
    }
}

fn find_node_recursive_mut(node: &mut dyn WidgetNode, target: Uuid) -> Option<&mut dyn WidgetNode> {
    if node.id() == target {
        return Some(node);
    }
    if let Some(children) = node.children_mut() {
        for child in children {
            if let Some(found) = find_node_recursive_mut(child.as_mut(), target) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{ButtonWidget, VerticalLayout};

    #[test]
    fn test_find_node() {
        let mut root = VerticalLayout::default();
        let btn1 = ButtonWidget::default();
        let btn1_id = btn1.id;

        let mut sub_layout = VerticalLayout::default();
        let btn2 = ButtonWidget::default();
        let btn2_id = btn2.id;

        sub_layout.children.push(Box::new(btn2));
        root.children.push(Box::new(btn1));
        root.children.push(Box::new(sub_layout));

        let mut project = ProjectState::new(Box::new(root));

        // Test finding a direct child
        let found1 = project.find_node_mut(btn1_id);
        assert!(found1.is_some());
        assert_eq!(found1.unwrap().id(), btn1_id);

        // Test finding a nested child
        let found2 = project.find_node_mut(btn2_id);
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().id(), btn2_id);

        // Test finding non-existent
        let found3 = project.find_node_mut(Uuid::new_v4());
        assert!(found3.is_none());
    }

    #[test]
    fn test_variable_storage() {
        let mut project = ProjectState::new(Box::new(VerticalLayout::default()));

        project.variables.insert(
            "counter".to_string(),
            Variable {
                name: "counter".to_string(),
                v_type: VariableType::Integer,
                value: "0".to_string(),
            },
        );

        assert!(project.variables.contains_key("counter"));
        assert_eq!(project.variables["counter"].v_type, VariableType::Integer);
    }
}

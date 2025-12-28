use egui::Ui;
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

/// Asset metadata for images and other resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
}

/// Types of assets that can be managed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Image,
    Audio,
    Data,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Image => "Image",
                Self::Audio => "Audio",
                Self::Data => "Data",
            }
        )
    }
}

/// Manages all assets in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManager {
    pub assets: HashMap<String, Asset>, // name -> Asset
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}

impl AssetManager {
    /// Create a new empty asset manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an asset to the manager
    pub fn add_asset(&mut self, name: String, asset_type: AssetType, path: PathBuf) {
        let asset = Asset {
            id: Uuid::new_v4(),
            name: name.clone(),
            asset_type,
            path,
        };
        self.assets.insert(name, asset);
    }

    /// Remove an asset by name
    pub fn remove_asset(&mut self, name: &str) -> Option<Asset> {
        self.assets.remove(name)
    }

    /// Get an asset by name
    pub fn get_asset(&self, name: &str) -> Option<&Asset> {
        self.assets.get(name)
    }

    /// Get all image assets
    pub fn get_images(&self) -> Vec<&Asset> {
        self.assets
            .values()
            .filter(|a| a.asset_type == AssetType::Image)
            .collect()
    }

    /// List all asset names
    pub fn asset_names(&self) -> Vec<&str> {
        self.assets.keys().map(|s| s.as_str()).collect()
    }
}

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

/// Widget event types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WidgetEvent {
    Clicked,
    Changed,
    Hovered,
    DoubleClicked,
    Focused,
    LostFocus,
}

impl std::fmt::Display for WidgetEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Clicked => "On Click",
                Self::Changed => "On Change",
                Self::Hovered => "On Hover",
                Self::DoubleClicked => "On Double Click",
                Self::Focused => "On Focus",
                Self::LostFocus => "On Blur",
            }
        )
    }
}

/// Standard action types that can be executed on widget events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Increment a variable by 1
    IncrementVariable(String),
    /// Set a variable to a value
    SetVariable(String, String),
    /// Custom Rust code
    Custom(String),
}

impl Action {
    /// Convert the action to Rust code
    pub fn to_code(&self) -> proc_macro2::TokenStream {
        match self {
            Action::IncrementVariable(var_name) => {
                let ident = quote::format_ident!("{}", var_name);
                quote::quote! { self.#ident += 1; }
            }
            Action::SetVariable(var_name, value) => {
                let ident = quote::format_ident!("{}", var_name);
                // Try to parse the value as a TokenStream, fallback to string literal
                match value.parse::<proc_macro2::TokenStream>() {
                    Ok(tokens) => quote::quote! { self.#ident = #tokens; },
                    Err(_) => quote::quote! { self.#ident = #value.to_string(); },
                }
            }
            Action::Custom(code) => {
                match code.parse::<proc_macro2::TokenStream>() {
                    Ok(tokens) => tokens,
                    Err(_) => quote::quote! { /* Invalid Rust code */ },
                }
            }
        }
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
    /// Clone this widget node into a boxed trait object
    fn clone_box(&self) -> Box<dyn WidgetNode>;
    /// Distinct behavior 1: Editor Visualization
    /// How the widget renders itself inside the designer canvas.
    /// [cite: 50]
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>);

    /// Distinct behavior 2: Property Introspection
    /// How the widget exposes configurable fields to the Inspector.
    /// [cite: 51, 134]
    /// known_assets: Vec of (asset_name, filename) pairs
    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], known_assets: &[(String, String)]);

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

    /// Project name used for code generation.
    ///
    #[serde(default = "default_project_name")]
    pub project_name: String,

    /// Asset manager for images and other resources.
    ///
    #[serde(default)]
    pub assets: AssetManager,

    /// Pending reorder operation (source_id, target_id).
    /// Not serialized - runtime only.
    #[serde(skip)]
    pub pending_reorder: Option<(Uuid, Uuid)>,
}

fn default_project_name() -> String {
    "my_app".to_string()
}

impl Clone for ProjectState {
    fn clone(&self) -> Self {
        Self {
            root_node: self.root_node.clone_box(),
            selection: self.selection.clone(),
            variables: self.variables.clone(),
            project_name: self.project_name.clone(),
            assets: self.assets.clone(),
            pending_reorder: None, // Reset pending operations on clone
        }
    }
}

impl ProjectState {
    pub fn new(root: Box<dyn WidgetNode>) -> Self {
        Self {
            root_node: root,
            selection: HashSet::new(),
            variables: HashMap::new(),
            project_name: default_project_name(),
            assets: AssetManager::new(),
            pending_reorder: None,
        }
    }

    /// Create an empty project with just a vertical layout
    pub fn empty() -> Self {
        Self::new(Box::new(crate::widgets::VerticalLayout::default()))
    }

    /// Create a counter app template with label, button, and counter variable
    pub fn template_counter_app() -> Self {
        use crate::widgets::{VerticalLayout, LabelWidget, ButtonWidget};

        let mut root = VerticalLayout::default();
        root.children.push(Box::new(LabelWidget {
            text: "Counter App".to_string(),
            ..Default::default()
        }));

        let mut counter_label = LabelWidget {
            text: "Count: 0".to_string(),
            ..Default::default()
        };
        counter_label.bindings.insert("text".to_string(), "counter".to_string());
        root.children.push(Box::new(counter_label));

        let mut increment_button = ButtonWidget {
            text: "Increment".to_string(),
            ..Default::default()
        };
        increment_button.events.insert(
            crate::model::WidgetEvent::Clicked,
            crate::model::Action::IncrementVariable("counter".to_string()),
        );
        root.children.push(Box::new(increment_button));

        let mut project = Self::new(Box::new(root));
        project.project_name = "Counter App".to_string();
        project.variables.insert(
            "counter".to_string(),
            Variable {
                name: "counter".to_string(),
                v_type: VariableType::Integer,
                value: "0".to_string(),
            },
        );
        project
    }

    /// Create a simple form template with fields and submit button
    pub fn template_form() -> Self {
        use crate::widgets::{VerticalLayout, LabelWidget, TextEditWidget, ButtonWidget};

        let mut root = VerticalLayout::default();

        root.children.push(Box::new(LabelWidget {
            text: "Contact Form".to_string(),
            ..Default::default()
        }));

        root.children.push(Box::new(LabelWidget {
            text: "Name:".to_string(),
            ..Default::default()
        }));

        let mut name_field = TextEditWidget {
            text: "".to_string(),
            ..Default::default()
        };
        name_field.bindings.insert("value".to_string(), "name".to_string());
        root.children.push(Box::new(name_field));

        root.children.push(Box::new(LabelWidget {
            text: "Email:".to_string(),
            ..Default::default()
        }));

        let mut email_field = TextEditWidget {
            text: "".to_string(),
            ..Default::default()
        };
        email_field.bindings.insert("value".to_string(), "email".to_string());
        root.children.push(Box::new(email_field));

        root.children.push(Box::new(ButtonWidget {
            text: "Submit".to_string(),
            ..Default::default()
        }));

        let mut project = Self::new(Box::new(root));
        project.project_name = "Contact Form".to_string();
        project.variables.insert(
            "name".to_string(),
            Variable {
                name: "name".to_string(),
                v_type: VariableType::String,
                value: "".to_string(),
            },
        );
        project.variables.insert(
            "email".to_string(),
            Variable {
                name: "email".to_string(),
                v_type: VariableType::String,
                value: "".to_string(),
            },
        );
        project
    }

    /// Create a dashboard template with labels and progress bars
    pub fn template_dashboard() -> Self {
        use crate::widgets::{VerticalLayout, HorizontalLayout, LabelWidget, ProgressBarWidget};

        let mut root = VerticalLayout::default();

        root.children.push(Box::new(LabelWidget {
            text: "Dashboard".to_string(),
            ..Default::default()
        }));

        // First metric row
        let mut row1 = HorizontalLayout::default();
        row1.children.push(Box::new(LabelWidget {
            text: "CPU Usage:".to_string(),
            ..Default::default()
        }));
        let mut cpu_progress = ProgressBarWidget {
            value: 0.45,
            ..Default::default()
        };
        cpu_progress.bindings.insert("value".to_string(), "cpu_usage".to_string());
        row1.children.push(Box::new(cpu_progress));
        root.children.push(Box::new(row1));

        // Second metric row
        let mut row2 = HorizontalLayout::default();
        row2.children.push(Box::new(LabelWidget {
            text: "Memory Usage:".to_string(),
            ..Default::default()
        }));
        let mut memory_progress = ProgressBarWidget {
            value: 0.60,
            ..Default::default()
        };
        memory_progress.bindings.insert("value".to_string(), "memory_usage".to_string());
        row2.children.push(Box::new(memory_progress));
        root.children.push(Box::new(row2));

        let mut project = Self::new(Box::new(root));
        project.project_name = "Dashboard".to_string();
        project.variables.insert(
            "cpu_usage".to_string(),
            Variable {
                name: "cpu_usage".to_string(),
                v_type: VariableType::Float,
                value: "0.45".to_string(),
            },
        );
        project.variables.insert(
            "memory_usage".to_string(),
            Variable {
                name: "memory_usage".to_string(),
                v_type: VariableType::Float,
                value: "0.60".to_string(),
            },
        );
        project
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

    /// Delete a widget by its ID. Returns true if the widget was found and deleted.
    /// Cannot delete the root node.
    pub fn delete_widget(&mut self, id: Uuid) -> bool {
        // Don't allow deleting the root node
        if self.root_node.id() == id {
            return false;
        }

        delete_node_recursive(self.root_node.as_mut(), id)
    }

    /// Find the parent of a widget by the child's ID
    pub fn find_parent_mut(&mut self, child_id: Uuid) -> Option<&mut dyn WidgetNode> {
        find_parent_recursive_mut(self.root_node.as_mut(), child_id)
    }

    /// Move a widget within its parent's children list
    pub fn reorder_widget(&mut self, widget_id: Uuid, new_index: usize) -> bool {
        reorder_widget_recursive(self.root_node.as_mut(), widget_id, new_index)
    }

    /// Move a widget up in its parent's children list (towards index 0)
    pub fn move_widget_up(&mut self, widget_id: Uuid) -> bool {
        move_widget_in_parent(self.root_node.as_mut(), widget_id, -1)
    }

    /// Move a widget down in its parent's children list (towards end)
    pub fn move_widget_down(&mut self, widget_id: Uuid) -> bool {
        move_widget_in_parent(self.root_node.as_mut(), widget_id, 1)
    }

    /// Get all widget IDs in hierarchy order (depth-first traversal)
    pub fn get_all_widget_ids(&self) -> Vec<Uuid> {
        let mut ids = Vec::new();
        collect_widget_ids(self.root_node.as_ref(), &mut ids);
        ids
    }

    /// Get the current root layout type as a string
    pub fn root_layout_type(&self) -> &str {
        self.root_node.name()
    }

    /// Change the root layout type (preserves children and ID)
    pub fn set_root_layout_type(&mut self, layout_type: &str) {
        use crate::widgets::{VerticalLayout, HorizontalLayout, GridLayout};

        // Extract current children and ID
        let current_id = self.root_node.id();
        let current_children = if let Some(children) = self.root_node.children() {
            children.iter().map(|c| c.clone_box()).collect()
        } else {
            Vec::new()
        };

        // Create new root with same ID and children
        self.root_node = match layout_type {
            "Vertical Layout" => Box::new(VerticalLayout {
                id: current_id,
                children: current_children,
                spacing: 5.0,
                ..Default::default()
            }),
            "Horizontal Layout" => Box::new(HorizontalLayout {
                id: current_id,
                children: current_children,
                spacing: 5.0,
            }),
            "Grid Layout" => Box::new(GridLayout {
                id: current_id,
                children: current_children,
                columns: 2,
                spacing: 5.0,
            }),
            _ => return, // Unknown layout type, do nothing
        };
    }

    /// Re-parent a widget: move it from its current parent to a new container at a specific index
    /// Returns true if successful
    pub fn reparent_widget(&mut self, widget_id: Uuid, new_parent_id: Uuid, index: usize) -> bool {
        // Can't re-parent the root node
        if widget_id == self.root_node.id() {
            return false;
        }

        // Can't move a widget into itself or its descendants
        if widget_id == new_parent_id || self.is_descendant_of(new_parent_id, widget_id) {
            return false;
        }

        // Extract the widget from its current location
        if let Some(widget) = extract_widget_recursive(self.root_node.as_mut(), widget_id) {
            // Insert it at the new location
            if insert_widget_at(self.root_node.as_mut(), new_parent_id, widget, index) {
                return true;
            }
        }
        false
    }

    /// Move a widget before another widget (for drop insertion)
    pub fn move_widget_before(&mut self, widget_id: Uuid, before_id: Uuid) -> bool {
        // Find the parent and index of the "before" widget
        if let Some(parent_id) = self.get_parent_id(before_id) {
            if let Some(parent) = self.find_node_mut(parent_id) {
                if let Some(index) = get_widget_index(parent, before_id) {
                    return self.reparent_widget(widget_id, parent_id, index);
                }
            }
        }
        false
    }

    /// Move a widget after another widget (for drop insertion)
    pub fn move_widget_after(&mut self, widget_id: Uuid, after_id: Uuid) -> bool {
        if let Some(parent_id) = self.get_parent_id(after_id) {
            if let Some(parent) = self.find_node_mut(parent_id) {
                if let Some(index) = get_widget_index(parent, after_id) {
                    return self.reparent_widget(widget_id, parent_id, index + 1);
                }
            }
        }
        false
    }

    /// Get the parent ID of a widget
    pub fn get_parent_id(&self, child_id: Uuid) -> Option<Uuid> {
        get_parent_id_recursive(self.root_node.as_ref(), child_id)
    }

    /// Check if `potential_descendant` is a descendant of `potential_ancestor`
    fn is_descendant_of(&self, potential_descendant: Uuid, potential_ancestor: Uuid) -> bool {
        if let Some(node) = find_node_by_id(self.root_node.as_ref(), potential_ancestor) {
            return contains_id_recursive(node, potential_descendant);
        }
        false
    }

    /// Check if a widget is a container that can accept children
    pub fn is_container(&self, widget_id: Uuid) -> bool {
        if let Some(node) = find_node_by_id(self.root_node.as_ref(), widget_id) {
            return is_container(node);
        }
        false
    }
}

fn collect_widget_ids(node: &dyn WidgetNode, ids: &mut Vec<Uuid>) {
    ids.push(node.id());
    if let Some(children) = node.children() {
        for child in children {
            collect_widget_ids(child.as_ref(), ids);
        }
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

fn delete_node_recursive(node: &mut dyn WidgetNode, target: Uuid) -> bool {
    if let Some(children) = node.children_mut() {
        // First, check if any direct child matches the target
        if let Some(index) = children.iter().position(|c| c.id() == target) {
            children.remove(index);
            return true;
        }

        // If not found in direct children, recurse into each child
        for child in children.iter_mut() {
            if delete_node_recursive(child.as_mut(), target) {
                return true;
            }
        }
    }
    false
}

fn find_parent_recursive_mut(node: &mut dyn WidgetNode, child_id: Uuid) -> Option<&mut dyn WidgetNode> {
    // First, check if any direct child matches (without holding the borrow)
    let has_matching_child = if let Some(children) = node.children() {
        children.iter().any(|c| c.id() == child_id)
    } else {
        false
    };

    if has_matching_child {
        return Some(node);
    }

    // Recurse into each child
    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            if let Some(parent) = find_parent_recursive_mut(child.as_mut(), child_id) {
                return Some(parent);
            }
        }
    }
    None
}

fn reorder_widget_recursive(node: &mut dyn WidgetNode, widget_id: Uuid, new_index: usize) -> bool {
    if let Some(children) = node.children_mut() {
        // Check if the widget is a direct child
        if let Some(old_index) = children.iter().position(|c| c.id() == widget_id) {
            if new_index < children.len() {
                let widget = children.remove(old_index);
                let insert_index = if new_index > old_index {
                    new_index - 1
                } else {
                    new_index
                };
                children.insert(insert_index.min(children.len()), widget);
                return true;
            }
        }

        // Recurse into each child
        for child in children.iter_mut() {
            if reorder_widget_recursive(child.as_mut(), widget_id, new_index) {
                return true;
            }
        }
    }
    false
}

/// Move a widget up or down within its parent's children list
/// delta: -1 for up (towards index 0), +1 for down (towards end)
fn move_widget_in_parent(node: &mut dyn WidgetNode, widget_id: Uuid, delta: i32) -> bool {
    if let Some(children) = node.children_mut() {
        // Check if the widget is a direct child
        if let Some(current_index) = children.iter().position(|c| c.id() == widget_id) {
            let new_index = current_index as i32 + delta;

            // Check bounds
            if new_index >= 0 && new_index < children.len() as i32 {
                let widget = children.remove(current_index);
                children.insert(new_index as usize, widget);
                return true;
            }
            return false; // Can't move further in that direction
        }

        // Recurse into each child
        for child in children.iter_mut() {
            if move_widget_in_parent(child.as_mut(), widget_id, delta) {
                return true;
            }
        }
    }
    false
}

/// Remove a widget from the tree and return it (if found)
fn extract_widget_recursive(node: &mut dyn WidgetNode, widget_id: Uuid) -> Option<Box<dyn WidgetNode>> {
    if let Some(children) = node.children_mut() {
        // Check if any direct child matches
        if let Some(index) = children.iter().position(|c| c.id() == widget_id) {
            return Some(children.remove(index));
        }

        // Recurse into each child
        for child in children.iter_mut() {
            if let Some(found) = extract_widget_recursive(child.as_mut(), widget_id) {
                return Some(found);
            }
        }
    }
    None
}

/// Insert a widget as a child of the target container at the specified index
fn insert_widget_at(node: &mut dyn WidgetNode, target_id: Uuid, widget: Box<dyn WidgetNode>, index: usize) -> bool {
    if node.id() == target_id {
        if let Some(children) = node.children_mut() {
            let insert_at = index.min(children.len());
            children.insert(insert_at, widget);
            return true;
        }
        return false; // Target is not a container
    }

    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            if insert_widget_at(child.as_mut(), target_id, widget.clone_box(), index) {
                return true;
            }
        }
    }
    false
}

/// Check if a widget is a container (has children capability)
pub fn is_container(node: &dyn WidgetNode) -> bool {
    node.children().is_some()
}

/// Get the index of a widget within its parent's children
fn get_widget_index(node: &dyn WidgetNode, widget_id: Uuid) -> Option<usize> {
    if let Some(children) = node.children() {
        for (i, child) in children.iter().enumerate() {
            if child.id() == widget_id {
                return Some(i);
            }
        }
    }
    None
}

/// Get the parent ID of a widget
fn get_parent_id_recursive(node: &dyn WidgetNode, child_id: Uuid) -> Option<Uuid> {
    if let Some(children) = node.children() {
        // Check if any direct child matches
        if children.iter().any(|c| c.id() == child_id) {
            return Some(node.id());
        }

        // Recurse into children
        for child in children {
            if let Some(parent_id) = get_parent_id_recursive(child.as_ref(), child_id) {
                return Some(parent_id);
            }
        }
    }
    None
}

/// Find a node by ID (immutable version)
fn find_node_by_id(node: &dyn WidgetNode, target_id: Uuid) -> Option<&dyn WidgetNode> {
    if node.id() == target_id {
        return Some(node);
    }
    if let Some(children) = node.children() {
        for child in children {
            if let Some(found) = find_node_by_id(child.as_ref(), target_id) {
                return Some(found);
            }
        }
    }
    None
}

/// Check if a node contains a specific ID in its descendants
fn contains_id_recursive(node: &dyn WidgetNode, target_id: Uuid) -> bool {
    if node.id() == target_id {
        return true;
    }
    if let Some(children) = node.children() {
        for child in children {
            if contains_id_recursive(child.as_ref(), target_id) {
                return true;
            }
        }
    }
    false
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

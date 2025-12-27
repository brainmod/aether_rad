use aether_rad::model::{ProjectState, Variable, VariableType};
use aether_rad::widgets::{ButtonWidget, LabelWidget, VerticalLayout};
use aether_rad::compiler::Compiler;

#[test]
fn test_save_load_round_trip() {
    // Create a project with various widgets
    let mut root = VerticalLayout::default();

    let button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Test Button".to_string(),
        clicked_code: "self.counter += 1;".to_string(),
        bindings: std::collections::HashMap::new(),
    };

    let label = LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Test Label".to_string(),
        bindings: std::collections::HashMap::new(),
    };

    root.children.push(Box::new(button));
    root.children.push(Box::new(label));

    let mut project = ProjectState::new(Box::new(root));

    // Add a variable
    project.variables.insert(
        "counter".to_string(),
        Variable {
            name: "counter".to_string(),
            v_type: VariableType::Integer,
            value: "0".to_string(),
        },
    );

    // Serialize to JSON
    let json = serde_json::to_string(&project).expect("Failed to serialize");

    // Deserialize back
    let loaded: ProjectState = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify round-trip fidelity
    assert_eq!(project.variables.len(), loaded.variables.len());
    assert_eq!(
        project.variables.get("counter").unwrap().name,
        loaded.variables.get("counter").unwrap().name
    );

    // Verify widget structure
    assert_eq!(project.root_node.name(), loaded.root_node.name());
    assert_eq!(
        project.root_node.children().unwrap().len(),
        loaded.root_node.children().unwrap().len()
    );
}

#[test]
fn test_code_generation_valid_rust() {
    // Create a simple project
    let mut root = VerticalLayout::default();

    let button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Click Me".to_string(),
        clicked_code: "println!(\"Button clicked!\");".to_string(),
        bindings: std::collections::HashMap::new(),
    };

    root.children.push(Box::new(button));

    let mut project = ProjectState::new(Box::new(root));
    project.project_name = "test_app".to_string();

    // Generate code
    let app_rs = Compiler::generate_app_rs(&project);
    let main_rs = Compiler::generate_main_rs();
    let cargo_toml = Compiler::generate_cargo_toml("test_app");

    // Verify code contains expected elements
    assert!(app_rs.contains("struct MyApp"));
    assert!(app_rs.contains("impl MyApp") || app_rs.contains("impl Default for MyApp"));
    assert!(app_rs.contains("fn update"));

    assert!(main_rs.contains("fn main"));
    assert!(main_rs.contains("eframe"));

    assert!(cargo_toml.contains("[package]"));
    assert!(cargo_toml.contains("test_app"));
    assert!(cargo_toml.contains("egui"));

    // Verify button click code is injected
    assert!(app_rs.contains("Button clicked!"));
}

#[test]
fn test_widget_tree_manipulation() {
    let mut project = ProjectState::new(Box::new(VerticalLayout::default()));

    // Add widgets to root
    let button_id = uuid::Uuid::new_v4();
    let label_id = uuid::Uuid::new_v4();

    if let Some(children) = project.root_node.children_mut() {
        children.push(Box::new(ButtonWidget {
            id: button_id,
            text: "Button 1".to_string(),
            clicked_code: String::new(),
            bindings: std::collections::HashMap::new(),
        }));

        children.push(Box::new(LabelWidget {
            id: label_id,
            text: "Label 1".to_string(),
            bindings: std::collections::HashMap::new(),
        }));
    }

    // Test find_node
    let found_button = project.find_node_mut(button_id);
    assert!(found_button.is_some());
    assert_eq!(found_button.unwrap().name(), "Button");

    // Test delete_widget
    let delete_result = project.delete_widget(button_id);
    assert!(delete_result);

    // Verify deletion
    let find_deleted = project.find_node_mut(button_id);
    assert!(find_deleted.is_none());

    // Verify other widget still exists
    let found_label = project.find_node_mut(label_id);
    assert!(found_label.is_some());
    assert_eq!(found_label.unwrap().name(), "Label");
}

#[test]
fn test_widget_reordering() {
    let mut project = ProjectState::new(Box::new(VerticalLayout::default()));

    // Add three widgets
    let id1 = uuid::Uuid::new_v4();
    let id2 = uuid::Uuid::new_v4();
    let id3 = uuid::Uuid::new_v4();

    if let Some(children) = project.root_node.children_mut() {
        children.push(Box::new(LabelWidget {
            id: id1,
            text: "First".to_string(),
            bindings: std::collections::HashMap::new(),
        }));

        children.push(Box::new(LabelWidget {
            id: id2,
            text: "Second".to_string(),
            bindings: std::collections::HashMap::new(),
        }));

        children.push(Box::new(LabelWidget {
            id: id3,
            text: "Third".to_string(),
            bindings: std::collections::HashMap::new(),
        }));
    }

    // Move second widget down
    let move_result = project.move_widget_down(id2);
    assert!(move_result);

    // Verify order: First, Third, Second
    if let Some(children) = project.root_node.children() {
        assert_eq!(children[0].id(), id1);
        assert_eq!(children[1].id(), id3);
        assert_eq!(children[2].id(), id2);
    }

    // Move third widget up (now at index 1)
    let move_result = project.move_widget_up(id3);
    assert!(move_result);

    // Verify order: Third, First, Second
    if let Some(children) = project.root_node.children() {
        assert_eq!(children[0].id(), id3);
        assert_eq!(children[1].id(), id1);
        assert_eq!(children[2].id(), id2);
    }
}

#[test]
fn test_root_layout_type_switching() {
    let mut project = ProjectState::new(Box::new(VerticalLayout::default()));

    // Add some children
    if let Some(children) = project.root_node.children_mut() {
        children.push(Box::new(ButtonWidget::default()));
        children.push(Box::new(LabelWidget::default()));
    }

    let root_id = project.root_node.id();

    // Switch to horizontal layout
    project.set_root_layout_type("Horizontal Layout");
    assert_eq!(project.root_layout_type(), "Horizontal Layout");
    assert_eq!(project.root_node.id(), root_id); // ID should be preserved
    assert_eq!(project.root_node.children().unwrap().len(), 2); // Children preserved

    // Switch to grid layout
    project.set_root_layout_type("Grid Layout");
    assert_eq!(project.root_layout_type(), "Grid Layout");
    assert_eq!(project.root_node.id(), root_id);
    assert_eq!(project.root_node.children().unwrap().len(), 2);

    // Switch back to vertical
    project.set_root_layout_type("Vertical Layout");
    assert_eq!(project.root_layout_type(), "Vertical Layout");
    assert_eq!(project.root_node.id(), root_id);
    assert_eq!(project.root_node.children().unwrap().len(), 2);
}

#[test]
fn test_variable_management() {
    let mut project = ProjectState::new(Box::new(VerticalLayout::default()));

    // Add variables
    project.variables.insert(
        "counter".to_string(),
        Variable {
            name: "counter".to_string(),
            v_type: VariableType::Integer,
            value: "0".to_string(),
        },
    );

    project.variables.insert(
        "message".to_string(),
        Variable {
            name: "message".to_string(),
            v_type: VariableType::String,
            value: "Hello".to_string(),
        },
    );

    // Serialize and deserialize
    let json = serde_json::to_string(&project).unwrap();
    let loaded: ProjectState = serde_json::from_str(&json).unwrap();

    // Verify variables
    assert_eq!(loaded.variables.len(), 2);

    let counter = loaded.variables.get("counter").unwrap();
    assert_eq!(counter.value, "0");
    assert!(matches!(counter.v_type, VariableType::Integer));

    let message = loaded.variables.get("message").unwrap();
    assert_eq!(message.value, "Hello");
    assert!(matches!(message.v_type, VariableType::String));
}

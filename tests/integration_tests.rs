use aether_rad::model::{ProjectState, Variable, VariableType, WidgetEvent, Action};
use aether_rad::widgets::{ButtonWidget, LabelWidget, VerticalLayout, HorizontalLayout, GridLayout, CheckboxWidget, SliderWidget};
use aether_rad::compiler::Compiler;

#[test]
fn test_save_load_round_trip() {
    // Create a project with various widgets
    let mut root = VerticalLayout::default();

    let mut button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Test Button".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    };
    button.events.insert(
        WidgetEvent::Clicked,
        Action::Custom("self.counter += 1;".to_string()),
    );

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

    let mut button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Click Me".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    };
    button.events.insert(
        WidgetEvent::Clicked,
        Action::Custom("println!(\"Button clicked!\");".to_string()),
    );

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
            events: std::collections::HashMap::new(),
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

#[test]
fn test_deeply_nested_widget_structure() {
    // Create a deeply nested structure:
    // VerticalLayout -> HorizontalLayout -> GridLayout -> Button/Label/Checkbox

    // Create innermost widgets for the grid
    let button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Nested Button".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    };

    let label = LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Nested Label".to_string(),
        bindings: std::collections::HashMap::new(),
    };

    let checkbox = CheckboxWidget {
        id: uuid::Uuid::new_v4(),
        label: "Nested Checkbox".to_string(),
        checked: false,
        bindings: std::collections::HashMap::new(),
        events: std::collections::HashMap::new(),
    };

    let slider = SliderWidget {
        id: uuid::Uuid::new_v4(),
        min: 0.0,
        max: 100.0,
        value: 50.0,
        bindings: std::collections::HashMap::new(),
        events: std::collections::HashMap::new(),
    };

    // Create Grid containing the widgets
    let mut grid = GridLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        columns: 2,
        spacing: 8.0,
    };
    grid.children.push(Box::new(button));
    grid.children.push(Box::new(label));
    grid.children.push(Box::new(checkbox));
    grid.children.push(Box::new(slider));

    // Create Horizontal containing the Grid
    let mut horizontal = HorizontalLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        spacing: 10.0,
    };
    horizontal.children.push(Box::new(grid));

    // Add another widget beside the grid in horizontal
    horizontal.children.push(Box::new(LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Sibling Label".to_string(),
        bindings: std::collections::HashMap::new(),
    }));

    // Create Vertical (root) containing the Horizontal
    let mut vertical = VerticalLayout::default();
    vertical.children.push(Box::new(horizontal));

    // Add a top-level widget too
    vertical.children.push(Box::new(ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Top Level Button".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    }));

    // Create project with nested structure
    let project = ProjectState::new(Box::new(vertical));

    // Verify structure before serialization
    assert_eq!(project.root_node.name(), "Vertical Layout");
    assert_eq!(project.root_node.children().unwrap().len(), 2);

    // Get the horizontal layout
    let horizontal_ref = project.root_node.children().unwrap().get(0).unwrap();
    assert_eq!(horizontal_ref.name(), "Horizontal Layout");
    assert_eq!(horizontal_ref.children().unwrap().len(), 2);

    // Get the grid layout
    let grid_ref = horizontal_ref.children().unwrap().get(0).unwrap();
    assert_eq!(grid_ref.name(), "Grid Layout");
    assert_eq!(grid_ref.children().unwrap().len(), 4);

    // Serialize and deserialize
    let json = serde_json::to_string(&project).expect("Failed to serialize nested structure");
    let loaded: ProjectState = serde_json::from_str(&json).expect("Failed to deserialize nested structure");

    // Verify structure after deserialization
    assert_eq!(loaded.root_node.name(), "Vertical Layout");
    assert_eq!(loaded.root_node.children().unwrap().len(), 2);

    let loaded_horizontal = loaded.root_node.children().unwrap().get(0).unwrap();
    assert_eq!(loaded_horizontal.name(), "Horizontal Layout");
    assert_eq!(loaded_horizontal.children().unwrap().len(), 2);

    let loaded_grid = loaded_horizontal.children().unwrap().get(0).unwrap();
    assert_eq!(loaded_grid.name(), "Grid Layout");
    assert_eq!(loaded_grid.children().unwrap().len(), 4);

    // Verify the grid children types
    let grid_children = loaded_grid.children().unwrap();
    assert_eq!(grid_children[0].name(), "Button");
    assert_eq!(grid_children[1].name(), "Label");
    assert_eq!(grid_children[2].name(), "Checkbox");
    assert_eq!(grid_children[3].name(), "Slider");
}

#[test]
fn test_nested_codegen_produces_valid_code() {
    // Create nested structure
    let mut grid = GridLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        columns: 2,
        spacing: 8.0,
    };
    grid.children.push(Box::new(ButtonWidget::default()));
    grid.children.push(Box::new(LabelWidget::default()));

    let mut horizontal = HorizontalLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        spacing: 10.0,
    };
    horizontal.children.push(Box::new(grid));

    let mut vertical = VerticalLayout::default();
    vertical.children.push(Box::new(horizontal));

    let mut project = ProjectState::new(Box::new(vertical));
    project.project_name = "nested_test".to_string();

    // Generate code
    let app_rs = Compiler::generate_app_rs(&project);

    // Verify nested structure is present in generated code
    // GridLayout generates nested vertical/horizontal layouts, not egui::Grid
    assert!(app_rs.contains("ui.vertical"), "Should contain vertical layout");
    assert!(app_rs.contains("ui.horizontal"), "Should contain horizontal layout");
    // The button generates an egui button widget
    assert!(app_rs.contains("ui.button") || app_rs.contains(".clicked()"), "Should contain button interaction");
}

#[test]
fn test_three_level_nesting_serialization() {
    // Create a three-level nesting: Vertical -> Horizontal -> Vertical -> Button
    let button = ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Deep Button".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    };

    let mut inner_vertical = VerticalLayout::default();
    inner_vertical.children.push(Box::new(button));

    let mut horizontal = HorizontalLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        spacing: 10.0,
    };
    horizontal.children.push(Box::new(inner_vertical));

    let mut outer_vertical = VerticalLayout::default();
    outer_vertical.children.push(Box::new(horizontal));

    let project = ProjectState::new(Box::new(outer_vertical));

    // Serialize
    let json = serde_json::to_string(&project).expect("Failed to serialize");

    // Deserialize
    let loaded: ProjectState = serde_json::from_str(&json).expect("Failed to deserialize");

    // Navigate to the deeply nested button
    let loaded_horizontal = loaded.root_node.children().unwrap().get(0).unwrap();
    let loaded_inner_vertical = loaded_horizontal.children().unwrap().get(0).unwrap();
    let loaded_button = loaded_inner_vertical.children().unwrap().get(0).unwrap();

    assert_eq!(loaded_button.name(), "Button");
}

#[test]
fn test_codegen_compiles_successfully() {
    use std::fs;
    use std::process::Command;

    // Create a project with various widgets
    let mut root = VerticalLayout::default();

    // Add various widgets
    root.children.push(Box::new(LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Hello World".to_string(),
        bindings: std::collections::HashMap::new(),
    }));

    root.children.push(Box::new(ButtonWidget {
        id: uuid::Uuid::new_v4(),
        text: "Click Me".to_string(),
        events: std::collections::HashMap::new(),
        bindings: std::collections::HashMap::new(),
    }));

    root.children.push(Box::new(CheckboxWidget {
        id: uuid::Uuid::new_v4(),
        label: "Enable feature".to_string(),
        checked: false,
        bindings: std::collections::HashMap::new(),
        events: std::collections::HashMap::new(),
    }));

    root.children.push(Box::new(SliderWidget {
        id: uuid::Uuid::new_v4(),
        min: 0.0,
        max: 100.0,
        value: 50.0,
        bindings: std::collections::HashMap::new(),
        events: std::collections::HashMap::new(),
    }));

    // Add a nested horizontal layout
    let mut horizontal = HorizontalLayout {
        id: uuid::Uuid::new_v4(),
        children: Vec::new(),
        spacing: 10.0,
    };
    horizontal.children.push(Box::new(LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Left".to_string(),
        bindings: std::collections::HashMap::new(),
    }));
    horizontal.children.push(Box::new(LabelWidget {
        id: uuid::Uuid::new_v4(),
        text: "Right".to_string(),
        bindings: std::collections::HashMap::new(),
    }));
    root.children.push(Box::new(horizontal));

    let mut project = ProjectState::new(Box::new(root));
    project.project_name = "codegen_test_project".to_string();

    // Generate code
    let app_rs = Compiler::generate_app_rs(&project);
    let main_rs = Compiler::generate_main_rs();
    let cargo_toml = Compiler::generate_cargo_toml(&project.project_name);

    // Create temp directory
    let temp_dir = std::env::temp_dir().join("aether_rad_codegen_test");
    let src_dir = temp_dir.join("src");

    // Clean up any previous test run
    let _ = fs::remove_dir_all(&temp_dir);

    // Create directories
    fs::create_dir_all(&src_dir).expect("Failed to create temp src directory");

    // Write files
    fs::write(temp_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");
    fs::write(src_dir.join("main.rs"), main_rs).expect("Failed to write main.rs");
    fs::write(src_dir.join("app.rs"), app_rs).expect("Failed to write app.rs");

    // Run cargo check
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to run cargo check");

    // Print stderr for debugging if check fails
    if !output.status.success() {
        eprintln!("cargo check stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        eprintln!("cargo check stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);

    // Assert success
    assert!(output.status.success(), "Generated code should compile successfully");
}

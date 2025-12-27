mod model;
mod widgets;

use model::ProjectState;
use model::WidgetNode;
use widgets::{ButtonWidget, VerticalLayout}; // Import the trait to use Box<dyn WidgetNode>

fn main() {
    println!("Initializing Aether Kernel (Recursion Test)...");

    // 1. Create Children
    let button1 = Box::new(ButtonWidget {
        text: "Top Button".to_string(),
        clicked_code: "println!(\"Top\");".to_string(),
    });

    let button2 = Box::new(ButtonWidget {
        text: "Bottom Button".to_string(),
        clicked_code: "println!(\"Bottom\");".to_string(),
    });

    // 2. Create Container and add children
    let mut layout = VerticalLayout::default();
    layout.children.push(button1);
    layout.children.push(button2);

    // 3. Wrap in Project State
    // The Root is now the Layout, not a single button
    let project = ProjectState::new(Box::new(layout));

    // 4. Serialize (Save)
    let json_output = project.to_json();
    println!("--- Serialized Tree (JSON) ---");
    println!("{}", json_output);

    // 5. Deserialize (Load)
    let loaded_project: ProjectState =
        serde_json::from_str(&json_output).expect("Failed to deserialize tree");

    // 6. Inspect the Tree
    let root = loaded_project.root_node;
    println!("\n--- Inspection ---");
    println!("Root Type: {}", root.name());

    if let Some(children) = root.children() {
        println!("Child Count: {}", children.len());
        println!("First Child: {}", children[0].name());
        // In a real test, we would check children[0].text == "Top Button"
    } else {
        println!("Error: Root has no children!");
    }
}

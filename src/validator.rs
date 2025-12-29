use crate::compiler::Compiler;
use crate::model::ProjectState;
use std::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ValidationStatus {
    NotRun,
    Checking,
    Success,
    Failed(String),
}

impl ValidationStatus {
    pub fn display_text(&self) -> String {
        match self {
            ValidationStatus::NotRun => "Click 'Check Code' to validate compilation".to_string(),
            ValidationStatus::Checking => "Cargo check in progress...".to_string(),
            ValidationStatus::Success => "✓ Code compiles successfully!".to_string(),
            ValidationStatus::Failed(err) => format!("✗ Compilation failed:\n{}", err),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, ValidationStatus::Success)
    }

    pub fn is_checking(&self) -> bool {
        matches!(self, ValidationStatus::Checking)
    }
}

pub struct CodeValidator;

impl CodeValidator {
    /// Validate that generated code compiles by running cargo check
    pub fn validate(project_state: &ProjectState) -> Result<String, String> {
        // 1. Perform logical validation on the widget tree
        let mut logical_errors = Vec::new();
        validate_node_recursive(project_state.root_node.as_ref(), &project_state.variables, &mut logical_errors);

        if !logical_errors.is_empty() {
            return Err(format!("Logical Validation Failed:\n- {}", logical_errors.join("\n- ")));
        }

        // 2. Run Cargo Check (slow but thorough)
        // Create a temporary directory
        let temp_dir = std::env::temp_dir().join(format!("aether_check_{}", uuid::Uuid::new_v4()));

        // Create directory structure
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;

        let src_dir = temp_dir.join("src");
        std::fs::create_dir(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

        // Write Cargo.toml
        let cargo_toml = Compiler::generate_cargo_toml(&project_state.project_name);
        let cargo_path = temp_dir.join("Cargo.toml");
        std::fs::write(&cargo_path, cargo_toml)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

        // Write main.rs
        let main_rs = Compiler::generate_main_rs();
        std::fs::write(src_dir.join("main.rs"), main_rs)
            .map_err(|e| format!("Failed to write main.rs: {}", e))?;

        // Write app.rs
        let app_rs = Compiler::generate_app_rs(project_state);
        std::fs::write(src_dir.join("app.rs"), app_rs)
            .map_err(|e| format!("Failed to write app.rs: {}", e))?;

        // Run cargo check
        let output = Command::new("cargo")
            .args(&["check", "--manifest-path"])
            .arg(&cargo_path)
            .output()
            .map_err(|e| format!("Failed to run cargo check: {}", e))?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Check result
        if output.status.success() {
            Ok("Compilation check passed".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("{}\n{}", stdout, stderr))
        }
    }
}

fn validate_node_recursive(
    node: &dyn crate::model::WidgetNode,
    variables: &std::collections::HashMap<String, crate::model::Variable>,
    errors: &mut Vec<String>,
) {
    // Validate current node
    errors.extend(node.validate(variables));

    // Recurse into children
    if let Some(children) = node.children() {
        for child in children {
            validate_node_recursive(child.as_ref(), variables, errors);
        }
    }
}

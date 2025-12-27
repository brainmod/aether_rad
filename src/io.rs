/// File I/O utilities with WASM support
/// Provides conditional implementations for native and WASM targets

use std::path::PathBuf;

/// Show a folder picker dialog
/// Returns the selected folder path
#[cfg(target_arch = "wasm32")]
pub fn pick_folder() -> Option<PathBuf> {
    // WASM doesn't have native folder picker
    // Users can manually input paths in a text field instead
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

/// Show a file picker dialog for a specific file type
/// Returns the selected file path
#[cfg(target_arch = "wasm32")]
pub fn pick_file(filter_name: &str) -> Option<PathBuf> {
    // WASM doesn't have native file picker
    // Users can manually input paths in a text field instead
    let _ = filter_name;
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_file(filter_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(filter_name, &["png", "jpg", "jpeg", "gif", "bmp"])
        .pick_file()
}

/// Save dialog for project files
#[cfg(target_arch = "wasm32")]
pub fn save_file(default_name: &str) -> Option<PathBuf> {
    // WASM doesn't have native save dialog
    // Default to current directory with default name
    let _ = default_name;
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(default_name)
        .save_file()
}

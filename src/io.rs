/// File I/O utilities with WASM support
/// Provides conditional implementations for native and WASM targets

use std::path::PathBuf;

// ============== Native Implementations ==============

/// Show a folder picker dialog
/// Returns the selected folder path
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

/// Show a file picker dialog for a specific file type
/// Returns the selected file path
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_file(filter_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(filter_name, &["png", "jpg", "jpeg", "gif", "bmp"])
        .pick_file()
}

/// Save dialog for project files
#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(default_name)
        .save_file()
}

/// Write content to a file
#[cfg(not(target_arch = "wasm32"))]
pub fn write_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

/// Read content from a file
#[cfg(not(target_arch = "wasm32"))]
pub fn read_file(path: &std::path::Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

/// Create a directory and all parent directories
#[cfg(not(target_arch = "wasm32"))]
pub fn create_dir_all(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

// ============== WASM Implementations ==============

#[cfg(target_arch = "wasm32")]
pub fn pick_folder() -> Option<PathBuf> {
    // WASM doesn't have native folder picker
    // Users can manually input paths in a text field instead
    None
}

#[cfg(target_arch = "wasm32")]
pub fn pick_file(_filter_name: &str) -> Option<PathBuf> {
    // WASM doesn't have synchronous file picker
    // Use the async version or file input element instead
    None
}

#[cfg(target_arch = "wasm32")]
pub fn save_file(_default_name: &str) -> Option<PathBuf> {
    // WASM doesn't have native save dialog
    None
}

#[cfg(target_arch = "wasm32")]
pub fn write_file(_path: &std::path::Path, _content: &str) -> std::io::Result<()> {
    // WASM can't write to filesystem directly
    // Use local storage or download instead
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Direct file writing not supported in WASM",
    ))
}

#[cfg(target_arch = "wasm32")]
pub fn read_file(_path: &std::path::Path) -> std::io::Result<String> {
    // WASM can't read from filesystem directly
    // Use file input or fetch instead
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Direct file reading not supported in WASM",
    ))
}

#[cfg(target_arch = "wasm32")]
pub fn create_dir_all(_path: &std::path::Path) -> std::io::Result<()> {
    // WASM doesn't have a filesystem
    Ok(())
}

// ============== WASM Browser Storage ==============

/// Save project data to browser localStorage
#[cfg(target_arch = "wasm32")]
pub fn save_to_local_storage(key: &str, data: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .set_item(key, data)
        .map_err(|_| "Failed to save to localStorage".to_string())
}

/// Load project data from browser localStorage
#[cfg(target_arch = "wasm32")]
pub fn load_from_local_storage(key: &str) -> Result<Option<String>, String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .get_item(key)
        .map_err(|_| "Failed to read from localStorage".to_string())
}

/// Delete project data from browser localStorage
#[cfg(target_arch = "wasm32")]
pub fn remove_from_local_storage(key: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .remove_item(key)
        .map_err(|_| "Failed to remove from localStorage".to_string())
}

/// List all project keys in localStorage
#[cfg(target_arch = "wasm32")]
pub fn list_local_storage_keys(prefix: &str) -> Result<Vec<String>, String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;

    let length = storage.length().map_err(|_| "Failed to get storage length")?;
    let mut keys = Vec::new();

    for i in 0..length {
        if let Ok(Some(key)) = storage.key(i) {
            if key.starts_with(prefix) {
                keys.push(key);
            }
        }
    }

    Ok(keys)
}

/// Trigger a file download in the browser
#[cfg(target_arch = "wasm32")]
pub fn trigger_download(filename: &str, content: &str, mime_type: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;

    // Create a blob from the content
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let mut blob_options = web_sys::BlobPropertyBag::new();
    blob_options.type_(mime_type);

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)
        .map_err(|_| "Failed to create blob")?;

    // Create a URL for the blob
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Failed to create object URL")?;

    // Create an anchor element and trigger download
    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| "Failed to create anchor element")?
        .dyn_into()
        .map_err(|_| "Failed to cast to anchor element")?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Cleanup
    web_sys::Url::revoke_object_url(&url).map_err(|_| "Failed to revoke object URL")?;

    Ok(())
}

// Native stubs for browser storage (no-ops on native)
#[cfg(not(target_arch = "wasm32"))]
pub fn save_to_local_storage(_key: &str, _data: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_local_storage(_key: &str) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove_from_local_storage(_key: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn list_local_storage_keys(_prefix: &str) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_download(_filename: &str, _content: &str, _mime_type: &str) -> Result<(), String> {
    Ok(())
}

// ============== Project Storage Constants ==============

/// Key prefix for project storage
pub const PROJECT_KEY_PREFIX: &str = "aether_project_";

/// Get the storage key for a project
pub fn project_storage_key(name: &str) -> String {
    format!("{}{}", PROJECT_KEY_PREFIX, name)
}

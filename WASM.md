# WASM Build Configuration for Aether RAD

This document describes the WASM support added to Aether RAD, enabling cross-platform deployment on the web.

## Overview

Aether RAD now supports compilation to WebAssembly (WASM), allowing the application to run in web browsers. This is achieved through conditional compilation and platform-specific implementations.

## Configuration

### Cargo.toml

The `Cargo.toml` has been updated with WASM support:

```toml
[features]
default = ["native"]
native = []
wasm = ["web-sys", "wasm-bindgen"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlElement"] }
wasm-bindgen = "0.2"
```

### Platform Detection

The codebase uses Rust's `#[cfg(target_arch = "wasm32")]` attribute for conditional compilation:
- WASM target: Uses web-sys and wasm-bindgen
- Native target: Uses standard file dialogs (rfd crate)

## File Dialog Handling

### Native Implementation
- Uses `rfd` crate for native file/folder dialogs
- Full support for:
  - Pick folder (export)
  - Pick file (load projects)
  - Save file (save projects)
  - Image file picker (for image assets)

### WASM Implementation
- File dialogs return `None` in WASM builds
- Users can manually input file paths through text fields
- The application gracefully handles missing file dialog support
- Future improvements could use the File API or drag-and-drop

## Building for WASM

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
# or use trunk: cargo install trunk
```

### Build Commands

**Using wasm-pack:**
```bash
wasm-pack build --target web
```

**Using trunk:**
```bash
cargo install trunk
trunk serve  # For development
trunk build  # For production
```

### Output
- WASM binaries in `pkg/` directory (wasm-pack) or `dist/` (trunk)
- Can be deployed to any static hosting service

## HTML Template

An `index.html` template would be needed for web deployment:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Aether RAD</title>
    <style>
        body { margin: 0; }
        canvas { display: block; }
    </style>
</head>
<body>
    <script type="module">
        import init from './pkg/aether_rad.js';
        init();
    </script>
</body>
</html>
```

## Limitations on WASM

1. **No Native File Dialogs**: Users must manually enter file paths or use text fields
2. **File System Access**: Limited by browser sandboxing
3. **Project Export**: Full export may not work - projects can be saved as JSON blobs
4. **Performance**: Some features may run slower than native

## Future Improvements

1. **File API Integration**: Use browser File API for file selection
2. **IndexedDB Storage**: For persistent project storage
3. **Drag & Drop**: Support dragging files into the canvas
4. **Service Workers**: For offline support
5. **WebGL Canvas**: For better graphics performance

## Testing WASM Build

```bash
# Build for WASM
wasm-pack build --target web

# Serve locally for testing
python -m http.server 8000

# Visit http://localhost:8000 in your browser
```

## Known Issues

- File path operations use `PathBuf` which may behave differently on WASM
- Export functionality limited without proper file API integration
- Some dependencies may not have WASM-compatible implementations

## References

- [wasm-pack Book](https://rustwasm.org/docs/wasm-pack/)
- [egui WASM Support](https://github.com/emilk/egui#web)
- [Web API Documentation](https://developer.mozilla.org/en-US/docs/Web/API)

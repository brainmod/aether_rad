# Aether RAD

A visual UI builder for Rust/egui applications. Design your UI visually, then generate production-ready Rust code.

## Overview

Aether RAD bridges the gap between **Immediate Mode** GUI rendering (egui) and **Retained Mode** visual editing (Qt Designer style) using a **Shadow Object Model (SOM)** architecture.

## Features

- **Visual UI Designer** - Drag-and-drop widget placement with real-time preview
- **Property Inspector** - Edit widget properties with immediate visual feedback
- **Data Binding** - Bind widget properties to application state variables
- **Event System** - Attach actions to widget events (click, change, etc.)
- **Code Generation** - Export complete, compilable Rust/egui projects
- **Project Templates** - Start from Empty, Counter App, Form, or Dashboard templates
- **Undo/Redo** - 50-step history with Ctrl+Z/Y shortcuts
- **Multi-Selection** - Ctrl+click to select multiple widgets
- **Light/Dark Theme** - Toggle between themes
- **Live Code Preview** - See generated code update in real-time
- **Code Validation** - Run `cargo check` on generated code

## Current Status

**Version 2.0 - All Phases Complete**

All five architectural phases from the development plan have been implemented, plus numerous extended features.

### Widget Library (15 Widgets)

| Category | Widgets |
|----------|---------|
| **Layouts** | VerticalLayout, HorizontalLayout, GridLayout |
| **Inputs** | Button, TextEdit, Checkbox, Slider, ComboBox |
| **Display** | Label, ProgressBar, Image, Separator, Spinner, Hyperlink, ColorPicker |

### Phase Completion

| Phase | Component | Status |
|-------|-----------|--------|
| 1 | Kernel (SOM, Serialization) | Complete |
| 2 | Shell (Docking Layout) | Complete |
| 3 | Interactive Canvas (D&D, Gizmos) | Complete |
| 4 | Logic & Data Binding | Complete |
| 5 | Compiler (Code Generation) | Complete |

## Quick Start

```bash
# Clone and build
git clone <repo-url>
cd aether_rad
cargo run

# Using the visual editor:
# 1. Drag widgets from Palette (left) to Canvas (center)
# 2. Select widgets to edit properties in Inspector (right)
# 3. Add variables in Variables panel for data binding
# 4. Attach events to widgets (click actions, change handlers)
# 5. Click "Export Project" in Output panel to generate code
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |
| Ctrl+C | Copy widget |
| Ctrl+V | Paste widget |
| Delete | Delete selected widget |
| Arrow Up/Down | Navigate hierarchy |
| Escape | Clear selection |
| Ctrl+Click | Toggle multi-selection |

## Architecture

```
src/
├── main.rs       # Entry point
├── app.rs        # Main application loop, undo/redo, copy/paste
├── model.rs      # SOM: WidgetNode trait, ProjectState, Variable, Action
├── widgets.rs    # 15 widget implementations
├── ui.rs         # Panel rendering (8 panels)
├── compiler.rs   # Code generation with prettyplease formatting
├── theme.rs      # Light/Dark theme configuration
├── syntax.rs     # Syntax highlighting for code preview
├── validator.rs  # Cargo check integration
├── io.rs         # Platform-agnostic file I/O
└── lib.rs        # Library exports

tests/
└── integration_tests.rs  # Serialization, codegen, manipulation tests
```

## Dependencies

- **egui/eframe** - Immediate mode GUI framework
- **egui_dock** - IDE-style docking layout
- **serde/typetag** - Polymorphic serialization
- **quote/proc-macro2** - Rust code generation
- **prettyplease** - Code formatting
- **syntect** - Syntax highlighting
- **rfd** - Native file dialogs

## WASM Support

Aether RAD includes preliminary WASM support. See [WASM.md](WASM.md) for build instructions and limitations.

```bash
# Build for WASM
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve  # Development
trunk build  # Production
```

## Documentation

- **CLAUDE.md** - Development guide with bite-sized tasks for contributors
- **WASM.md** - WASM build configuration and limitations
- **Rust RAD Utility Development Plan.md** - Full architectural blueprint

## License

See [LICENSE](LICENSE) for details.

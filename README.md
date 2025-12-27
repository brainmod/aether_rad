# Aether RAD

A native Rust Rapid Application Development (RAD) utility for building egui applications visually.

## Overview

Aether bridges the gap between **Immediate Mode** GUI rendering (egui) and **Retained Mode** visual editing (Qt Designer style) using a **Shadow Object Model (SOM)** architecture. Design your UI visually, then generate production-ready Rust code.

## Features

- **Visual UI Designer** - Drag-and-drop widget placement with real-time preview
- **Property Inspector** - Edit widget properties with immediate visual feedback
- **Data Binding** - Bind widget properties to application state variables
- **Code Generation** - Export complete, compilable Rust/egui projects
- **Save/Load Projects** - Persist designs as JSON for iterative development

## Current Status

**Version 1.0 Prototype Complete** - All five architectural phases implemented:

| Phase | Component | Status |
|-------|-----------|--------|
| 1 | Kernel (SOM, Serialization) | Complete |
| 2 | Shell (Docking Layout) | Complete |
| 3 | Interactive Canvas (D&D, Gizmos) | Complete |
| 4 | Logic & Data Binding | Complete |
| 5 | Compiler (Code Generation) | Complete |

## Widget Library

**Layouts:** VerticalLayout, HorizontalLayout
**Controls:** Button, Label, TextEdit, Checkbox, Slider

## Quick Start

```bash
# Clone and build
git clone <repo-url>
cd aether_rad
cargo run

# Use the visual editor:
# 1. Drag widgets from Palette (left) to Canvas (center)
# 2. Select widgets to edit properties in Inspector (right)
# 3. Add variables in Variables panel for data binding
# 4. Click "Generate Code" in Output panel to export
```

## Architecture

```
src/
├── main.rs       # Entry point
├── app.rs        # Main application loop, docking setup
├── model.rs      # SOM: WidgetNode trait, ProjectState, Variable
├── widgets.rs    # Concrete widget implementations
├── ui.rs         # Panel rendering (Canvas, Palette, Inspector, etc.)
└── compiler.rs   # Code generation (Cargo.toml, main.rs, app.rs)
```

## Dependencies

- **egui/eframe** - Immediate mode GUI framework
- **egui_dock** - IDE-style docking layout
- **serde/typetag** - Polymorphic serialization
- **quote/proc-macro2** - Rust code generation
- **rfd** - Native file dialogs

## Documentation

- `CLAUDE.md` - Development guide with bite-sized tasks for contributors
- `GEMINI.md` - Development status and progress log
- `Rust RAD Utility Development Plan.md` - Full architectural blueprint

## License

See [LICENSE](LICENSE) for details.

# Project Aether: Development Status & Roadmap

**Based on Architectural Blueprint:** "Egui Forge"
**Current Phase:** ✅ V1.5 Prototype Complete (Core features + polish)

## 1. Executive Summary

Aether is a native Rust Rapid Application Development (RAD) utility. It bridges the gap between **Immediate Mode** rendering (egui) and **Retained Mode** editing (Qt Designer style) using a **Shadow Object Model (SOM)**.

We have successfully implemented the full prototype with extended features including undo/redo, copy/paste, keyboard navigation, live code preview, and project export.

## 2. Architectural Pillars

### The Shadow Object Model (SOM)

- **Concept:** A persistent data structure representing the UI state, distinct from the ephemeral frame rendering.
- **Implementation:** The `ProjectState` struct holds a root `Box<dyn WidgetNode>` and a `HashMap<String, Variable>`.
- **Polymorphism:** Solved using `typetag` to serialize dynamic trait objects.

### The "Edit Mode" Rendering

- **Concept:** Widgets render themselves. In the editor, they are wrapped in interaction logic (Gizmos, Drag & Drop).
- **Status:** Active. Widgets support visual editing, selection outlines, resize handles, and property binding.

## 3. Progress Log

### ✅ Phase 1: The Kernel (Completed)

- [x] **WidgetNode Trait:** Defined with `render_editor`, `inspect`, `codegen`, and `clone_box`.
- [x] **Serialization Engine:** Integrated `serde` + `typetag`.
- [x] **Project State:** Root container for application definition.

### ✅ Phase 2: The Shell (Completed)

- [x] **Workspace Layout:** Integrated `egui_dock` (v0.18) for resizable panels.
- [x] **Panel Architecture:** `AetherTabViewer` with 7 panels (Canvas, Palette, Hierarchy, Inspector, Variables, Output, CodePreview).
- [x] **The Feedback Loop:** Connecting Inspector to Canvas.

### ✅ Phase 3: The Interactive Canvas (Completed)

- [x] **Hierarchy Tree Walker:** Recursive visual tree in the "Hierarchy" panel.
- [x] **Selection System:** `HashSet<Uuid>` based selection.
- [x] **Gizmos:** Selection outlines + resize handles for Image widget.
- [x] **Drag & Drop:** `dnd_drag_source` (Palette) -> `dnd_drop_zone` (Containers).
- [x] **Keyboard Navigation:** Arrow keys in Hierarchy, Escape to deselect.

### ✅ Phase 4: Logic & Data Binding (Completed)

- [x] **Variable Store:** `Variable` struct (String, Integer, Float, Boolean).
- [x] **Variables Panel:** UI for adding/removing/editing state variables.
- [x] **Data Binding:** Inspector allows binding properties to variables.
- [x] **Event Actions:** Button `clicked_code` with syntax validation.

### ✅ Phase 5: The Compiler (Completed)

- [x] **Codegen Strategy:** `quote!` macros generating ASTs.
- [x] **Scaffolding:** Generators for `Cargo.toml`, `main.rs`, and `app.rs`.
- [x] **Binding Support:** Generated code references variables (`self.var_name`).
- [x] **Export to Disk:** File picker + write project files.
- [x] **Live Code Preview:** Real-time code generation display.
- [x] **Custom Project Names:** Editable in Output panel.

### ✅ Extended Features (Completed)

- [x] **Undo/Redo System:** 50-step history with Ctrl+Z/Y shortcuts.
- [x] **Widget Deletion:** Delete key + button in Inspector.
- [x] **Widget Reordering:** Move Up/Down buttons in Inspector.
- [x] **Copy/Paste:** Ctrl+C/V with UUID regeneration.
- [x] **Root Layout Switching:** Change between Vertical/Horizontal/Grid layouts.
- [x] **Integration Tests:** Save/load round-trip, codegen validation, manipulation tests.

### ✅ Widget Library (11 Widgets)

| Widget | Status | Bindings | Events |
|--------|--------|----------|--------|
| Button | ✅ | text | clicked_code |
| Label | ✅ | text | - |
| Text Edit | ✅ | value | - |
| Checkbox | ✅ | checked | - |
| Slider | ✅ | value | - |
| Progress Bar | ✅ | value | - |
| ComboBox | ✅ | selected | - |
| Image | ✅ | - | - |
| Vertical Layout | ✅ | - | - |
| Horizontal Layout | ✅ | - | - |
| Grid Layout | ✅ | - | - |

## 4. Future Roadmap (V2)

### 🎨 Code Quality & Output Polish
- **Pretty-Print Output:** Use `prettyplease` to format generated code.
- **Syntax Highlighting:** Add highlighting to Code Preview panel.
- **Validate Compilation:** Run `cargo check` on generated code.

### ⚡ Event System Expansion
- **Multi-Event Support:** Add `changed`, `hovered`, `double_clicked` events.
- **Standard Actions:** Pre-built actions (Increment Variable, Set Variable).

### 🧱 Widget Expansion
- **Separator Widget:** Visual separator for layouts.
- **Spinner Widget:** Loading indicator.
- **ColorPicker Widget:** Color selection.
- **Hyperlink Widget:** Clickable links.
- **Window Container:** Create popup windows.

### 🖱️ Hierarchy & Canvas Improvements
- **Re-enable Hierarchy DnD:** Drag-and-drop reordering in tree.
- **Canvas Zoom/Pan:** Navigate large designs.
- **Multi-Selection:** Ctrl+click, Shift+click support.

### 🌐 WASM & Cross-Platform
- **WASM Configuration:** Enable web deployment.
- **Project Templates:** Counter App, Form, Dashboard starters.

### 📦 Asset Management
- **Asset Manager:** Centralized image/font registry.
- **Asset Panel:** Import and manage assets.

## 5. Technical Reference

### Key Files
- `src/model.rs`: SOM, `WidgetNode`, `ProjectState`, `Variable`.
- `src/widgets.rs`: Concrete widgets & Codegen logic.
- `src/ui.rs`: Docking layout (`AetherTabViewer`) and panel rendering.
- `src/compiler.rs`: Code generation logic.
- `src/app.rs`: Main runtime loop, undo/redo, copy/paste.
- `tests/integration_tests.rs`: Automated testing.

### Dependencies
- `egui` / `eframe`: GUI framework
- `egui_dock`: Docking layout
- `serde` / `serde_json` / `typetag`: Serialization
- `quote` / `proc-macro2`: Code generation
- `uuid`: Widget identification
- `rfd`: File dialogs

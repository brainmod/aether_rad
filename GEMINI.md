# Project Aether: Development Status & Roadmap

**Based on Architectural Blueprint:** "Egui Forge"
**Current Phase:** ✅ Version 1.0 Prototype Complete (Phases 1-5 finished).

## 1. Executive Summary

Aether is a native Rust Rapid Application Development (RAD) utility. It bridges the gap between **Immediate Mode** rendering (egui) and **Retained Mode** editing (Qt Designer style) using a **Shadow Object Model (SOM)**.

We have successfully implemented the full prototype, enabling users to visually design a UI, define state variables, bind properties, and generate valid Rust code.

## 2. Architectural Pillars

### The Shadow Object Model (SOM)

- **Concept:** A persistent data structure representing the UI state, distinct from the ephemeral frame rendering.
- **Implementation:** The `ProjectState` struct holds a root `Box<dyn WidgetNode>` and a `HashMap<String, Variable>`.
- **Polymorphism:** Solved using `typetag` to serialize dynamic trait objects.

### The "Edit Mode" Rendering

- **Concept:** Widgets render themselves. In the editor, they are wrapped in interaction logic (Gizmos, Drag & Drop).
- **Status:** Active. Widgets support visual editing, selection outlines, and property binding.

## 3. Progress Log

### ✅ Phase 1: The Kernel (Completed)

- [x] **WidgetNode Trait:** Defined with `render_editor`, `inspect`, and `codegen`.
- [x] **Serialization Engine:** Integrated `serde` + `typetag`.
- [x] **Project State:** Root container for application definition.

### ✅ Phase 2: The Shell (Completed)

- [x] **Workspace Layout:** Integrated `egui_dock` (v0.18) for resizable panels.
- [x] **Panel Architecture:** `AetherTabViewer` with Context Injection.
- [x] **The Feedback Loop:** Connecting Inspector to Canvas.

### ✅ Phase 3: The Interactive Canvas (Completed)

- [x] **Hierarchy Tree Walker:** Recursive visual tree in the "Hierarchy" panel.
- [x] **Selection System:** `HashSet<Uuid>` based selection.
- [x] **Gizmos:** Selection outlines (`StrokeKind::Outside`).
- [x] **Drag & Drop:** `dnd_drag_source` (Palette) -> `dnd_drop_zone` (Containers).

### ✅ Phase 4: Logic & Data Binding (Completed)

- [x] **Variable Store:** `Variable` struct (String, Integer, Float, Boolean).
- [x] **Variables Panel:** UI for adding/removing/editing state variables.
- [x] **Data Binding:** Inspector allows binding properties (e.g., Button Text) to variables.

### ✅ Phase 5: The Compiler (Completed)

- [x] **Codegen Strategy:** `quote!` macros generating ASTs.
- [x] **Scaffolding:** Generators for `Cargo.toml`, `main.rs`, and `app.rs`.
- [x] **Binding Support:** Generated code references variables (`self.var_name`) instead of literals.
- [x] **UI Integration:** "Generate Code" button in Output panel (stdout preview).

## 4. Future Roadmap (Post-V1)

### 💾 Persistence & I/O
- **Save/Load:** Implement File Dialogs to save `ProjectState` to JSON files.
- **Real Compilation:** Write generated code to a temporary Cargo project and run `cargo build`.

### 🛠 Widget Expansion
- **Text Inputs:** Add `Label` and `LineEdit` widgets.
- **Layouts:** Add `HorizontalLayout` and `Grid`.

### ⚡ UX Refinement
- **Reordering:** Support Drag & Drop *within* the Hierarchy/Canvas to reorder children.
- **Undo/Redo:** Implement command history for `ProjectState`.

## 5. Technical Reference

### Key Files
- `src/model.rs`: SOM, `WidgetNode`, `ProjectState`, `Variable`.
- `src/widgets.rs`: Concrete widgets (`ButtonWidget`, `VerticalLayout`) & Codegen logic.
- `src/ui.rs`: Docking layout (`AetherTabViewer`) and panel rendering.
- `src/compiler.rs`: Code generation logic.
- `src/app.rs`: Main runtime loop.

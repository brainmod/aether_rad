# Project Aether: Development Status & Roadmap

**Based on Architectural Blueprint:** "Egui Forge" **Current Phase:** Transitioning from Phase 2 (Shell) to Phase 3 (Interactive Canvas).

## 1. Executive Summary

Aether is a native Rust Rapid Application Development (RAD) utility. It bridges the gap between **Immediate Mode** rendering (egui) and **Retained Mode** editing (Qt Designer style) using a **Shadow Object Model (SOM)** .

We have successfully implemented the **Kernel** (serialization/data model) and the **Shell** (docking workspace), proving that the core architectural risks (polymorphism and state persistence) are resolved.

## 2. Architectural Pillars

### The Shadow Object Model (SOM)

- **Concept:** A persistent data structure representing the UI state, distinct from the ephemeral frame rendering .
- **Implementation:** The `ProjectState` struct holds a root `Box<dyn WidgetNode>`.
- **Polymorphism:** Solved using `typetag` to serialize dynamic trait objects without rigid Enums .

### The "Edit Mode" Rendering

- **Concept:** Widgets render themselves. In the editor, they are wrapped in interaction logic (Gizmos) rather than executing their runtime behavior .
- **Status:** Basic rendering is active. Interaction interception is the next major milestone.

## 3. Progress Log

### ✅ Phase 1: The Kernel (Completed)

- [x] **WidgetNode Trait:** Defined with `render_editor`, `inspect`, and `codegen` capabilities .
- [x] **Serialization Engine:** Integrated `serde` + `typetag`. Verified JSON output for polymorphic types (`ButtonWidget`) .
- [x] **Recursion:** Implemented `VerticalLayout` to prove the SOM can handle nested trees .
- [x] **Project State:** Created the root container for the application definition .

### ✅ Phase 2: The Shell (Completed)

- [x] **Workspace Layout:** Integrated `egui_dock` to support resizable, tabbed panels .
- [x] **Panel Architecture:** Implemented `AetherTabViewer` with context injection (`ProjectState`) .
- [x] **The Feedback Loop:** Connecting the **Inspector** directly to the **Canvas**. Changes in property fields update the render immediately .

## 4. Upcoming Roadmap

### 🚧 Phase 3: The Interactive Canvas (Current Focus)

**Goal:** Distinguish between *interacting with the editor* and *interacting with the widget*.

- **Task 3.1: Hierarchy Tree Walker:** Improve the visual tree in the "Hierarchy" panel to properly visualize recursion.
- **Task 3.2: Selection System:** Implement a Global Selection set (`HashSet<Uuid>`). Clicking a widget in Canvas or Hierarchy selects it .
- **Task 3.3: Gizmos:** Draw selection outlines over the active widget in the Canvas .
- **Task 3.4: Drag & Drop:** Implement `egui_dnd` to drag widgets from Palette -> Canvas and reorder them .

### 📅 Phase 4: Logic & Data Binding

**Goal:** Allow users to define app behavior, not just visuals.

- **Variable Store:** Define the virtual `struct MyApp` state .
- **Data Binding:** Allow properties to bind to variables (e.g., Slider binds to `self.value`) .
- **Event System:** Add "Signals" (e.g., `On Click`) that trigger actions or code snippets .

### 📅 Phase 5: The Compiler

**Goal:** Generate standalone Rust code.

- **Codegen:** Utilize `quote` crate to synthesize ASTs from the SOM .
- **Scaffolding:** Generate `Cargo.toml`, `main.rs`, and `app.rs` .

## 5. Technical Reference

### Key Files

- `src/model.rs`: Defines `WidgetNode` (Trait) and `ProjectState` (Root). **(The SOM)**
- `src/widgets.rs`: Concrete implementations (`ButtonWidget`, `VerticalLayout`). **(The Standard Lib)**
- `src/ui.rs`: Layout definitions for `egui_dock` and panel rendering logic. **(The View)**
- `src/app.rs`: Main application state holding `DockState` and `ProjectState`.

### Adding a New Widget

To add a new widget (e.g., `Label`):

1. Define `struct LabelWidget` in `widgets.rs`.
2. Implement `Default`.
3. Implement `WidgetNode` with `#[typetag::serde]`.
4. Implement `render_editor` (draw the label).
5. Implement `inspect` (draw text input for content).
6. Implement `codegen` (return `ui.label(...)` tokens).

------

*Document generated based on "Egui Forge" Architectural Analysis .*
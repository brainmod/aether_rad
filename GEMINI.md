# Aether RAD - Gemini Context

## Current State Assessment (Dec 28, 2025)

**Status:** `v2.0` (Functionally Complete Prototype)
**Architecture:** Native Rust/egui with a "Shadow Object Model" (SOM).
**Platform:** Desktop (macOS/Linux/Windows) & Web (WASM).

The project has successfully transitioned from an `egui_dock` based tiling interface to a standard, idiomatic `egui` panel architecture (Left/Right Sidebars, Bottom Panel, Central Canvas).

| Component | Status | Implementation Details |
| :--- | :--- | :--- |
| **Core Kernel** | ✅ | `ProjectState` and `WidgetNode` trait facilitate a flexible SOM. Serialization via `serde`/`typetag` is working. |
| **Shell UI** | ✅ | Refactored to native panels. `UiState` manages visibility. Includes Palette, Assets, Hierarchy, Inspector, and Variables panels. |
| **Canvas** | ✅ | Interactive editor with selection, gizmos (outlines), and drag-and-drop placement. Supports basic layouts (Vertical, Horizontal, Grid). |
| **Logic** | ✅ | Variable store (typed variables) and Event/Action system (Click -> Increment/Set) are implemented. |
| **Compiler** | ✅ | Generates compilable `egui` code using `quote`. Includes `CodePreview` window. |
| **WASM** | ✅ | Configured with conditional compilation (`cfg(target_arch = "wasm32")`) and `trunk` support. |

## Development Roadmap

### Phase 1: Refinement & Polish (Current Focus)
*Focus: Stabilizing the recent refactor and improving User Experience (UX).*

*   [ ] **Cleanup:** Remove unused `egui_dock` dependencies and legacy code from `ui/mod.rs`.
*   [x] **Fix Regressions:** Fix Drag-and-Drop placement and Widget Selection issues.
*   [x] **Visual Feedback:** Improve Drag-and-Drop (DnD) visualization (ghost previews).
*   [x] **Property Polish:** Review `inspect` methods. Add "Reset" buttons.
*   [x] **Validation:** Enhance `CodeValidator`.

### Phase 2: Feature Expansion
*Focus: Closing the gap between a "Prototype" and a "Production Tool".*

*   [ ] **Expanded Widget Library:** Table/List, Plot, TreeView.
*   [ ] **Styling System:** Theme Editor and Export.
*   [ ] **Asset Management:** Drag-to-Canvas for images, Font Support.

### Phase 3: Advanced Capabilities
*   [ ] **Component System (Prefabs)**
*   [ ] **Interactive Preview ("Play Mode")**
*   [ ] **Multi-View Support**

## Active Tasks

1.  **Implement TreeView:** ⏳ Add `TreeViewWidget`.
2.  **Asset Management Polish:** Add drag-to-canvas for images.

## Completed Tasks
*   **Cleanup:** ✅ Remove `egui_dock`. (Done)
*   **Table Widget:** ✅ Added `TableWidget` with `egui_extras::TableBuilder`.
*   **Plot Widget:** ✅ Added `PlotWidget` with `egui_plot`. Supports Line, Bar, and Points series.
*   **Dependency Alignment:** ✅ Downgraded egui crates to `0.32.3` to match `egui_plot` requirements and fixed version mismatches.

## References
*   `CLAUDE.md` - Development Guide & Best Practices
*   `Rust RAD Utility Development Plan.md` - Original Architecture Blueprint
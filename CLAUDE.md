# CLAUDE.md - Development Guide for Aether RAD

This document provides context, development status, and best practices for AI agents and contributors.

## Project Overview

**Aether RAD** is a visual UI builder for Rust/egui applications. It uses a **Shadow Object Model (SOM)** architecture to bridge immediate-mode rendering with retained-mode editing.

**Current Version:** 2.0 (Functionally Complete Prototype)
**Architecture:** Native Rust/egui with Shadow Object Model
**Platforms:** Desktop (macOS/Linux/Windows) & Web (WASM)

### Key Concepts

1. **Shadow Object Model (SOM)**: A persistent data structure (`ProjectState`) that holds the UI tree. Widgets exist as `Box<dyn WidgetNode>` trait objects.

2. **WidgetNode Trait**: The core abstraction. Every widget implements:
   - `render_editor()` - Draw in canvas with selection/gizmos
   - `inspect()` - Expose properties in Inspector panel
   - `codegen()` - Generate Rust code via `quote!` macros

3. **typetag**: Enables polymorphic serialization of `Box<dyn WidgetNode>` to JSON.

4. **Native Panels**: IDE-style layout with Left Sidebar (Palette, Assets, Hierarchy), Right Sidebar (Inspector, Variables), Bottom Panel, and Central Canvas.

---

## Architecture Quick Reference

```
src/
├── main.rs           # Entry point
├── app.rs            # Main app struct, eframe integration, undo/redo, copy/paste
├── model.rs          # WidgetNode trait, ProjectState, Variable, VariableType, Action, WidgetEvent
├── widgets.rs        # 21 widget implementations
├── compiler.rs       # Code generation (Cargo.toml, main.rs, app.rs) with prettyplease formatting
├── theme.rs          # Light/Dark theme configuration, color schemes
├── syntax.rs         # Syntax highlighting for code preview (Rust and TOML)
├── validator.rs      # Cargo check integration for code validation
├── io.rs             # Platform-agnostic file I/O (native + WASM stubs)
├── lib.rs            # Library exports
└── ui/
    ├── mod.rs        # EditorContext, UiState, panel visibility management
    ├── canvas.rs     # Central canvas with zoom/pan/grid
    ├── palette.rs    # Widget library with drag-and-drop
    ├── hierarchy.rs  # Tree view with keyboard navigation
    ├── inspector.rs  # Property editor
    ├── variables.rs  # Variable management panel
    ├── assets.rs     # Asset manager UI
    └── code_preview.rs # Syntax-highlighted code view

tests/
└── integration_tests.rs  # 8 comprehensive tests
```

---

## Current Implementation Status

### Phase Completion Summary

| Phase | Name | Status | Notes |
|-------|------|--------|-------|
| 1 | The Kernel (SOM, Serialization) | Complete | WidgetNode trait, ProjectState, typetag |
| 2 | The Shell (Workspace Layout) | Complete | Native egui panels with UiState management |
| 3 | Interactive Canvas (D&D, Gizmos) | Complete | Selection, DnD, click-to-add all working |
| 4 | Logic & Data Binding | Complete | Variables, bindings, events, actions |
| 5 | The Compiler (Code Generation) | Complete | quote! macros, prettyplease formatting |

### Known Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| *(None critical)* | - | All major issues resolved |

### Recently Fixed

- **Selection**: Widgets now use inert rendering with reliable click detection
- **DnD**: Click-to-add fallback ensures widgets can always be added
- **Unused Code**: All ~30 warnings resolved with `#[allow(dead_code)]` on reserved code
- **Visual Feedback**: Drop zone indicators and improved drag ghost previews
- **Reordering**: Ctrl+Up/Down, Move buttons, and drag-to-reorder all work
- **Property Polish**: Added reset buttons to numeric and text properties

### Implemented Widgets (21 total)

| Widget | Category | Bindings | Events | Notes |
|--------|----------|----------|--------|-------|
| VerticalLayout | Layout | - | - | Container with spacing |
| HorizontalLayout | Layout | - | - | Container with spacing |
| GridLayout | Layout | - | - | Configurable columns |
| FreeformLayout | Layout | - | - | Absolute positioning |
| ScrollAreaWidget | Layout | - | - | Scrollable container |
| TabContainerWidget | Layout | - | - | Tabbed container |
| WindowWidget | Container | - | - | Floating window (partial) |
| Button | Input | text | Clicked, DoubleClicked | Full event support |
| TextEdit | Input | value | Changed | String binding |
| Checkbox | Input | checked | Changed | Boolean binding |
| Slider | Input | value | Changed | Range configurable |
| ComboBox | Input | selected | - | Dynamic options list |
| Label | Display | text | - | Text binding |
| ProgressBar | Display | value | - | 0.0-1.0 range |
| Image | Display | - | - | Resize handles, file picker |
| Separator | Display | - | - | Visual divider |
| Spinner | Display | - | - | Loading indicator with size |
| Hyperlink | Display | - | - | Clickable URL link |
| ColorPicker | Display | color | - | RGBA color selection |
| TableWidget | Data | - | - | With egui_extras::TableBuilder |
| PlotWidget | Data | - | - | Line, Bar, Points series |

### Extended Features

| Feature | Location | Notes |
|---------|----------|-------|
| Undo/Redo System | `src/app.rs` | 50-step history, Ctrl+Z/Y shortcuts |
| Copy/Paste | `src/app.rs` | Ctrl+C/V with UUID regeneration |
| Keyboard Navigation | `src/ui/hierarchy.rs` | Arrow keys in Hierarchy |
| Code Preview Panel | `src/ui/code_preview.rs` | Live-updating, all 3 files |
| Project Export | `src/ui/` | Write to disk with folder picker |
| Event System | `src/model.rs`, `src/widgets.rs` | WidgetEvent enum + Action enum |
| Light/Dark Theme | `src/theme.rs`, `src/app.rs` | Theme-aware colors |
| Syntax Highlighting | `src/syntax.rs` | LayoutJob-based coloring |
| Cargo Check Validation | `src/validator.rs` | Validates generated code compiles |
| Canvas Zoom/Pan | `src/ui/canvas.rs` | Zoom slider, fit/100% buttons |
| Project Templates | `src/model.rs` | Empty, Counter App, Form, Dashboard |
| Asset Manager | `src/model.rs`, `src/ui/assets.rs` | Images, audio, data assets |
| WASM Support | `Cargo.toml`, `src/io.rs` | Platform-agnostic file I/O |

---

## Development Roadmap

### Phase 1: Core Editing (Complete)

- [x] **Widget Selection**: Inert rendering with reliable click detection
- [x] **Drag-and-Drop**: Native egui DnD with click-to-add fallback
- [x] **Widget Manipulation**: Move Up/Down, keyboard shortcuts, drag-to-reorder
- [x] **Multi-selection**: Ctrl+click support with bulk actions

### Phase 2: Refinement & Polish (Current Priority)

**Focus:** Stabilize and improve UX

- [x] **Cleanup:** Remove unused code (~30 warnings)
- [x] **Visual Feedback:** Improve DnD ghost previews with drop zone indicators
- [x] **Property Polish:** Add "Reset to Default" buttons
- [ ] **Error Handling:** Replace panics with graceful fallbacks
- [ ] **Validation:** Enhance `CodeValidator` feedback

### Phase 3: Feature Expansion

**Focus:** Close gap between prototype and production tool

- [ ] **TreeView Widget:** Hierarchical data display
- [ ] **Asset Integration:** Complete codegen for assets
- [ ] **Styling System:** Theme editor and export
- [ ] **Component System:** Prefabs/reusable components

### Phase 4: Advanced Capabilities

- [ ] **Interactive Preview ("Play Mode")**
- [ ] **Multi-View Support** (multiple windows/screens)
- [ ] **WASM File API:** Browser File API for file selection
- [ ] **IndexedDB Storage:** For persistent WASM projects

---

## Event System

### WidgetEvent Types
- `Clicked` - Button click, widget tap
- `Changed` - Value modification (TextEdit, Checkbox, Slider)
- `Hovered` - Mouse hover (available but not widely used)
- `DoubleClicked` - Double-click (Button)
- `Focused` - Widget gained focus (defined, not implemented)
- `LostFocus` - Widget lost focus (defined, not implemented)

### Action Types
- `IncrementVariable(String)` - Increment a numeric variable by 1
- `SetVariable(String, String)` - Set a variable to a specific value
- `Custom(String)` - Custom Rust code injection

---

## Best Practices

### Adding a New Widget

1. **Define the struct** in `src/widgets.rs`:
   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone)]
   pub struct MyWidget {
       pub id: Uuid,
       pub some_property: String,
       #[serde(default)]
       pub bindings: std::collections::HashMap<String, String>,
       #[serde(default)]
       pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
   }
   ```

2. **Implement Default**:
   ```rust
   impl Default for MyWidget {
       fn default() -> Self {
           Self {
               id: Uuid::new_v4(),
               some_property: "default".to_string(),
               bindings: std::collections::HashMap::new(),
               events: std::collections::HashMap::new(),
           }
       }
   }
   ```

3. **Implement WidgetNode** with `#[typetag::serde]`:
   ```rust
   #[typetag::serde]
   impl WidgetNode for MyWidget {
       fn clone_box(&self) -> Box<dyn WidgetNode> { Box::new(self.clone()) }
       fn id(&self) -> Uuid { self.id }
       fn name(&self) -> &str { "My Widget" }
       fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) { ... }
       fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) { ... }
       fn codegen(&self) -> TokenStream { ... }
   }
   ```

4. **Register in drop zones** - Add to match statements in ALL layout `render_editor()` methods

5. **Add to Palette** - Add widget name to appropriate category in `render_widget_category()`

6. **Add label** - Add widget label in `src/theme.rs` `WidgetLabels::get()` method

### Code Style

- Use `quote!` for code generation, not string concatenation
- Always derive `Debug, Serialize, Deserialize, Clone` on widget structs
- Use `#[serde(default)]` for optional fields like `bindings` and `events`
- Selection gizmos use `Color32::from_rgb(255, 165, 0)` (orange)
- Keep `render_editor` logic simple: draw widget, handle click, draw gizmo if selected
- Use `handle_selection()` helper for multi-selection support
- Use `draw_gizmo()` and `draw_resize_handles()` helpers for visual feedback

### Testing

```bash
cargo build              # Verify compilation
cargo test               # Run unit + integration tests
cargo run                # Manual testing
cargo clippy             # Lint check (aim for zero warnings)
```

---

## Common Patterns

### Selection and Gizmo Pattern

```rust
fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
    let response = ui.some_widget(...);

    // Use helper for multi-selection
    handle_selection(ui, self.id, response.clicked(), selection);

    if selection.contains(&self.id) {
        draw_gizmo(ui, response.rect);
    }

    // Optional: tooltips
    response.on_hover_text(format!("{}: {}\nID: {}", self.name(), some_property, self.id));
}
```

### Adding a Binding-Enabled Property

```rust
// In inspect():
let is_bound = self.bindings.contains_key("property_name");
let mut bound_mode = is_bound;
if ui.checkbox(&mut bound_mode, "Bind").changed() {
    if bound_mode {
        self.bindings.insert("property_name".to_string(),
            known_variables.first().cloned().unwrap_or_default());
    } else {
        self.bindings.remove("property_name");
    }
}
// ... show ComboBox if bound, text_edit if not

// In codegen():
if let Some(var) = self.bindings.get("property_name") {
    let ident = quote::format_ident!("{}", var);
    quote! { &self.#ident }
} else {
    let val = &self.property;
    quote! { #val }
}
```

### Adding Events to a Widget

```rust
// In struct:
#[serde(default)]
pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,

// In inspect():
ui.separator();
ui.heading("Events");

let possible_events = [WidgetEvent::Clicked, WidgetEvent::Changed];

for event in &possible_events {
    if self.events.contains_key(event) {
        ui.collapsing(format!("{}", event), |ui| {
            if let Some(action) = self.events.get_mut(event) {
                render_action_editor(ui, action, known_variables);
            }
            if ui.button("Remove Event").clicked() {
                self.events.remove(event);
            }
        });
    } else if ui.button(format!("+ Add {}", event)).clicked() {
        self.events.insert(*event, Action::Custom(String::new()));
    }
}

// In codegen():
let clicked_code = if let Some(action) = self.events.get(&WidgetEvent::Clicked) {
    action.to_code()
} else {
    quote! {}
};

quote! {
    if ui.button(#label).clicked() {
        #clicked_code
    }
}
```

---

## Development Workflow

1. **Pick a task** from the roadmap (start with P0 issues)
2. **Read the relevant files** to understand current implementation
3. **Make minimal, focused changes** - avoid over-engineering
4. **Test manually** with `cargo run`
5. **Run `cargo clippy`** to check for issues
6. **Run `cargo test`** to verify no regressions
7. **Commit with descriptive message**

---

## Reference Documentation

- **Full Architecture**: See `Rust RAD Utility Development Plan.md`
- **WASM Setup**: See `WASM.md`
- **egui Docs**: https://docs.rs/egui
- **quote Docs**: https://docs.rs/quote
- **typetag Docs**: https://docs.rs/typetag
- **prettyplease Docs**: https://docs.rs/prettyplease

---

## Quick Debugging Commands

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Check for warnings
cargo clippy 2>&1 | head -100

# Run specific test
cargo test test_name -- --nocapture

# Check WASM build
cargo build --target wasm32-unknown-unknown --features wasm
```

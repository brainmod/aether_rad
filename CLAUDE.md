# CLAUDE.md - Development Guide for AI Agents

This document provides context, bite-sized tasks, and best practices for AI agents contributing to Aether RAD.

## Project Context

**Aether RAD** is a visual UI builder for Rust/egui applications. It uses a **Shadow Object Model (SOM)** architecture to bridge immediate-mode rendering with retained-mode editing.

### Key Concepts

1. **Shadow Object Model (SOM)**: A persistent data structure (`ProjectState`) that holds the UI tree. Widgets exist as `Box<dyn WidgetNode>` trait objects.

2. **WidgetNode Trait**: The core abstraction. Every widget implements:
   - `render_editor()` - Draw in canvas with selection/gizmos
   - `inspect()` - Expose properties in Inspector panel
   - `codegen()` - Generate Rust code via `quote!` macros

3. **typetag**: Enables polymorphic serialization of `Box<dyn WidgetNode>` to JSON.

4. **Panels**: IDE-style docking layout with Canvas, Palette, Hierarchy, Inspector, Variables, Assets, Output, and CodePreview panels.

---

## Architecture Quick Reference

```
src/model.rs     - WidgetNode trait, ProjectState, Variable, VariableType, Action, WidgetEvent, AssetManager
src/widgets.rs   - Widget implementations (15 widgets including layouts, inputs, and display widgets)
src/ui.rs        - AetherTabViewer, panel rendering, docking layout
src/compiler.rs  - Code generation (Cargo.toml, main.rs, app.rs) with prettyplease formatting
src/app.rs       - Main app struct, eframe integration, save/load, undo/redo, copy/paste
src/theme.rs     - Light/Dark theme configuration, color schemes
src/syntax.rs    - Syntax highlighting for code preview (Rust and TOML)
src/validator.rs - Cargo check integration for code validation
src/io.rs        - Platform-agnostic file I/O (native + WASM stubs)
src/lib.rs       - Library exports
src/main.rs      - Entry point
tests/           - Integration tests for serialization, codegen, and manipulation
```

---

## Current Implementation Status

### Phase Completion Summary

| Phase | Name | Status | Notes |
|-------|------|--------|-------|
| 1 | The Kernel (SOM, Serialization) | Complete | WidgetNode trait, ProjectState, typetag |
| 2 | The Shell (Workspace Layout) | Complete | egui_dock with 8 panels |
| 3 | Interactive Canvas (D&D, Gizmos) | Complete | Selection, gizmos, palette DnD |
| 4 | Logic & Data Binding | Complete | Variables, bindings, events, actions |
| 5 | The Compiler (Code Generation) | Complete | quote! macros, prettyplease formatting |

### Extended Features (Beyond Original Plan)

| Feature | Location | Notes |
|---------|----------|-------|
| Undo/Redo System | `src/app.rs` | 50-step history, Ctrl+Z/Y shortcuts |
| Delete Widget | `src/app.rs`, `src/ui.rs` | Delete key + button in Inspector |
| Widget Reordering | `src/model.rs`, `src/ui.rs` | Move Up/Down buttons |
| Copy/Paste | `src/app.rs` | Ctrl+C/V with UUID regeneration |
| Keyboard Navigation | `src/ui.rs` | Arrow keys in Hierarchy |
| Code Preview Panel | `src/ui.rs` | Live-updating, all 3 files |
| Project Export | `src/ui.rs` | Write to disk with folder picker |
| Custom Project Names | `src/model.rs`, `src/compiler.rs` | Editable in Output panel |
| Event System | `src/model.rs`, `src/widgets.rs` | WidgetEvent enum + Action enum |
| Integration Tests | `tests/integration_tests.rs` | Round-trip, codegen, manipulation |
| Gizmo System | `src/widgets.rs` | Orange outline, resize handles (Image) |
| Light/Dark Theme Toggle | `src/theme.rs`, `src/app.rs` | Theme-aware colors, user-selectable |
| Syntax Highlighting | `src/syntax.rs`, `src/ui.rs` | LayoutJob-based coloring, theme-aware |
| Cargo Check Validation | `src/ui.rs`, `src/validator.rs` | Spinner animation, disabled button while checking |
| Canvas Zoom/Pan | `src/ui.rs` | Zoom slider, fit/100% buttons |
| Multi-Selection | `src/ui.rs`, `src/widgets.rs` | Ctrl+click toggle, bulk delete |
| Project Templates | `src/model.rs` | Empty, Counter App, Form, Dashboard |
| Asset Manager | `src/model.rs`, `src/ui.rs` | Assets panel, image/audio/data types |
| WASM Support | `Cargo.toml`, `src/io.rs` | Platform-agnostic file I/O |

### Implemented Widgets (15 total)

| Widget | Category | Bindings | Events | Notes |
|--------|----------|----------|--------|-------|
| VerticalLayout | Layout | - | - | Container with spacing |
| HorizontalLayout | Layout | - | - | Container with spacing |
| GridLayout | Layout | - | - | Configurable columns |
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

---

## Event System

### WidgetEvent Types
- `Clicked` - Button click, widget tap
- `Changed` - Value modification (TextEdit, Checkbox, Slider)
- `Hovered` - Mouse hover (available but not widely used)
- `DoubleClicked` - Double-click (Button)

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

4. **Register in drop zones** - Add to match statements in ALL layout `render_editor()` methods:
   - `VerticalLayout::render_editor()`
   - `HorizontalLayout::render_editor()`
   - `GridLayout::render_editor()`

5. **Add to Palette** - Add widget name to the appropriate category in `render_widget_category()` calls in `src/ui.rs`

6. **Add label** - Add widget label in `src/theme.rs` `WidgetLabels::get()` method

### Code Style

- Use `quote!` for code generation, not string concatenation
- Always derive `Debug, Serialize, Deserialize, Clone` on widget structs
- Use `#[serde(default)]` for optional fields like `bindings` and `events`
- Selection gizmos use `Color32::from_rgb(255, 165, 0)` (orange)
- Keep `render_editor` logic simple: draw widget, handle click for selection, draw gizmo if selected
- Use `handle_selection()` helper for multi-selection support
- Use `draw_gizmo()` and `draw_resize_handles()` helpers for visual feedback

### Testing

```bash
cargo build              # Verify compilation
cargo test               # Run unit + integration tests
cargo run                # Manual testing
cargo clippy             # Lint check
```

---

## Bite-Sized Tasks (Future Work)

### Priority 1: Widget Expansion

#### Task: Add Window Container Widget
**Complexity:** Hard | **Files:** `src/widgets.rs`, `src/ui.rs`

Per the development plan, allow creating egui::Window containers.

- [ ] Create `WindowWidget` with `title: String`, `children: Vec<Box<dyn WidgetNode>>`
- [ ] `render_editor`: Draw a styled frame representing the window
- [ ] Inspector: title field, open/closeable toggles
- [ ] `codegen`: emit `egui::Window::new(...).show(ctx, |ui| { ... })`
- [ ] Handle window state in generated app struct

#### Task: Add TabContainer Widget
**Complexity:** Medium | **Files:** `src/widgets.rs`, `src/ui.rs`

A tabbed container for organizing content.

- [ ] Create `TabContainerWidget` with `tabs: Vec<(String, Vec<Box<dyn WidgetNode>>)>`
- [ ] `render_editor`: Show tabs with content switching
- [ ] Inspector: tab management (add/remove/rename)
- [ ] `codegen`: emit proper tab UI code

#### Task: Add ScrollArea Widget
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

A scrollable container widget.

- [ ] Create `ScrollAreaWidget` with `children: Vec<Box<dyn WidgetNode>>`, scroll direction options
- [ ] `render_editor`: Wrap children in ScrollArea
- [ ] `codegen`: emit `egui::ScrollArea::...`

### Priority 2: Hierarchy & Canvas Improvements

#### Task: Re-enable Hierarchy Drag-and-Drop Re-parenting
**Complexity:** Medium | **Files:** `src/ui.rs`, `src/model.rs`

The hierarchy DnD sources exist but re-parenting is not fully functional.

- [ ] Implement proper `dnd_drop_zone` handling in hierarchy
- [ ] Fix the `pending_reorder` handling for cross-container moves
- [ ] Add visual drop indicators (insertion lines between items)
- [ ] Add undo support for drag-and-drop reorders

#### Task: Improve Canvas Zoom/Pan UX
**Complexity:** Easy | **Files:** `src/ui.rs`

Enhance the zoom/pan experience.

- [ ] Add Ctrl+scroll wheel for zooming
- [ ] Add middle-mouse drag for panning
- [ ] Show zoom percentage in canvas header
- [ ] Persist zoom/pan state in project

### Priority 3: Code Quality

#### Task: Add Codegen Compilation Test
**Complexity:** Medium | **Files:** `tests/`

Test that generated code actually compiles.

- [ ] Create test that generates a project with various widgets
- [ ] Write generated code to temp directory
- [ ] Run `cargo check` on the generated project
- [ ] Assert process exits with success
- [ ] Clean up temp files

#### Task: Add Nested Widget Tests
**Complexity:** Easy | **Files:** `tests/integration_tests.rs`

Test deeply nested widget structures.

- [ ] Test VerticalLayout containing HorizontalLayout containing Grid
- [ ] Verify serialization/deserialization preserves nesting
- [ ] Verify codegen produces valid nested code

### Priority 4: Event System Expansion

#### Task: Add Hover Event Support
**Complexity:** Easy | **Files:** `src/widgets.rs`

Enable hover events for interactive widgets.

- [ ] Add `Hovered` event handling to Button, Image widgets
- [ ] Update Inspector to show hover event option
- [ ] Update codegen to emit `.hovered()` checks

#### Task: Add Focus Events
**Complexity:** Medium | **Files:** `src/model.rs`, `src/widgets.rs`

Support focus/blur events for input widgets.

- [ ] Add `Focused`, `LostFocus` to WidgetEvent enum
- [ ] Implement for TextEdit, ComboBox
- [ ] Update codegen

### Priority 5: WASM Improvements

#### Task: Implement Browser File API
**Complexity:** Hard | **Files:** `src/io.rs`

Currently WASM file dialogs return None. Use browser File API.

- [ ] Research web-sys File API integration
- [ ] Implement async file picker for WASM target
- [ ] Support drag-and-drop file loading
- [ ] Add IndexedDB storage for projects

### Priority 6: Asset Management

#### Task: Complete Asset Integration
**Complexity:** Medium | **Files:** `src/ui.rs`, `src/model.rs`

Finish the asset manager integration.

- [ ] Implement actual file import in Assets panel
- [ ] Allow ImageWidget to reference assets by name
- [ ] Copy assets to output directory on export
- [ ] Generate code that loads assets from relative paths

---

## Development Workflow

1. **Pick a task** from the list above
2. **Read the relevant files** to understand current implementation
3. **Make minimal, focused changes** - avoid over-engineering
4. **Test manually** with `cargo run`
5. **Run `cargo clippy`** to check for issues
6. **Run `cargo test`** to verify no regressions
7. **Commit with descriptive message**

---

## Common Patterns

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

### Adding Events to a Widget

```rust
// In struct:
#[serde(default)]
pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,

// In inspect():
ui.separator();
ui.heading("Events");

let mut events_to_add = None;
let mut events_to_remove = None;

let possible_events = [WidgetEvent::Changed]; // List supported events

for event in &possible_events {
    if self.events.contains_key(event) {
        ui.collapsing(format!("{}", event), |ui| {
            if let Some(action) = self.events.get_mut(event) {
                render_action_editor(ui, action, known_variables);
            }
            if ui.button("Remove Event").clicked() {
                events_to_remove = Some(*event);
            }
        });
    } else {
        if ui.button(format!("+ Add {}", event)).clicked() {
            events_to_add = Some(*event);
        }
    }
}

if let Some(event) = events_to_add {
    self.events.insert(event, Action::Custom(String::new()));
}
if let Some(event) = events_to_remove {
    self.events.remove(&event);
}

// In codegen():
let changed_code = if let Some(action) = self.events.get(&WidgetEvent::Changed) {
    action.to_code()
} else {
    quote! {}
};

quote! {
    if ui.add(SomeWidget).changed() {
        #changed_code
    }
}
```

---

## Reference Documentation

- **Full Architecture**: See `Rust RAD Utility Development Plan.md`
- **WASM Setup**: See `WASM.md`
- **egui Docs**: https://docs.rs/egui
- **quote Docs**: https://docs.rs/quote
- **typetag Docs**: https://docs.rs/typetag
- **prettyplease Docs**: https://docs.rs/prettyplease
- **syntect Docs**: https://docs.rs/syntect

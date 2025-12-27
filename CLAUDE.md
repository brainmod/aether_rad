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

4. **Panels**: IDE-style docking layout with Canvas, Palette, Hierarchy, Inspector, Variables, Output, and CodePreview panels.

---

## Architecture Quick Reference

```
src/model.rs    - WidgetNode trait, ProjectState, Variable, VariableType
src/widgets.rs  - Widget implementations (Button, Label, layouts, etc.)
src/ui.rs       - AetherTabViewer, panel rendering, docking layout
src/compiler.rs - Code generation (Cargo.toml, main.rs, app.rs)
src/app.rs      - Main app struct, eframe integration, save/load, undo/redo
src/main.rs     - Entry point
tests/          - Integration tests for serialization and code generation
```

---

## Current Implementation Status

### ✅ Completed Features

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
| Event Code Injection | `src/widgets.rs` | `clicked_code` with syntax validation |
| Integration Tests | `tests/integration_tests.rs` | Round-trip, codegen, manipulation |
| Gizmo System | `src/widgets.rs` | Orange outline, resize handles (Image) |
| Light/Dark Theme Toggle | `src/theme.rs`, `src/app.rs`, `src/ui.rs` | Theme-aware colors, user-selectable |
| Syntax Highlighting (Per-Token) | `src/syntax.rs`, `src/ui.rs` | LayoutJob-based coloring, theme-aware |
| Cargo Check with Progress Feedback | `src/ui.rs`, `src/validator.rs` | Spinner animation, disabled button while checking |

### ✅ Implemented Widgets (11 total)

| Widget | Bindings | Events | Notes |
|--------|----------|--------|-------|
| Button | text | clicked_code | Full event support |
| Label | text | - | |
| Text Edit | value | - | |
| Checkbox | checked | - | |
| Slider | value | - | Range configurable |
| Progress Bar | value | - | 0.0-1.0 range |
| ComboBox | selected | - | Dynamic options list |
| Image | - | - | Resize handles, file picker |
| Vertical Layout | - | - | Container |
| Horizontal Layout | - | - | Container |
| Grid Layout | - | - | Configurable columns |

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
           }
       }
   }
   ```

3. **Implement WidgetNode** with `#[typetag::serde]`:
   ```rust
   #[typetag::serde]
   impl WidgetNode for MyWidget {
       fn clone_box(&self) -> Box<dyn WidgetNode> {
           Box::new(self.clone())
       }
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

5. **Add to Palette** - Add widget name to the `widgets` vec in `AetherTabViewer::render_palette()` in `src/ui.rs`

### Code Style

- Use `quote!` for code generation, not string concatenation
- Always derive `Debug, Serialize, Deserialize, Clone` on widget structs
- Use `#[serde(default)]` for optional fields like `bindings`
- Selection gizmos use `Color32::from_rgb(255, 165, 0)` (orange)
- Keep `render_editor` logic simple: draw widget, handle click for selection, draw gizmo if selected

### Testing

```bash
cargo build              # Verify compilation
cargo test               # Run unit tests
cargo run                # Manual testing
cargo clippy             # Lint check
```

---

## Bite-Sized Tasks

### Priority 1: Code Quality & Output Polish

#### Task: Add Pretty-Printing to Generated Code
**Complexity:** Easy | **Files:** `src/compiler.rs`, `Cargo.toml`

The generated code from `quote!` is unformatted. Use `prettyplease` to make output readable.

- [ ] Add `prettyplease = "0.2"` and `syn = { version = "2", features = ["full", "parsing"] }` to Cargo.toml
- [ ] In `generate_app_rs()`, parse the TokenStream with `syn::parse2()` and format with `prettyplease::unparse()`
- [ ] Handle parse errors gracefully (fall back to raw output)

#### Task: Add Syntax Highlighting to Code Preview
**Complexity:** Medium | **Files:** `src/ui.rs`, `Cargo.toml`

The Code Preview panel shows plain text. Add syntax highlighting for Rust code.

- [ ] Add `syntect = "5"` dependency
- [ ] Create a cached syntax highlighter in AetherApp
- [ ] In `render_code_preview()`, render highlighted text using egui's `RichText` or custom layouter
- [ ] Use a theme that works with both light/dark modes

#### Task: Validate Generated Code Compiles
**Complexity:** Hard | **Files:** `src/compiler.rs`, `src/ui.rs`

Add a "Check" button that runs `cargo check` on generated code.

- [ ] In Output panel, add "🔧 Check Code" button
- [ ] Write generated files to a temp directory
- [ ] Run `cargo check --manifest-path <temp>/Cargo.toml` and capture output
- [ ] Display success/error in a new status area
- [ ] Clean up temp directory after check

### Priority 2: Event System Expansion

#### Task: Add Multi-Event Support to Widgets
**Complexity:** Medium | **Files:** `src/widgets.rs`, `src/model.rs`

Currently only Button has `clicked_code`. Extend the event system to support more events.

- [ ] Create `WidgetEvent` enum: `Clicked`, `Changed`, `Hovered`, `DoubleClicked`
- [ ] Add `events: HashMap<WidgetEvent, String>` to widgets that support events
- [ ] Update Inspector to show event editors for each supported event type
- [ ] Update codegen to emit appropriate event handlers (`.clicked()`, `.changed()`, `.hovered()`)
- [ ] Add events to: Checkbox (changed), Slider (changed), TextEdit (changed)

#### Task: Add Standard Actions System
**Complexity:** Medium | **Files:** `src/model.rs`, `src/widgets.rs`, `src/ui.rs`

Per the development plan, provide pre-built actions users can select.

- [ ] Create `Action` enum: `IncrementVariable(String)`, `SetVariable(String, String)`, `Custom(String)`
- [ ] In Inspector event editors, add dropdown to select action type
- [ ] For `IncrementVariable`, show variable selector
- [ ] For `SetVariable`, show variable + value fields
- [ ] For `Custom`, show code editor (current behavior)
- [ ] Update codegen to emit appropriate code for each action type

### Priority 3: Widget Expansion

#### Task: Add Separator Widget
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

Simple visual separator for layouts.

- [ ] Create `SeparatorWidget` struct with `id: Uuid`
- [ ] `render_editor`: `ui.separator()` with selection gizmo
- [ ] `codegen`: `quote! { ui.separator(); }`
- [ ] Register in Palette and drop zones

#### Task: Add Spinner/Loading Widget
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

A loading spinner indicator.

- [ ] Create `SpinnerWidget` with `id: Uuid`, `size: f32`
- [ ] `render_editor`: `ui.spinner()` with size
- [ ] Inspector: size DragValue
- [ ] `codegen`: emit spinner code

#### Task: Add ColorPicker Widget
**Complexity:** Medium | **Files:** `src/widgets.rs`, `src/ui.rs`

A color selection widget.

- [ ] Create `ColorPickerWidget` with `color: [f32; 4]`, bindings
- [ ] `render_editor`: `ui.color_edit_button_rgba_unmultiplied()`
- [ ] Inspector: inline color picker + binding option
- [ ] `codegen`: emit color picker code with binding support

#### Task: Add Hyperlink Widget
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

A clickable hyperlink.

- [ ] Create `HyperlinkWidget` with `text: String`, `url: String`
- [ ] `render_editor`: `ui.hyperlink_to()` with selection
- [ ] Inspector: text and URL fields
- [ ] `codegen`: emit hyperlink code

#### Task: Add Window Container Widget
**Complexity:** Hard | **Files:** `src/widgets.rs`, `src/ui.rs`

Per the development plan, allow creating egui::Window containers.

- [ ] Create `WindowWidget` with `title: String`, `children: Vec<Box<dyn WidgetNode>>`
- [ ] `render_editor`: Draw a styled frame representing the window
- [ ] Inspector: title field, open/closeable toggles
- [ ] `codegen`: emit `egui::Window::new(...).show(ctx, |ui| { ... })`
- [ ] Handle window state in generated app struct

### Priority 4: Hierarchy & Canvas Improvements

#### Task: Re-enable Hierarchy Drag-and-Drop
**Complexity:** Medium | **Files:** `src/ui.rs`, `src/model.rs`

The hierarchy DnD is disabled. Re-enable with proper reorder logic.

- [ ] Switch from `draw_hierarchy_node_simple` back to `draw_hierarchy_node`
- [ ] Fix the `pending_reorder` handling to work correctly
- [ ] Add visual drop indicators (insertion lines between items)
- [ ] Support cross-container moves (move widget from one layout to another)
- [ ] Add undo support for drag-and-drop reorders

#### Task: Add Canvas Zoom and Pan
**Complexity:** Medium | **Files:** `src/ui.rs`

Allow users to zoom and pan the canvas for large designs.

- [ ] Wrap canvas content in a `ScrollArea` with zoom transform
- [ ] Add zoom slider or Ctrl+scroll wheel support
- [ ] Store zoom level in AetherApp (not ProjectState)
- [ ] Add "Fit to View" and "100%" buttons

#### Task: Add Multi-Selection Support
**Complexity:** Hard | **Files:** `src/model.rs`, `src/ui.rs`, `src/app.rs`

Allow selecting multiple widgets (Ctrl+click, Shift+click).

- [ ] Selection is already `HashSet<Uuid>`, use it properly
- [ ] In `render_editor`, check for Ctrl modifier before clearing selection
- [ ] In Hierarchy, implement Shift+click for range selection
- [ ] Update Inspector to show "N widgets selected" when multiple
- [ ] Group operations: delete all, move all, etc.

### Priority 5: WASM & Cross-Platform

#### Task: Add WASM Build Configuration
**Complexity:** Medium | **Files:** `Cargo.toml`, new files

Enable web deployment as per the development plan.

- [ ] Add `[target.'cfg(target_arch = "wasm32")'.dependencies]` section
- [ ] Replace `rfd` file dialogs with browser-compatible alternatives
- [ ] Create `index.html` and WASM loading script
- [ ] Add build script or instructions for `wasm-pack` / `trunk`
- [ ] Test in browser, document any limitations

#### Task: Add Project Templates
**Complexity:** Easy | **Files:** `src/app.rs`, `src/ui.rs`

Provide starter templates for common app types.

- [ ] Add "New Project" submenu in File menu
- [ ] Templates: Empty, Counter App, Form, Dashboard
- [ ] Each template creates pre-configured ProjectState with widgets and variables
- [ ] Counter App: Label + Button + counter variable

### Priority 6: Asset Management

#### Task: Implement Asset Manager
**Complexity:** Hard | **Files:** `src/model.rs`, `src/ui.rs`, new file

Per the development plan, create a centralized asset registry.

- [ ] Create `AssetManager` struct with `images: HashMap<String, PathBuf>`
- [ ] Add `assets` field to `ProjectState`
- [ ] Create "Assets" panel (new AetherTab variant)
- [ ] Allow importing images with friendly names
- [ ] ImageWidget references assets by name, not path
- [ ] On export, copy assets to output directory

### Priority 7: Testing & Documentation

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

#### Task: Add Binding Edge Case Tests
**Complexity:** Easy | **Files:** `tests/integration_tests.rs`

- [ ] Test binding to non-existent variable (should handle gracefully)
- [ ] Test changing variable type after binding
- [ ] Test multiple widgets bound to same variable

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

    if response.clicked() {
        selection.clear();
        selection.insert(self.id);
    }

    if selection.contains(&self.id) {
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)),
            egui::StrokeKind::Outside,
        );
    }

    // Optional: tooltips
    response.on_hover_text(format!("{}: {}\nID: {}", self.name(), some_property, self.id));
}
```

### Adding Events to a Widget

```rust
// In struct:
#[serde(default)]
pub changed_code: String,

// In inspect():
ui.separator();
ui.heading("On Change Event");
let code_editor = egui::TextEdit::multiline(&mut self.changed_code)
    .code_editor()
    .desired_rows(3);
ui.add(code_editor);

if !self.changed_code.trim().is_empty() {
    if self.changed_code.parse::<proc_macro2::TokenStream>().is_ok() {
        ui.colored_label(egui::Color32::GREEN, "✓ Valid Rust syntax");
    } else {
        ui.colored_label(egui::Color32::RED, "✗ Invalid Rust syntax");
    }
}

// In codegen():
let action = if !self.changed_code.trim().is_empty() {
    match self.changed_code.parse::<proc_macro2::TokenStream>() {
        Ok(tokens) => tokens,
        Err(_) => quote! { /* Invalid code */ }
    }
} else {
    quote! {}
};

quote! {
    if ui.add(SomeWidget).changed() {
        #action
    }
}
```

---

## Recent Improvements (Latest Session)

### ✨ Light/Dark Theme Toggle
- Added `ThemeMode` enum (Light/Dark) to `src/theme.rs`
- Created `LightModeColors` struct with appropriate light-themed colors
- Updated `configure_aether_theme()` to accept theme mode parameter
- Theme toggle button in Output panel (☀️/🌙 icons)
- All panels and components now respect theme setting
- Real-time theme switching on every frame

### ✨ Syntax Highlighting with Per-Token Colors
- Replaced monochrome syntax highlighting with `LayoutJob`-based per-token coloring
- Extracts style information from syntect and applies proper colors to each token
- Theme-aware: uses "Solarized (light)" for light mode, "Solarized (dark)" for dark mode
- Code preview now displays proper syntax coloring in all code sections
- Implemented in `src/syntax.rs` with helper functions for color conversion

### ✨ Cargo Check with Progress Feedback
- Added visual spinner animation while cargo check runs
- Button disabled during validation to prevent multiple concurrent checks
- Status message updates: "⏳ Cargo check in progress..."
- Spinner provides immediate visual feedback that work is happening
- Extended `ValidationStatus` enum with proper checking state handling

## Reference Documentation

- **Full Architecture**: See `Rust RAD Utility Development Plan.md`
- **Progress Log**: See `GEMINI.md`
- **egui Docs**: https://docs.rs/egui
- **quote Docs**: https://docs.rs/quote
- **typetag Docs**: https://docs.rs/typetag
- **prettyplease Docs**: https://docs.rs/prettyplease
- **syntect Docs**: https://docs.rs/syntect

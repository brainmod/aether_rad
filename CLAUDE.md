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

4. **Panels**: IDE-style docking layout with Canvas, Palette, Hierarchy, Inspector, Variables, and Output panels.

---

## Architecture Quick Reference

```
src/model.rs    - WidgetNode trait, ProjectState, Variable, VariableType
src/widgets.rs  - Widget implementations (Button, Label, layouts, etc.)
src/ui.rs       - AetherTabViewer, panel rendering, docking layout
src/compiler.rs - Code generation (Cargo.toml, main.rs, app.rs)
src/app.rs      - Main app struct, eframe integration, save/load
src/main.rs     - Entry point
```

---

## Best Practices

### Adding a New Widget

1. **Define the struct** in `src/widgets.rs`:
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct MyWidget {
       pub id: Uuid,
       pub some_property: String,
       #[serde(default)]
       pub bindings: HashMap<String, String>,
   }
   ```

2. **Implement Default**:
   ```rust
   impl Default for MyWidget {
       fn default() -> Self {
           Self {
               id: Uuid::new_v4(),
               some_property: "default".to_string(),
               bindings: HashMap::new(),
           }
       }
   }
   ```

3. **Implement WidgetNode** with `#[typetag::serde]`:
   ```rust
   #[typetag::serde]
   impl WidgetNode for MyWidget {
       fn id(&self) -> Uuid { self.id }
       fn name(&self) -> &str { "My Widget" }
       fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) { ... }
       fn inspect(&mut self, ui: &mut Ui, known_variables: &[String]) { ... }
       fn codegen(&self) -> TokenStream { ... }
   }
   ```

4. **Register in drop zones** - Add to match statements in `VerticalLayout::render_editor()` and `HorizontalLayout::render_editor()`

5. **Add to Palette** - Add widget name to the `widgets` vec in `AetherTabViewer::render_palette()` in `src/ui.rs`

### Code Style

- Use `quote!` for code generation, not string concatenation
- Always derive `Debug, Serialize, Deserialize` on widget structs
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

### Priority 1: Core Improvements

#### Task: Implement Undo/Redo System
**Complexity:** Medium | **Files:** `src/model.rs`, `src/app.rs`

- [ ] Create `UndoStack` struct with `Vec<ProjectState>` history
- [ ] Add `push_state()` before mutations
- [ ] Implement `undo()` and `redo()` methods
- [ ] Add keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z)
- [ ] Add Edit menu with Undo/Redo buttons

#### Task: Add Delete Widget Functionality
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

- [ ] Add "Delete" button to Inspector when widget selected
- [ ] Implement recursive `remove_child()` on containers
- [ ] Add keyboard shortcut (Delete key)
- [ ] Clear selection after deletion

#### Task: Implement Widget Reordering in Hierarchy
**Complexity:** Medium | **Files:** `src/ui.rs`

- [ ] Add drag handles to hierarchy tree items
- [ ] Implement `egui_dnd` or native drag for reordering
- [ ] Update parent's children Vec on drop
- [ ] Visual feedback during drag

### Priority 2: Widget Expansion

#### Task: Add ProgressBar Widget
**Complexity:** Easy | **Files:** `src/widgets.rs`, `src/ui.rs`

- [ ] Create `ProgressBarWidget` struct with `value: f32` (0.0-1.0)
- [ ] Implement `render_editor` using `ui.add(egui::ProgressBar::new(self.value))`
- [ ] Add binding support for `value` property
- [ ] Register in Palette and drop zones

#### Task: Add ComboBox Widget
**Complexity:** Medium | **Files:** `src/widgets.rs`, `src/ui.rs`

- [ ] Create `ComboBoxWidget` with `options: Vec<String>`, `selected: usize`
- [ ] Inspector UI for editing options list
- [ ] Binding for selected index
- [ ] Proper codegen with options array

#### Task: Add Image Widget
**Complexity:** Medium | **Files:** `src/widgets.rs`, `src/ui.rs`, `Cargo.toml`

- [ ] Add `egui_extras` dependency for image support
- [ ] Create `ImageWidget` with `path: String`, `size: Vec2`
- [ ] File picker in Inspector
- [ ] Handle missing images gracefully

#### Task: Add Grid Layout Container
**Complexity:** Hard | **Files:** `src/widgets.rs`

- [ ] Create `GridLayout` with `columns: usize`, `children: Vec<Box<dyn WidgetNode>>`
- [ ] Implement proper grid drop zones
- [ ] Grid-based codegen with `ui.columns()`

### Priority 3: UX Enhancements

#### Task: Add Live Code Preview Panel
**Complexity:** Medium | **Files:** `src/ui.rs`

- [ ] Add new `AetherTab::CodePreview` variant
- [ ] Render generated code with syntax highlighting
- [ ] Update in real-time as user edits

#### Task: Improve Gizmo System
**Complexity:** Medium | **Files:** `src/widgets.rs`

- [ ] Add resize handles to gizmos
- [ ] Implement drag-to-resize for widgets with explicit sizing
- [ ] Show property tooltips on hover

#### Task: Add Widget Copy/Paste
**Complexity:** Medium | **Files:** `src/app.rs`, `src/model.rs`

- [ ] Serialize selected widget to clipboard
- [ ] Paste as new widget with new UUID
- [ ] Keyboard shortcuts (Ctrl+C, Ctrl+V)

### Priority 4: Code Generation

#### Task: Write Generated Code to Disk
**Complexity:** Easy | **Files:** `src/ui.rs`, `src/compiler.rs`

- [ ] Add "Export Project" button to Output panel
- [ ] Use `rfd::FileDialog` to pick output directory
- [ ] Write `Cargo.toml`, `src/main.rs`, `src/app.rs`
- [ ] Show success/error feedback

#### Task: Add Event Action Code Injection
**Complexity:** Hard | **Files:** `src/widgets.rs`, `src/compiler.rs`

- [ ] Parse `clicked_code` field and inject into codegen
- [ ] Handle variable references in code snippets
- [ ] Validate Rust syntax (basic)

#### Task: Support Custom Project Names
**Complexity:** Easy | **Files:** `src/model.rs`, `src/compiler.rs`, `src/ui.rs`

- [ ] Add `project_name: String` to `ProjectState`
- [ ] Use in `Cargo.toml` generation
- [ ] Add editable field in Output panel

### Priority 5: Testing & Polish

#### Task: Add Integration Tests
**Complexity:** Medium | **Files:** `tests/`

- [ ] Test save/load round-trip fidelity
- [ ] Test code generation produces valid Rust
- [ ] Test widget tree manipulation

#### Task: Add Keyboard Navigation
**Complexity:** Medium | **Files:** `src/ui.rs`

- [ ] Arrow keys to navigate hierarchy
- [ ] Enter to select, Escape to deselect
- [ ] Tab to cycle through panels

---

## Development Workflow

1. **Pick a task** from the list above
2. **Read the relevant files** to understand current implementation
3. **Make minimal, focused changes** - avoid over-engineering
4. **Test manually** with `cargo run`
5. **Run `cargo clippy`** to check for issues
6. **Commit with descriptive message**

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
}
```

---

## Reference Documentation

- **Full Architecture**: See `Rust RAD Utility Development Plan.md`
- **Progress Log**: See `GEMINI.md`
- **egui Docs**: https://docs.rs/egui
- **quote Docs**: https://docs.rs/quote
- **typetag Docs**: https://docs.rs/typetag

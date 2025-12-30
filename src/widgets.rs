use crate::model::WidgetNode;
use egui::Ui;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// ... existing imports
// Ensure you import ButtonWidget and the necessary macros

/// Create a widget by its display name
pub fn create_widget_by_name(name: &str) -> Option<Box<dyn WidgetNode>> {
    Some(match name {
        "Button" => Box::new(ButtonWidget::default()),
        "Label" => Box::new(LabelWidget::default()),
        "Text Edit" => Box::new(TextEditWidget::default()),
        "Checkbox" => Box::new(CheckboxWidget::default()),
        "Slider" => Box::new(SliderWidget::default()),
        "Progress Bar" => Box::new(ProgressBarWidget::default()),
        "ComboBox" => Box::new(ComboBoxWidget::default()),
        "Image" => Box::new(ImageWidget::default()),
        "Vertical Layout" => Box::new(VerticalLayout::default()),
        "Horizontal Layout" => Box::new(HorizontalLayout::default()),
        "Grid Layout" => Box::new(GridLayout::default()),
        "Freeform Layout" => Box::new(FreeformLayout::default()),
        "Separator" => Box::new(SeparatorWidget::default()),
        "Spinner" => Box::new(SpinnerWidget::default()),
        "Hyperlink" => Box::new(HyperlinkWidget::default()),
        "Color Picker" => Box::new(ColorPickerWidget::default()),
        "Table" => Box::new(TableWidget::default()),
        "Plot" => Box::new(PlotWidget::default()),
        "Scroll Area" => Box::new(ScrollAreaWidget::default()),
        "Tab Container" => Box::new(TabContainerWidget::default()),
        "Window" => Box::new(WindowWidget::default()),
        _ => return None,
    })
}

// === Gizmo Helper Functions ===

const GIZMO_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);
const DROP_ZONE_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
const HANDLE_SIZE: f32 = 8.0;

/// Draw a selection gizmo (orange outline) around a widget
fn draw_gizmo(ui: &egui::Ui, rect: egui::Rect) {
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, GIZMO_COLOR),
        egui::StrokeKind::Outside,
    );
}

/// Draw a drop zone indicator (dashed blue border) when dragging over a container
fn draw_drop_zone_indicator(ui: &egui::Ui, rect: egui::Rect) {
    // Draw a glowing border effect
    let expanded = rect.expand(2.0);
    ui.painter().rect_stroke(
        expanded,
        4.0,
        egui::Stroke::new(2.0, DROP_ZONE_COLOR),
        egui::StrokeKind::Outside,
    );
    // Draw inner glow
    let fill_color = egui::Color32::from_rgba_unmultiplied(100, 200, 255, 20);
    ui.painter().rect_filled(rect, 4.0, fill_color);
}

/// Draw resize handles at the corners and edges of a rect
fn draw_resize_handles(ui: &egui::Ui, rect: egui::Rect) {
    let handles = [
        (rect.left_top(), "nw"),
        (rect.center_top(), "n"),
        (rect.right_top(), "ne"),
        (rect.right_center(), "e"),
        (rect.right_bottom(), "se"),
        (rect.center_bottom(), "s"),
        (rect.left_bottom(), "sw"),
        (rect.left_center(), "w"),
    ];

    for (pos, _label) in handles {
        let handle_rect = egui::Rect::from_center_size(pos, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
        ui.painter().rect_filled(handle_rect, 0.0, GIZMO_COLOR);
        ui.painter().rect_stroke(
            handle_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
}

/// Check if mouse is hovering over a resize handle and return which one
fn check_resize_handle(_ui: &egui::Ui, rect: egui::Rect, mouse_pos: egui::Pos2) -> Option<&'static str> {
    let handles = [
        (rect.left_top(), "nw"),
        (rect.center_top(), "n"),
        (rect.right_top(), "ne"),
        (rect.right_center(), "e"),
        (rect.right_bottom(), "se"),
        (rect.center_bottom(), "s"),
        (rect.left_bottom(), "sw"),
        (rect.left_center(), "w"),
    ];

    for (pos, label) in handles {
        let handle_rect = egui::Rect::from_center_size(pos, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
        if handle_rect.contains(mouse_pos) {
            return Some(label);
        }
    }
    None
}

/// Handle multi-selection: Ctrl/Cmd toggles, normal click clears and selects
fn handle_selection(ui: &egui::Ui, widget_id: Uuid, response_clicked: bool, selection: &mut HashSet<Uuid>) {
    if !response_clicked {
        return;
    }

    ui.input(|i| {
        if i.modifiers.command {
            // Ctrl/Cmd + click: toggle selection
            if selection.contains(&widget_id) {
                selection.remove(&widget_id);
            } else {
                selection.insert(widget_id);
            }
        } else {
            // Normal click: clear and select only this widget
            selection.clear();
            selection.insert(widget_id);
        }
    });
}

/// Context menu action for widgets
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ContextMenuAction {
    None,
    Delete,
    Duplicate,
    MoveUp,
    MoveDown,
    WrapInVertical,
    WrapInHorizontal,
}

/// Create an interaction overlay for more reliable selection hit detection
/// This adds an expanded clickable area around the widget rect
/// Returns both the response and any context menu action triggered
fn create_selection_overlay(ui: &mut egui::Ui, rect: egui::Rect, widget_id: Uuid) -> egui::Response {
    // Expand the rect slightly for easier clicking
    let expanded_rect = rect.expand(4.0);

    // Create an invisible interaction layer over the widget
    let id = egui::Id::new("select_overlay").with(widget_id);
    let response = ui.interact(expanded_rect, id, egui::Sense::click_and_drag());

    // Add context menu on right-click
    response.clone().context_menu(|ui| {
        render_widget_context_menu(ui, widget_id);
    });

    response
}

/// Render context menu items for a widget
fn render_widget_context_menu(ui: &mut egui::Ui, _widget_id: Uuid) {
    if ui.button("✕ Delete").clicked() {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::Delete));
        ui.close();
    }
    if ui.button("⎘ Duplicate").clicked() {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::Duplicate));
        ui.close();
    }
    ui.separator();
    if ui.button("↑ Move Up").clicked() {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::MoveUp));
        ui.close();
    }
    if ui.button("↓ Move Down").clicked() {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::MoveDown));
        ui.close();
    }
    ui.separator();
    ui.menu_button("⊞ Wrap in...", |ui| {
        if ui.button("Vertical Layout").clicked() {
            ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::WrapInVertical));
            ui.close();
        }
        if ui.button("Horizontal Layout").clicked() {
            ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("context_action"), ContextMenuAction::WrapInHorizontal));
            ui.close();
        }
    });
}

/// Create a container selection overlay that only responds to clicks in the border area
/// This allows child widgets to be selected without also selecting the parent
fn create_container_selection_overlay(ui: &mut egui::Ui, outer_rect: egui::Rect, inner_margin: f32, widget_id: Uuid) -> bool {
    // Only respond to clicks in the border area (outer rect minus inner content rect)
    let inner_rect = outer_rect.shrink(inner_margin.max(8.0));
    let id = egui::Id::new("container_select_overlay").with(widget_id);

    // Check if mouse is in border area (outside inner rect but inside outer rect)
    if let Some(mouse_pos) = ui.ctx().pointer_hover_pos() {
        if outer_rect.expand(4.0).contains(mouse_pos) && !inner_rect.shrink(4.0).contains(mouse_pos) {
            // Mouse is in border area - create interaction
            let response = ui.interact(outer_rect.expand(4.0), id, egui::Sense::click_and_drag());

            // Add context menu on right-click
            response.clone().context_menu(|ui| {
                render_widget_context_menu(ui, widget_id);
            });

            return response.clicked();
        }
    }
    false
}

/// Render an action editor in the Inspector
fn render_action_editor(ui: &mut egui::Ui, action: &mut crate::model::Action, known_variables: &[String]) {
    use crate::model::Action;

    let action_ptr = action as *const _ as usize;

    let action_type = match action {
        Action::IncrementVariable(_) => "Increment",
        Action::SetVariable(_, _) => "Set",
        Action::Custom(_) => "Custom",
    };

    ui.horizontal(|ui| {
        ui.label("Action Type:");
        let mut selected = action_type.to_string();
        egui::ComboBox::from_id_salt(format!("action_{}", action_ptr))
            .selected_text(&selected)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, "Increment".to_string(), "Increment");
                ui.selectable_value(&mut selected, "Set".to_string(), "Set");
                ui.selectable_value(&mut selected, "Custom".to_string(), "Custom");
            });

        // Change action type if needed
        if selected != action_type {
            match selected.as_str() {
                "Increment" => *action = Action::IncrementVariable(
                    known_variables.first().cloned().unwrap_or_default()
                ),
                "Set" => *action = Action::SetVariable(
                    known_variables.first().cloned().unwrap_or_default(),
                    "".to_string(),
                ),
                "Custom" => *action = Action::Custom(String::new()),
                _ => {}
            }
        }
    });

    // Handle each action type with separate variables to avoid borrow conflicts
    let mut should_update_inc_var = false;
    let mut inc_var_new_value = String::new();

    let mut should_update_set_var = false;
    let mut set_var_new_value = String::new();

    let mut should_update_set_value = false;
    let mut set_value_new_value = String::new();

    match action {
        Action::IncrementVariable(var_name) => {
            ui.horizontal(|ui| {
                ui.label("Variable:");
                let mut selected_var = var_name.clone();
                egui::ComboBox::from_id_salt(format!("inc_var_{}", action_ptr))
                    .selected_text(&selected_var)
                    .show_ui(ui, |ui| {
                        for var in known_variables {
                            ui.selectable_value(&mut selected_var, var.clone(), var);
                        }
                    });
                if selected_var != *var_name {
                    inc_var_new_value = selected_var;
                    should_update_inc_var = true;
                }
            });
        }
        Action::SetVariable(var_name, value) => {
            ui.horizontal(|ui| {
                ui.label("Variable:");
                let mut selected_var = var_name.clone();
                egui::ComboBox::from_id_salt(format!("set_var_{}", action_ptr))
                    .selected_text(&selected_var)
                    .show_ui(ui, |ui| {
                        for var in known_variables {
                            ui.selectable_value(&mut selected_var, var.clone(), var);
                        }
                    });
                if selected_var != *var_name {
                    set_var_new_value = selected_var;
                    should_update_set_var = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Value:");
                if ui.text_edit_singleline(value).changed() {
                    should_update_set_value = true;
                    set_value_new_value = value.clone();
                }
            });
        }
        Action::Custom(code) => {
            ui.label("Rust Code:");
            let code_editor = egui::TextEdit::multiline(code)
                .code_editor()
                .desired_rows(3)
                .desired_width(f32::INFINITY);
            ui.add(code_editor);

            if !code.trim().is_empty() {
                if code.parse::<proc_macro2::TokenStream>().is_ok() {
                    ui.colored_label(egui::Color32::GREEN, "✓ Valid Rust syntax");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ Invalid Rust syntax");
                }
            }
        }
    }

    // Apply updates after the match to avoid borrow conflicts
    if should_update_inc_var {
        if let Action::IncrementVariable(var_name) = action {
            *var_name = inc_var_new_value;
        }
    }
    if should_update_set_var {
        if let Action::SetVariable(var_name, _) = action {
            *var_name = set_var_new_value;
        }
    }
}

/// Alignment options for layout containers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayoutAlignment {
    Start,
    Center,
    End,
}

impl Default for LayoutAlignment {
    fn default() -> Self {
        Self::Start
    }
}

/// A container that arranges children vertically.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerticalLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub spacing: f32,
    #[serde(default)]
    pub padding: f32,
    #[serde(default)]
    pub min_width: Option<f32>,
    #[serde(default)]
    pub max_width: Option<f32>,
    #[serde(default)]
    pub alignment: LayoutAlignment,
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            spacing: 5.0,
            padding: 0.0,
            min_width: None,
            max_width: None,
            alignment: LayoutAlignment::Start,
        }
    }
}

#[typetag::serde]
impl WidgetNode for VerticalLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            spacing: self.spacing,
            padding: self.padding,
            min_width: self.min_width,
            max_width: self.max_width,
            alignment: self.alignment,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Vertical Layout"
    }

    // RECURSION: Render children inside a vertical layout
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        // Apply layout alignment
        let layout = match self.alignment {
            LayoutAlignment::Start => egui::Layout::top_down(egui::Align::LEFT),
            LayoutAlignment::Center => egui::Layout::top_down(egui::Align::Center),
            LayoutAlignment::End => egui::Layout::top_down(egui::Align::RIGHT),
        };

        // Wrap the entire layout in a drop zone so you can drop anywhere on it
        let (response, payload_option) = ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::same(self.padding.min(127.0) as i8))
                .show(ui, |ui| {
                    // Apply size constraints
                    if let Some(min_w) = self.min_width {
                        ui.set_min_width(min_w);
                    }
                    if let Some(max_w) = self.max_width {
                        ui.set_max_width(max_w);
                    }

                    ui.with_layout(layout, |ui| {
                        ui.spacing_mut().item_spacing.y = self.spacing;
                        for child in &mut self.children {
                            child.render_editor(ui, selection);
                        }

                        // Render ghost preview if dragging
                        if let Some(dragged_type) = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type"))) {
                            // Only show if hovering this layout (heuristic: if pointer is inside min_rect)
                            // Note: dnd_drop_zone expands, so min_rect covers the area.
                            // We use a simplified check here since dnd_drop_zone doesn't expose hover state during closure.
                            if ui.rect_contains_pointer(ui.min_rect()) {
                                ui.add_space(self.spacing);
                                ui.vertical(|ui| {
                                    ui.disable(); // Make it look like a ghost
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Copy); // Indicate copy/add
                                    render_widget_preview(ui, &dragged_type, egui::Color32::WHITE);
                                });
                            }
                        }
                    });
                }).response
        });

        // Handle container selection only via border (not content area where children are)
        let widget_rect = response.response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, self.padding.max(8.0), self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        // Gizmo (Outline) if selected
        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        // Handle Drop
        if let Some(payload) = payload_option {
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(new_widget) = create_widget_by_name(&payload) {
                    self.children.push(new_widget);
                }
            }
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Vertical Layout Settings");
        ui.label(format!("ID: {}", self.id));

        ui.separator();
        ui.label("Layout Options:");

        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.5).range(0.0..=50.0));
            reset_button(ui, &mut self.spacing, 5.0);
        });

        ui.horizontal(|ui| {
            ui.label("Padding:");
            ui.add(egui::DragValue::new(&mut self.padding).speed(0.5).range(0.0..=50.0));
            reset_button(ui, &mut self.padding, 0.0);
        });

        ui.horizontal(|ui| {
            ui.label("Alignment:");
            egui::ComboBox::from_id_salt("vlayout_align")
                .selected_text(match self.alignment {
                    LayoutAlignment::Start => "Left",
                    LayoutAlignment::Center => "Center",
                    LayoutAlignment::End => "Right",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.alignment, LayoutAlignment::Start, "Left");
                    ui.selectable_value(&mut self.alignment, LayoutAlignment::Center, "Center");
                    ui.selectable_value(&mut self.alignment, LayoutAlignment::End, "Right");
                });
        });

        ui.separator();
        ui.label("Size Constraints:");

        ui.horizontal(|ui| {
            let mut has_min = self.min_width.is_some();
            if ui.checkbox(&mut has_min, "Min Width:").changed() {
                self.min_width = if has_min { Some(100.0) } else { None };
            }
            if let Some(ref mut min_w) = self.min_width {
                ui.add(egui::DragValue::new(min_w).speed(1.0).range(0.0..=1000.0));
            }
        });

        ui.horizontal(|ui| {
            let mut has_max = self.max_width.is_some();
            if ui.checkbox(&mut has_max, "Max Width:").changed() {
                self.max_width = if has_max { Some(500.0) } else { None };
            }
            if let Some(ref mut max_w) = self.max_width {
                ui.add(egui::DragValue::new(max_w).speed(1.0).range(0.0..=2000.0));
            }
        });

        ui.separator();
        ui.label(format!("Children count: {}", self.children.len()));
    }

    // ... VerticalLayout ...

    // RECURSION: Generate code for the layout and all children
    fn codegen(&self) -> proc_macro2::TokenStream {
        // 1. Generate token streams for all children
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();

        // 2. Wrap them in the egui vertical builder
        quote! {
            ui.vertical(|ui| {
                #(#child_streams)*
            });
        }
    }

    // Expose children for the Hierarchy View (Tree Walker)
    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A container that arranges children horizontally.
#[derive(Debug, Serialize, Deserialize)]
pub struct HorizontalLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub spacing: f32,
}

impl Default for HorizontalLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for HorizontalLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            spacing: self.spacing,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Horizontal Layout"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let (response, payload_option) = ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = self.spacing;
                for child in &mut self.children {
                    child.render_editor(ui, selection);
                }

                // Render ghost preview if dragging
                if let Some(dragged_type) = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type"))) {
                    if ui.rect_contains_pointer(ui.min_rect()) {
                        ui.add_space(self.spacing);
                        ui.vertical(|ui| { // Wrap in vertical to keep height consistent or horizontal?
                            // Horizontal layout adds items horizontally.
                            // ui.vertical inside horizontal works but might look odd if it expands height.
                            // But render_widget_preview typically renders a block.
                            ui.disable();
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                            render_widget_preview(ui, &dragged_type, egui::Color32::WHITE);
                        });
                    }
                }
            }).response
        });

        // Handle container selection only via border (not content area where children are)
        let widget_rect = response.response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 8.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        if let Some(payload) = payload_option {
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(new_widget) = create_widget_by_name(&payload) {
                    self.children.push(new_widget);
                }
            }
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Horizontal Layout Settings");
        ui.label(format!("ID: {}", self.id));
        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.1));
            reset_button(ui, &mut self.spacing, 5.0);
        });
        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();
        quote! {
            ui.horizontal(|ui| {
                #(#child_streams)*
            });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A container that arranges children in a grid with a specified number of columns.
#[derive(Debug, Serialize, Deserialize)]
pub struct GridLayout {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub columns: usize,
    pub spacing: f32,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            columns: 2, // Default to 2 columns
            spacing: 5.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for GridLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            columns: self.columns,
            spacing: self.spacing,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Grid Layout"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let (response, payload_option) = ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(self.spacing, self.spacing);

                // Render children in grid format
                let mut row_children = Vec::new();
                let total_children = self.children.len();
                for (idx, child) in self.children.iter_mut().enumerate() {
                    row_children.push(child);

                    // When we reach the column count or the last child, render the row
                    if (idx + 1) % self.columns == 0 || idx == total_children - 1 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = self.spacing;
                            for child in row_children.drain(..) {
                                child.render_editor(ui, selection);
                            }
                        });
                    }
                }

                // Render ghost preview if dragging
                if let Some(dragged_type) = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type"))) {
                    if ui.rect_contains_pointer(ui.min_rect()) {
                        // For grid, just append at the bottom for now
                        ui.horizontal(|ui| {
                            ui.disable();
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                            render_widget_preview(ui, &dragged_type, egui::Color32::WHITE);
                        });
                    }
                }
            }).response
        });

        // Handle container selection only via border (not content area where children are)
        let widget_rect = response.response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 8.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        if let Some(payload) = payload_option {
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(new_widget) = create_widget_by_name(&payload) {
                    self.children.push(new_widget);
                }
            }
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Grid Layout Settings");
        ui.label(format!("ID: {}", self.id));
        ui.horizontal(|ui| {
            ui.label("Columns:");
            ui.add(egui::DragValue::new(&mut self.columns).speed(1.0).range(1..=10));
            reset_button(ui, &mut self.columns, 2);
        });
        ui.horizontal(|ui| {
            ui.label("Spacing:");
            ui.add(egui::DragValue::new(&mut self.spacing).speed(0.1));
            reset_button(ui, &mut self.spacing, 5.0);
        });
        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let columns = self.columns;

        // Group children into rows
        let mut row_streams = Vec::new();
        let mut current_row = Vec::new();

        for (idx, child) in self.children.iter().enumerate() {
            current_row.push(child.codegen());

            // When we reach the column count or the last child, complete the row
            if (idx + 1) % columns == 0 || idx == self.children.len() - 1 {
                let row_widgets = current_row.drain(..).collect::<Vec<_>>();
                row_streams.push(quote! {
                    ui.horizontal(|ui| {
                        #(#row_widgets)*
                    });
                });
            }
        }

        quote! {
            ui.vertical(|ui| {
                #(#row_streams)*
            });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

/// A concrete implementation of a Button.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonWidget {
    pub id: Uuid,
    pub text: String,

    // Maps event type to action
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,

    // Maps property name (e.g. "text") to variable name (e.g. "counter")
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ButtonWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "Click Me".to_string(),
            events: std::collections::HashMap::new(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ButtonWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Button"
    }

    // Render logic for the Editor Canvas
    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Use styled button with selection indicator
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };

        // Allocate rect first for reliable hit detection
        let desired_size = egui::vec2(ui.available_width().min(150.0), 28.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw button visuals
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.text,
            egui::FontId::default(),
            text_color,
        );

        // Handle selection directly from response
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip on hover
        response.on_hover_text(format!("Button: {}\nID: {}", self.text, self.id));
    }

    // The "Inspectable" pattern: The widget defines its own property UI.
    // [cite: 137]
    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Button Properties");
        ui.label(format!("ID: {}", self.id));

        ui.horizontal(|ui| {
            ui.label("Label Text:");

            // Check binding status
            let is_bound = self.bindings.contains_key("text");
            let mut bound_mode = is_bound;

            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    // Switch to bound: set default if empty
                    if !known_variables.is_empty() {
                        self.bindings
                            .insert("text".to_string(), known_variables[0].clone());
                    } else {
                        self.bindings.insert("text".to_string(), "".to_string());
                    }
                } else {
                    self.bindings.remove("text");
                }
            }

            if bound_mode {
                let current_var = self.bindings.get("text").cloned().unwrap_or_default();
                let mut selected_var = current_var.clone();

                egui::ComboBox::from_id_salt("btn_txt_bind")
                    .selected_text(&selected_var)
                    .show_ui(ui, |ui| {
                        for var in known_variables {
                            ui.selectable_value(&mut selected_var, var.clone(), var);
                        }
                    });

                if selected_var != current_var {
                    self.bindings.insert("text".to_string(), selected_var);
                }
            } else {
                ui.text_edit_singleline(&mut self.text);
                reset_button(ui, &mut self.text, "Click Me".to_string());
            }
        });

        ui.separator();
        ui.heading("Events");

        // List all possible events
        let possible_events = [
            crate::model::WidgetEvent::Clicked,
            crate::model::WidgetEvent::DoubleClicked,
            crate::model::WidgetEvent::Hovered,
        ];

        let mut events_to_add = None;
        let mut events_to_remove = None;

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
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    // Generating the AST for the final Rust application.
    // [cite: 184]
    fn codegen(&self) -> proc_macro2::TokenStream {
        use crate::model::WidgetEvent;

        let label_tokens = if let Some(var_name) = self.bindings.get("text") {
            let ident = quote::format_ident!("{}", var_name);
            quote! { &self.#ident }
        } else {
            let text = &self.text;
            quote! { #text }
        };

        // Generate code for the clicked event if present
        let clicked_code = if let Some(action) = self.events.get(&WidgetEvent::Clicked) {
            action.to_code()
        } else {
            quote! {}
        };

        // Generate code for the hovered event if present
        let hovered_code = if let Some(action) = self.events.get(&WidgetEvent::Hovered) {
            let action_code = action.to_code();
            quote! {
                if response.hovered() {
                    #action_code
                }
            }
        } else {
            quote! {}
        };

        quote! {
            let response = ui.button(#label_tokens);
            if response.clicked() {
                #clicked_code
            }
            #hovered_code
        }
    }
}

// ===================== HELPERS =====================

/// Helper to render a small reset button if the value differs from default
pub fn reset_button<T: PartialEq + Clone>(ui: &mut Ui, value: &mut T, default: T) {
    if *value != default {
        if ui.small_button("↺").on_hover_text("Reset to default").clicked() {
            *value = default;
        }
    }
}

/// Render a preview of a widget for drag-and-drop visualization
pub fn render_widget_preview(ui: &mut Ui, widget_type: &str, accent_color: egui::Color32) {
    use crate::theme;
    ui.set_max_width(150.0);

    match widget_type {
        "Button" => {
            let _ = ui.button("Click Me");
        }
        "Label" => {
            ui.label("Label Text");
        }
        "Text Edit" => {
            let mut preview_text = "Enter text...".to_string();
            ui.add(egui::TextEdit::singleline(&mut preview_text).desired_width(120.0));
        }
        "Checkbox" => {
            let mut checked = true;
            ui.checkbox(&mut checked, "Checkbox");
        }
        "Slider" => {
            let mut value = 0.5f32;
            ui.add(egui::Slider::new(&mut value, 0.0..=1.0).show_value(false));
        }
        "Progress Bar" => {
            ui.add(egui::ProgressBar::new(0.6).desired_width(120.0));
        }
        "ComboBox" => {
            let selected = "Option 1".to_string();
            egui::ComboBox::from_id_salt("preview_combo")
                .selected_text(&selected)
                .width(100.0)
                .show_ui(ui, |_ui| {});
        }
        "Image" => {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(100))
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("🖼").size(24.0));
                });
        }
        "Vertical Layout" | "Horizontal Layout" | "Grid Layout" => {
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(theme::WidgetLabels::get(widget_type)).small());
                });
        }
        "Separator" => {
            ui.separator();
        }
        "Spinner" => {
            ui.add(egui::Spinner::new());
        }
        "Hyperlink" => {
            ui.hyperlink_to("Link", "");
        }
        "Color Picker" => {
            let mut color = [0.3f32, 0.6, 0.9];
            ui.color_edit_button_rgb(&mut color);
        }
        "Freeform Layout" => {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(40))
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    // Draw a small grid preview
                    let (_id, rect) = ui.allocate_space(egui::vec2(60.0, 40.0));
                    let painter = ui.painter();
                    for i in 0..4 {
                        let x = rect.left() + (i as f32) * 15.0;
                        painter.line_segment(
                            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                    }
                    for i in 0..3 {
                        let y = rect.top() + (i as f32) * 15.0;
                        painter.line_segment(
                            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                    }
                });
        }
        "Scroll Area" | "Tab Container" | "Window" | "Table" | "Plot" => {
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(theme::WidgetLabels::get(widget_type)).small());
                });
        }
        _ => {
            ui.label(widget_type);
        }
    }
}

// --- Label ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LabelWidget {
    pub id: Uuid,
    pub text: String,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for LabelWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "Label".to_string(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for LabelWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Label"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Calculate size needed for the label
        let galley = ui.fonts(|f| {
            f.layout_no_wrap(self.text.clone(), egui::FontId::default(), ui.style().visuals.text_color())
        });
        let desired_size = galley.size() + egui::vec2(4.0, 4.0);

        // Allocate rect with click sensing for selection
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw label text
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };
        ui.painter().text(
            rect.left_center() + egui::vec2(2.0, 0.0),
            egui::Align2::LEFT_CENTER,
            &self.text,
            egui::FontId::default(),
            text_color,
        );

        // Handle selection directly from response
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Label: {}\nID: {}", self.text, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Label Properties");
        ui.horizontal(|ui| {
            ui.label("Text:");
            // Simplified binding logic for prototype
            let is_bound = self.bindings.contains_key("text");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("text".to_string(), first);
                } else {
                    self.bindings.remove("text");
                }
            }
            if bound_mode {
                let current = self.bindings.get("text").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("lbl_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("text".to_string(), selected);
                }
            } else {
                ui.text_edit_singleline(&mut self.text);
                reset_button(ui, &mut self.text, "Label".to_string());
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let content = if let Some(var) = self.bindings.get("text") {
            let ident = quote::format_ident!("{}", var);
            quote! { &self.#ident }
        } else {
            let t = &self.text;
            quote! { #t }
        };
        quote! { ui.label(#content); }
    }
}

// --- TextEdit ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextEditWidget {
    pub id: Uuid,
    pub text: String, // Fallback if not bound
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for TextEditWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "".to_string(),
            events: std::collections::HashMap::new(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for TextEditWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Text Edit"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Allocate rect for text field visual
        let desired_size = egui::vec2(ui.available_width().min(200.0), 24.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw text field visuals (looks like text_edit but non-interactive)
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            ui.style().visuals.extreme_bg_color,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );

        // Draw text content
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };
        let display_text = if self.text.is_empty() { "..." } else { &self.text };
        ui.painter().text(
            rect.left_center() + egui::vec2(6.0, 0.0),
            egui::Align2::LEFT_CENTER,
            display_text,
            egui::FontId::default(),
            text_color,
        );

        // Handle selection
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Text Edit: {}\nID: {}", self.text, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Text Edit Properties");
        ui.label("Note: Binds to String variables.");
        ui.horizontal(|ui| {
            ui.label("Bind Value:");
            let is_bound = self.bindings.contains_key("value");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("value".to_string(), first);
                } else {
                    self.bindings.remove("value");
                }
            }
            if bound_mode {
                let current = self.bindings.get("value").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("txt_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("value".to_string(), selected);
                }
            }
        });

        ui.separator();
        ui.heading("Events");

        let mut events_to_add = None;
        let mut events_to_remove = None;

        // TextEdit supports Changed, Focused, and LostFocus events
        let possible_events = [
            crate::model::WidgetEvent::Changed,
            crate::model::WidgetEvent::Focused,
            crate::model::WidgetEvent::LostFocus,
        ];

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
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        use crate::model::WidgetEvent;

        if let Some(var) = self.bindings.get("value") {
            let ident = quote::format_ident!("{}", var);

            let changed_code = if let Some(action) = self.events.get(&WidgetEvent::Changed) {
                action.to_code()
            } else {
                quote! {}
            };

            let focused_code = if let Some(action) = self.events.get(&WidgetEvent::Focused) {
                let action_code = action.to_code();
                quote! {
                    if response.gained_focus() {
                        #action_code
                    }
                }
            } else {
                quote! {}
            };

            let lost_focus_code = if let Some(action) = self.events.get(&WidgetEvent::LostFocus) {
                let action_code = action.to_code();
                quote! {
                    if response.lost_focus() {
                        #action_code
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                let response = ui.text_edit_singleline(&mut self.#ident);
                if response.changed() {
                    #changed_code
                }
                #focused_code
                #lost_focus_code
            }
        } else {
            // If not bound, it's just a placeholder or needs local state we don't track well in codegen yet
            quote! { ui.label("Unbound TextEdit"); }
        }
    }

    fn validate(&self, variables: &std::collections::HashMap<String, crate::model::Variable>) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(var_name) = self.bindings.get("value") {
            if let Some(var) = variables.get(var_name) {
                if var.v_type != crate::model::VariableType::String {
                    errors.push(format!("TextEdit '{}' bound to non-string variable '{}' ({})", self.id, var_name, var.v_type));
                }
            } else {
                errors.push(format!("TextEdit '{}' bound to missing variable '{}'", self.id, var_name));
            }
        }
        errors
    }
}

// --- Checkbox ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckboxWidget {
    pub id: Uuid,
    pub label: String,
    pub checked: bool,
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for CheckboxWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            label: "Check me".to_string(),
            checked: false,
            events: std::collections::HashMap::new(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for CheckboxWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Checkbox"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Calculate size for checkbox + label
        let galley = ui.fonts(|f| {
            f.layout_no_wrap(self.label.clone(), egui::FontId::default(), ui.style().visuals.text_color())
        });
        let checkbox_size = 18.0;
        let spacing = 4.0;
        let desired_size = egui::vec2(checkbox_size + spacing + galley.size().x + 4.0, checkbox_size.max(galley.size().y) + 4.0);

        // Allocate rect with click sensing
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw checkbox visual
        let checkbox_rect = egui::Rect::from_min_size(
            rect.left_center() - egui::vec2(0.0, checkbox_size / 2.0),
            egui::vec2(checkbox_size, checkbox_size),
        );
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            checkbox_rect,
            2.0,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );

        // Draw checkmark if checked
        if self.checked {
            ui.painter().text(
                checkbox_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::default(),
                ui.style().visuals.text_color(),
            );
        }

        // Draw label
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };
        ui.painter().text(
            egui::pos2(checkbox_rect.right() + spacing, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &self.label,
            egui::FontId::default(),
            text_color,
        );

        // Handle selection
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Checkbox: {} ({})\nID: {}", self.label, if self.checked { "✓" } else { "☐" }, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Checkbox Properties");
        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.text_edit_singleline(&mut self.label);
        });
        ui.horizontal(|ui| {
            ui.label("Bind Checked (Bool):");
            let is_bound = self.bindings.contains_key("checked");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("checked".to_string(), first);
                } else {
                    self.bindings.remove("checked");
                }
            }
            if bound_mode {
                let current = self.bindings.get("checked").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("chk_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("checked".to_string(), selected);
                }
            }
        });

        ui.separator();
        ui.heading("Events");

        let mut events_to_add = None;
        let mut events_to_remove = None;

        let event = crate::model::WidgetEvent::Changed;
        if self.events.contains_key(&event) {
            ui.collapsing(format!("{}", event), |ui| {
                if let Some(action) = self.events.get_mut(&event) {
                    render_action_editor(ui, action, known_variables);
                }
                if ui.button("Remove Event").clicked() {
                    events_to_remove = Some(event);
                }
            });
        } else {
            if ui.button(format!("+ Add {}", event)).clicked() {
                events_to_add = Some(event);
            }
        }

        if let Some(event) = events_to_add {
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        use crate::model::WidgetEvent;

        let label = &self.label;
        if let Some(var) = self.bindings.get("checked") {
            let ident = quote::format_ident!("{}", var);
            let changed_code = if let Some(action) = self.events.get(&WidgetEvent::Changed) {
                action.to_code()
            } else {
                quote! {}
            };
            quote! {
                if ui.checkbox(&mut self.#ident, #label).changed() {
                    #changed_code
                }
            }
        } else {
            let val = self.checked;
            quote! {
                let mut temp = #val;
                ui.checkbox(&mut temp, #label);
            }
        }
    }

    fn validate(&self, variables: &std::collections::HashMap<String, crate::model::Variable>) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(var_name) = self.bindings.get("checked") {
            if let Some(var) = variables.get(var_name) {
                if var.v_type != crate::model::VariableType::Boolean {
                    errors.push(format!("Checkbox '{}' bound to non-boolean variable '{}' ({})", self.id, var_name, var.v_type));
                }
            } else {
                errors.push(format!("Checkbox '{}' bound to missing variable '{}'", self.id, var_name));
            }
        }
        errors
    }
}

// --- Slider ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SliderWidget {
    pub id: Uuid,
    pub min: f64,
    pub max: f64,
    pub value: f64,
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for SliderWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            min: 0.0,
            max: 100.0,
            value: 50.0,
            events: std::collections::HashMap::new(),
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for SliderWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Slider"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Allocate rect for slider visual
        let desired_size = egui::vec2(ui.available_width().min(200.0), 20.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw slider track
        let track_height = 4.0;
        let track_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(rect.width() - 20.0, track_height),
        );
        ui.painter().rect(
            track_rect,
            2.0,
            ui.style().visuals.widgets.inactive.bg_fill,
            egui::Stroke::NONE,
            egui::StrokeKind::Inside,
        );

        // Draw slider handle position based on value
        let normalized = (self.value - self.min) / (self.max - self.min);
        let handle_x = track_rect.left() + normalized as f32 * track_rect.width();
        let handle_rect = egui::Rect::from_center_size(
            egui::pos2(handle_x, rect.center().y),
            egui::vec2(12.0, 16.0),
        );
        let handle_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.widgets.active.bg_fill
        };
        ui.painter().rect(handle_rect, 3.0, handle_color, egui::Stroke::NONE, egui::StrokeKind::Inside);

        // Draw value text
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };
        ui.painter().text(
            egui::pos2(rect.right() - 5.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("{:.1}", self.value),
            egui::FontId::proportional(10.0),
            text_color,
        );

        // Handle selection
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Slider: {} ({}-{})\nID: {}", self.value, self.min, self.max, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Slider Properties");
        ui.horizontal(|ui| {
            ui.label("Min:");
            ui.add(egui::DragValue::new(&mut self.min));
            reset_button(ui, &mut self.min, 0.0);
            ui.label("Max:");
            ui.add(egui::DragValue::new(&mut self.max));
            reset_button(ui, &mut self.max, 100.0);
        });
        ui.horizontal(|ui| {
            ui.label("Bind Value (Num):");
            let is_bound = self.bindings.contains_key("value");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("value".to_string(), first);
                } else {
                    self.bindings.remove("value");
                }
            }
            if bound_mode {
                let current = self.bindings.get("value").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("sld_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("value".to_string(), selected);
                }
            }
        });

        ui.separator();
        ui.heading("Events");

        let mut events_to_add = None;
        let mut events_to_remove = None;

        let event = crate::model::WidgetEvent::Changed;
        if self.events.contains_key(&event) {
            ui.collapsing(format!("{}", event), |ui| {
                if let Some(action) = self.events.get_mut(&event) {
                    render_action_editor(ui, action, known_variables);
                }
                if ui.button("Remove Event").clicked() {
                    events_to_remove = Some(event);
                }
            });
        } else {
            if ui.button(format!("+ Add {}", event)).clicked() {
                events_to_add = Some(event);
            }
        }

        if let Some(event) = events_to_add {
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        use crate::model::WidgetEvent;

        let min = self.min;
        let max = self.max;
        if let Some(var) = self.bindings.get("value") {
            let ident = quote::format_ident!("{}", var);
            let changed_code = if let Some(action) = self.events.get(&WidgetEvent::Changed) {
                action.to_code()
            } else {
                quote! {}
            };
            // Use `as _` to allow the compiler to infer the correct numeric type (f64, f32, i32, etc)
            // for the range limits based on the variable's type.
            quote! {
                if ui.add(egui::Slider::new(&mut self.#ident, (#min as _)..=(#max as _))).changed() {
                    #changed_code
                }
            }
        } else {
            let val = self.value;
            quote! {
                let mut temp = #val;
                ui.add(egui::Slider::new(&mut temp, #min..=#max));
            }
        }
    }

    fn validate(&self, variables: &std::collections::HashMap<String, crate::model::Variable>) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(var_name) = self.bindings.get("value") {
            if let Some(var) = variables.get(var_name) {
                match var.v_type {
                    crate::model::VariableType::Integer | crate::model::VariableType::Float => {},
                    _ => errors.push(format!("Slider '{}' bound to non-numeric variable '{}' ({})", self.id, var_name, var.v_type)),
                }
            } else {
                errors.push(format!("Slider '{}' bound to missing variable '{}'", self.id, var_name));
            }
        }
        errors
    }
}

// --- ProgressBar ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressBarWidget {
    pub id: Uuid,
    pub value: f32, // 0.0 to 1.0
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ProgressBarWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            value: 0.5,
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ProgressBarWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Progress Bar"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Allocate rect for progress bar
        let desired_size = egui::vec2(ui.available_width().min(200.0), 20.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // Draw progress bar background
        let bg_color = ui.style().visuals.widgets.inactive.bg_fill;
        ui.painter().rect(rect, 3.0, bg_color, egui::Stroke::NONE, egui::StrokeKind::Inside);

        // Draw progress fill
        let fill_width = rect.width() * self.value;
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
        let fill_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            egui::Color32::from_rgb(100, 150, 255)
        };
        ui.painter().rect(fill_rect, 3.0, fill_color, egui::Stroke::NONE, egui::StrokeKind::Inside);

        // Draw percentage text
        let text = format!("{:.0}%", self.value * 100.0);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::default(),
            ui.style().visuals.text_color(),
        );

        // Handle selection
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("Progress Bar: {:.0}%\nID: {}", self.value * 100.0, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Progress Bar Properties");
        ui.horizontal(|ui| {
            ui.label("Progress (0.0 - 1.0):");
            ui.add(egui::Slider::new(&mut self.value, 0.0..=1.0));
            reset_button(ui, &mut self.value, 0.5);
        });
        ui.horizontal(|ui| {
            ui.label("Bind Value:");
            let is_bound = self.bindings.contains_key("value");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("value".to_string(), first);
                } else {
                    self.bindings.remove("value");
                }
            }
            if bound_mode {
                let current = self.bindings.get("value").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("progress_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("value".to_string(), selected);
                }
            }
        });
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        if let Some(var) = self.bindings.get("value") {
            let ident = quote::format_ident!("{}", var);
            quote! { ui.add(egui::ProgressBar::new(self.#ident as f32).show_percentage()); }
        } else {
            let val = self.value;
            quote! { ui.add(egui::ProgressBar::new(#val).show_percentage()); }
        }
    }

    fn validate(&self, variables: &std::collections::HashMap<String, crate::model::Variable>) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(var_name) = self.bindings.get("value") {
            if let Some(var) = variables.get(var_name) {
                match var.v_type {
                    crate::model::VariableType::Integer | crate::model::VariableType::Float => {},
                    _ => errors.push(format!("ProgressBar '{}' bound to non-numeric variable '{}' ({})", self.id, var_name, var.v_type)),
                }
            } else {
                errors.push(format!("ProgressBar '{}' bound to missing variable '{}'", self.id, var_name));
            }
        }
        errors
    }
}

// --- ComboBox ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComboBoxWidget {
    pub id: Uuid,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
}

impl Default for ComboBoxWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            label: "Select:".to_string(),
            options: vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()],
            selected: 0,
            bindings: std::collections::HashMap::new(),
            events: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ComboBoxWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "ComboBox"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let is_selected = selection.contains(&self.id);

        // Calculate size for label + dropdown visual
        let label_galley = ui.fonts(|f| {
            f.layout_no_wrap(self.label.clone(), egui::FontId::default(), ui.style().visuals.text_color())
        });
        let selected_text = self.options.get(self.selected).map(|s| s.as_str()).unwrap_or("...");
        let dropdown_width = 120.0;
        let total_width = label_galley.size().x + 8.0 + dropdown_width;
        let height = 24.0;

        // Allocate rect with click sensing
        let (rect, response) = ui.allocate_exact_size(egui::vec2(total_width, height), egui::Sense::click());

        // Draw label
        let text_color = if is_selected {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            ui.style().visuals.text_color()
        };
        ui.painter().text(
            rect.left_center() + egui::vec2(2.0, 0.0),
            egui::Align2::LEFT_CENTER,
            &self.label,
            egui::FontId::default(),
            text_color,
        );

        // Draw dropdown visual
        let dropdown_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + label_galley.size().x + 8.0, rect.top()),
            egui::vec2(dropdown_width, height),
        );
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            dropdown_rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );

        // Draw selected text
        ui.painter().text(
            dropdown_rect.left_center() + egui::vec2(6.0, 0.0),
            egui::Align2::LEFT_CENTER,
            selected_text,
            egui::FontId::default(),
            text_color,
        );

        // Draw dropdown arrow
        ui.painter().text(
            dropdown_rect.right_center() - egui::vec2(14.0, 0.0),
            egui::Align2::CENTER_CENTER,
            "▼",
            egui::FontId::proportional(10.0),
            text_color,
        );

        // Handle selection
        handle_selection(ui, self.id, response.clicked(), selection);

        // Add context menu
        response.context_menu(|ui| {
            render_widget_context_menu(ui, self.id);
        });

        if is_selected {
            draw_gizmo(ui, rect);
        }

        // Show tooltip
        response.on_hover_text(format!("ComboBox: {}\nSelected: {}\nID: {}", self.label, selected_text, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("ComboBox Properties");

        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.text_edit_singleline(&mut self.label);
        });

        ui.separator();
        ui.label("Options:");

        let mut to_remove = None;
        for (idx, opt) in self.options.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(opt);
                if ui.button("🗑").clicked() {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            self.options.remove(idx);
            if self.selected >= self.options.len() && !self.options.is_empty() {
                self.selected = self.options.len() - 1;
            }
        }

        if ui.button("+ Add Option").clicked() {
            self.options.push(format!("Option {}", self.options.len() + 1));
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Bind Selected Index:");
            let is_bound = self.bindings.contains_key("selected");
            let mut bound_mode = is_bound;
            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    let first = known_variables.first().cloned().unwrap_or_default();
                    self.bindings.insert("selected".to_string(), first);
                } else {
                    self.bindings.remove("selected");
                }
            }
            if bound_mode {
                let current = self.bindings.get("selected").cloned().unwrap_or_default();
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("combo_bind")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for v in known_variables {
                            ui.selectable_value(&mut selected, v.clone(), v);
                        }
                    });
                if selected != current {
                    self.bindings.insert("selected".to_string(), selected);
                }
            }
        });

        ui.separator();
        ui.heading("Events");

        let mut events_to_add = None;
        let mut events_to_remove = None;

        // ComboBox supports Focused and LostFocus events
        let possible_events = [
            crate::model::WidgetEvent::Focused,
            crate::model::WidgetEvent::LostFocus,
        ];

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
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let label = &self.label;
        let options: Vec<_> = self.options.iter().map(|s| s.as_str()).collect();

        if let Some(var) = self.bindings.get("selected") {
            let ident = quote::format_ident!("{}", var);
            quote! {
                ui.horizontal(|ui| {
                    ui.label(#label);
                    let options = vec![#(#options),*];
                    let selected_text = options.get(self.#ident).unwrap_or(&"");
                    egui::ComboBox::from_label("")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (idx, opt) in options.iter().enumerate() {
                                ui.selectable_value(&mut self.#ident, idx, opt);
                            }
                        });
                });
            }
        } else {
            let selected = self.selected;
            quote! {
                ui.horizontal(|ui| {
                    ui.label(#label);
                    let mut selected = #selected;
                    let options = vec![#(#options),*];
                    let selected_text = options.get(selected).unwrap_or(&"");
                    egui::ComboBox::from_label("")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (idx, opt) in options.iter().enumerate() {
                                ui.selectable_value(&mut selected, idx, opt);
                            }
                        });
                });
            }
        }
    }
}

// --- Image ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageWidget {
    pub id: Uuid,
    pub path: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// Optional reference to an asset by name (from AssetManager)
    #[serde(default)]
    pub asset_name: Option<String>,
    /// Cached filename from the asset (for codegen)
    #[serde(default)]
    pub asset_filename: Option<String>,
    #[serde(default)]
    pub events: std::collections::HashMap<crate::model::WidgetEvent, crate::model::Action>,
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            path: "".to_string(),
            width: Some(100.0),
            height: None, // Maintain aspect ratio
            asset_name: None,
            asset_filename: None,
            events: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ImageWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Image"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = if self.path.is_empty() {
            ui.label("[No Image]")
        } else {
            ui.label(format!("🖼 {}", self.path.split('/').last().unwrap_or(&self.path)))
        };
        let widget_rect = response.rect;

        // Create selection overlay for better hit detection
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);

        let is_selected = selection.contains(&self.id);
        if is_selected {
            draw_gizmo(ui, widget_rect);
            draw_resize_handles(ui, widget_rect);

            // Handle resize dragging
            if let Some(hover_pos) = ui.ctx().pointer_hover_pos() {
                if let Some(handle) = check_resize_handle(ui, widget_rect, hover_pos) {
                    // Show resize cursor based on handle direction
                    let cursor = match handle {
                        "nw" | "se" => egui::CursorIcon::ResizeNwSe,
                        "ne" | "sw" => egui::CursorIcon::ResizeNeSw,
                        "n" | "s" => egui::CursorIcon::ResizeVertical,
                        "e" | "w" => egui::CursorIcon::ResizeHorizontal,
                        _ => egui::CursorIcon::Default,
                    };
                    ui.ctx().set_cursor_icon(cursor);

                    // If dragging, resize the image
                    if ui.input(|i| i.pointer.primary_down()) {
                        let delta = ui.input(|i| i.pointer.delta());

                        // Handle horizontal resize
                        match handle {
                            "e" | "ne" | "se" => {
                                if let Some(w) = self.width.as_mut() {
                                    *w = (*w + delta.x).max(10.0);
                                }
                            }
                            "w" | "nw" | "sw" => {
                                if let Some(w) = self.width.as_mut() {
                                    *w = (*w - delta.x).max(10.0);
                                }
                            }
                            _ => {}
                        }

                        // Handle vertical resize
                        match handle {
                            "s" | "se" | "sw" => {
                                if let Some(h) = self.height.as_mut() {
                                    *h = (*h + delta.y).max(10.0);
                                }
                            }
                            "n" | "nw" | "ne" => {
                                if let Some(h) = self.height.as_mut() {
                                    *h = (*h - delta.y).max(10.0);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Show tooltip with image info
        let size_info = match (self.width, self.height) {
            (Some(w), Some(h)) => format!(" ({}x{})", w, h),
            (Some(w), None) => format!(" (w:{})", w),
            (None, Some(h)) => format!(" (h:{})", h),
            (None, None) => "".to_string(),
        };
        overlay.on_hover_text(format!("Image{}\nPath: {}\nID: {}", size_info, self.path, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], known_assets: &[(String, String)]) {
        ui.heading("Image Properties");

        // Asset selection dropdown
        if !known_assets.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Asset:");
                let current_asset = self.asset_name.clone().unwrap_or_else(|| "(None)".to_string());
                egui::ComboBox::from_id_salt("image_asset_select")
                    .selected_text(&current_asset)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(self.asset_name.is_none(), "(None - use path)").clicked() {
                            self.asset_name = None;
                            self.asset_filename = None;
                        }
                        for (asset_name, asset_filename) in known_assets {
                            if ui.selectable_label(self.asset_name.as_ref() == Some(asset_name), asset_name).clicked() {
                                self.asset_name = Some(asset_name.clone());
                                self.asset_filename = Some(asset_filename.clone());
                            }
                        }
                    });
            });
            ui.add_space(4.0);
        }

        // Manual path entry (shown when no asset is selected)
        if self.asset_name.is_none() {
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.path);
            });

            if ui.button("📁 Browse...").clicked() {
                if let Some(path) = crate::io::pick_file("Images") {
                    if let Some(path_str) = path.to_str() {
                        self.path = path_str.to_string();
                    }
                }
            }
        } else {
            ui.label(format!("Using asset: {}", self.asset_name.as_ref().unwrap()));
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Width:");
            let mut has_width = self.width.is_some();
            if ui.checkbox(&mut has_width, "").changed() {
                self.width = if has_width { Some(100.0) } else { None };
            }
            if let Some(ref mut w) = self.width {
                ui.add(egui::DragValue::new(w).speed(1.0).range(10.0..=1000.0));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            let mut has_height = self.height.is_some();
            if ui.checkbox(&mut has_height, "").changed() {
                self.height = if has_height { Some(100.0) } else { None };
            }
            if let Some(ref mut h) = self.height {
                ui.add(egui::DragValue::new(h).speed(1.0).range(10.0..=1000.0));
            }
        });

        ui.separator();
        ui.heading("Events");

        // List possible events for Image (Hovered is useful for tooltips, effects)
        let possible_events = [
            crate::model::WidgetEvent::Hovered,
        ];

        let mut events_to_add = None;
        let mut events_to_remove = None;

        for event in &possible_events {
            if self.events.contains_key(event) {
                ui.collapsing(format!("{}", event), |ui| {
                    if let Some(action) = self.events.get_mut(event) {
                        render_action_editor(ui, action, &[]);
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
            self.events.insert(event, crate::model::Action::Custom(String::new()));
        }

        if let Some(event) = events_to_remove {
            self.events.remove(&event);
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        use crate::model::WidgetEvent;

        // Use asset path if an asset is selected, otherwise use manual path
        let path = if let Some(ref filename) = self.asset_filename {
            format!("assets/{}", filename)
        } else {
            self.path.clone()
        };

        let size_tokens = match (self.width, self.height) {
            (Some(w), Some(h)) => quote! { .max_size(egui::vec2(#w, #h)) },
            (Some(w), None) => quote! { .max_width(#w) },
            (None, Some(h)) => quote! { .max_height(#h) },
            (None, None) => quote! {},
        };

        // Generate code for the hovered event if present
        let hovered_code = if let Some(action) = self.events.get(&WidgetEvent::Hovered) {
            let action_code = action.to_code();
            quote! {
                if response.hovered() {
                    #action_code
                }
            }
        } else {
            quote! {}
        };

        quote! {
            let response = ui.add(
                egui::Image::new(#path)
                    #size_tokens
            );
            #hovered_code
        }
    }
}

// === NEW SIMPLE WIDGETS ===

// --- Separator ---
/// A visual separator line for layouts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeparatorWidget {
    pub id: Uuid,
}

impl Default for SeparatorWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for SeparatorWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Separator"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.separator();
        let widget_rect = response.rect;

        // Create selection overlay for better hit detection
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        overlay.on_hover_text(format!("Separator\nID: {}", self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Separator");
        ui.label("A visual divider between widgets.");
        ui.label(format!("ID: {}", self.id));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        quote! { ui.separator(); }
    }
}

// --- Spinner ---
/// A loading spinner indicator
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpinnerWidget {
    pub id: Uuid,
    pub size: f32,
}

impl Default for SpinnerWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            size: 20.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for SpinnerWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Spinner"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.add(egui::Spinner::new().size(self.size));
        let widget_rect = response.rect;

        // Create selection overlay for better hit detection
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        overlay.on_hover_text(format!("Spinner (size: {})\nID: {}", self.size, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Spinner Properties");
        ui.horizontal(|ui| {
            ui.label("Size:");
            ui.add(egui::DragValue::new(&mut self.size).speed(1.0).range(10.0..=100.0));
        });
        ui.label(format!("ID: {}", self.id));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let size = self.size;
        quote! { ui.add(egui::Spinner::new().size(#size)); }
    }
}

// --- Hyperlink ---
/// A clickable hyperlink
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HyperlinkWidget {
    pub id: Uuid,
    pub text: String,
    pub url: String,
}

impl Default for HyperlinkWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: "Click here".to_string(),
            url: "https://example.com".to_string(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for HyperlinkWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Hyperlink"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.hyperlink_to(&self.text, &self.url);
        let widget_rect = response.rect;

        // Create selection overlay for better hit detection
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        overlay.on_hover_text(format!("Hyperlink: {}\nURL: {}\nID: {}", self.text, self.url, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Hyperlink Properties");
        ui.horizontal(|ui| {
            ui.label("Text:");
            ui.text_edit_singleline(&mut self.text);
        });
        ui.horizontal(|ui| {
            ui.label("URL:");
            ui.text_edit_singleline(&mut self.url);
        });
        ui.label(format!("ID: {}", self.id));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let text = &self.text;
        let url = &self.url;
        quote! { ui.hyperlink_to(#text, #url); }
    }
}

// --- Window Container ---
/// A window container widget that represents an egui::Window
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowWidget {
    pub id: Uuid,
    pub title: String,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub closeable: bool,
    pub collapsible: bool,
    pub resizable: bool,
    pub default_width: f32,
    pub default_height: Option<f32>,
}

impl Default for WindowWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Window".to_string(),
            children: Vec::new(),
            closeable: true,
            collapsible: true,
            resizable: true,
            default_width: 300.0,
            default_height: Some(200.0),
        }
    }
}

#[typetag::serde]
impl WidgetNode for WindowWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            title: self.title.clone(),
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            closeable: self.closeable,
            collapsible: self.collapsible,
            resizable: self.resizable,
            default_width: self.default_width,
            default_height: self.default_height,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Window"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        // In the editor, we render the window as a styled frame
        // since actual egui::Window needs ctx-level access
        let frame = egui::Frame::new()
            .fill(ui.style().visuals.window_fill)
            .stroke(ui.style().visuals.window_stroke)
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(8));

        let response = frame.show(ui, |ui| {
            // Window title bar
            ui.horizontal(|ui| {
                ui.strong(&self.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.closeable {
                        ui.label("✕");
                    }
                    if self.collapsible {
                        ui.label("−");
                    }
                });
            });
            ui.separator();

            // Window content
            for child in &mut self.children {
                child.render_editor(ui, selection);
            }

            // Drop zone for adding widgets
            let (_response, payload_option) =
                ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
                    ui.label("Drag widget here to add...");
                });

            if let Some(payload) = payload_option {
                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(new_widget) = create_widget_by_name(&payload) {
                        self.children.push(new_widget);
                    }
                }
            }
        }).response;

        // Make the window container selectable only via border (not content area)
        let widget_rect = response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 8.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Window Properties");
        ui.label(format!("ID: {}", self.id));

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });

        ui.separator();
        ui.label("Window Options:");

        ui.checkbox(&mut self.closeable, "Closeable");
        ui.checkbox(&mut self.collapsible, "Collapsible");
        ui.checkbox(&mut self.resizable, "Resizable");

        ui.separator();
        ui.label("Size:");

        ui.horizontal(|ui| {
            ui.label("Default Width:");
            ui.add(egui::DragValue::new(&mut self.default_width).speed(1.0).range(100.0..=1000.0));
        });

        ui.horizontal(|ui| {
            ui.label("Default Height:");
            let mut has_height = self.default_height.is_some();
            if ui.checkbox(&mut has_height, "").changed() {
                self.default_height = if has_height { Some(200.0) } else { None };
            }
            if let Some(ref mut h) = self.default_height {
                ui.add(egui::DragValue::new(h).speed(1.0).range(50.0..=1000.0));
            }
        });

        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let title = &self.title;
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();

        // Generate a state variable name from the window title
        let state_var = quote::format_ident!("window_{}_open", self.id.to_string().replace("-", "_"));

        let collapsible = self.collapsible;
        let resizable = self.resizable;
        let default_width = self.default_width;

        let height_token = if let Some(h) = self.default_height {
            quote! { .default_height(#h) }
        } else {
            quote! {}
        };

        // Note: The window state variable needs to be added to the app struct
        // This codegen assumes the app will have `window_XXX_open: bool` field
        quote! {
            egui::Window::new(#title)
                .open(&mut self.#state_var)
                .collapsible(#collapsible)
                .resizable(#resizable)
                .default_width(#default_width)
                #height_token
                .show(ctx, |ui| {
                    #(#child_streams)*
                });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

// --- TabContainer ---
/// A tabbed container for organizing content
#[derive(Debug, Serialize, Deserialize)]
pub struct TabContainerWidget {
    pub id: Uuid,
    pub tabs: Vec<TabItem>,
    pub selected_tab: usize,
}

/// A single tab with a name and children
#[derive(Debug, Serialize, Deserialize)]
pub struct TabItem {
    pub name: String,
    pub children: Vec<Box<dyn WidgetNode>>,
}

impl Clone for TabItem {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            children: self.children.iter().map(|c| c.clone_box()).collect(),
        }
    }
}

impl Default for TabContainerWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            tabs: vec![
                TabItem {
                    name: "Tab 1".to_string(),
                    children: Vec::new(),
                },
                TabItem {
                    name: "Tab 2".to_string(),
                    children: Vec::new(),
                },
            ],
            selected_tab: 0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for TabContainerWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            tabs: self.tabs.clone(),
            selected_tab: self.selected_tab,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Tab Container"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.vertical(|ui| {
            // Tab bar
            ui.horizontal(|ui| {
                for (idx, tab) in self.tabs.iter().enumerate() {
                    let is_selected = idx == self.selected_tab;
                    let text = if is_selected {
                        egui::RichText::new(&tab.name).strong()
                    } else {
                        egui::RichText::new(&tab.name)
                    };
                    if ui.selectable_label(is_selected, text).clicked() {
                        self.selected_tab = idx;
                    }
                }
            });

            ui.separator();

            // Tab content
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                for child in &mut tab.children {
                    child.render_editor(ui, selection);
                }

                // Drop zone for adding widgets
                let (_response, payload_option) =
                    ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
                        ui.label("Drag widget here to add...");
                    });

                if let Some(payload) = payload_option {
                    if ui.input(|i| i.pointer.any_released()) {
                        if let Some(new_widget) = create_widget_by_name(&payload) {
                            tab.children.push(new_widget);
                        }
                    }
                }
            }
        }).response;

        // Make the tab container selectable only via border (not content area)
        let widget_rect = response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 8.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Tab Container Settings");
        ui.label(format!("ID: {}", self.id));

        ui.separator();
        ui.label("Tabs:");

        let mut tab_to_remove = None;
        let mut new_selected_tab = None;
        let tab_count = self.tabs.len();

        for idx in 0..self.tabs.len() {
            let is_selected = idx == self.selected_tab;

            ui.horizontal(|ui| {
                if ui.selectable_label(is_selected, "").clicked() {
                    new_selected_tab = Some(idx);
                }

                ui.text_edit_singleline(&mut self.tabs[idx].name);

                if ui.button("🗑").clicked() && tab_count > 1 {
                    tab_to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = new_selected_tab {
            self.selected_tab = idx;
        }

        if let Some(idx) = tab_to_remove {
            self.tabs.remove(idx);
            if self.selected_tab >= self.tabs.len() {
                self.selected_tab = self.tabs.len().saturating_sub(1);
            }
        }

        if ui.button("+ Add Tab").clicked() {
            self.tabs.push(TabItem {
                name: format!("Tab {}", self.tabs.len() + 1),
                children: Vec::new(),
            });
        }

        if let Some(tab) = self.tabs.get(self.selected_tab) {
            ui.label(format!("Children in '{}': {}", tab.name, tab.children.len()));
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let tab_count = self.tabs.len();

        let tab_names: Vec<_> = self.tabs.iter().map(|t| &t.name).collect();
        let tab_contents: Vec<_> = self.tabs.iter().map(|tab| {
            let child_streams: Vec<_> = tab.children.iter().map(|c| c.codegen()).collect();
            quote! {
                #(#child_streams)*
            }
        }).collect();

        let tab_indices: Vec<_> = (0..tab_count).collect();

        quote! {
            ui.horizontal(|ui| {
                #(
                    if ui.selectable_label(self.selected_tab == #tab_indices, #tab_names).clicked() {
                        self.selected_tab = #tab_indices;
                    }
                )*
            });
            ui.separator();
            match self.selected_tab {
                #(
                    #tab_indices => {
                        #tab_contents
                    }
                )*
                _ => {}
            }
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        // Return children of the currently selected tab
        self.tabs.get(self.selected_tab).map(|tab| &tab.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        // Return children of the currently selected tab
        self.tabs.get_mut(self.selected_tab).map(|tab| &mut tab.children)
    }
}

// --- ScrollArea ---
/// A scrollable container widget
#[derive(Debug, Serialize, Deserialize)]
pub struct ScrollAreaWidget {
    pub id: Uuid,
    pub children: Vec<Box<dyn WidgetNode>>,
    pub scroll_horizontal: bool,
    pub scroll_vertical: bool,
    pub max_height: Option<f32>,
    pub max_width: Option<f32>,
}

impl Default for ScrollAreaWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            scroll_horizontal: false,
            scroll_vertical: true,
            max_height: Some(200.0),
            max_width: None,
        }
    }
}

#[typetag::serde]
impl WidgetNode for ScrollAreaWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| c.clone_box()).collect(),
            scroll_horizontal: self.scroll_horizontal,
            scroll_vertical: self.scroll_vertical,
            max_height: self.max_height,
            max_width: self.max_width,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Scroll Area"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let scroll_area = egui::ScrollArea::new([self.scroll_horizontal, self.scroll_vertical])
            .max_height(self.max_height.unwrap_or(f32::INFINITY))
            .max_width(self.max_width.unwrap_or(f32::INFINITY));

        let response = scroll_area
            .show(ui, |ui| {
                for child in &mut self.children {
                    child.render_editor(ui, selection);
                }

                // Drop Zone for adding widgets
                let (_response, payload_option) =
                    ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
                        ui.label("Drag widget here to add...");
                    });

                if let Some(payload) = payload_option {
                    if ui.input(|i| i.pointer.any_released()) {
                        if let Some(new_widget) = create_widget_by_name(&payload) {
                            self.children.push(new_widget);
                        }
                    }
                }
            })
            .inner_rect;

        // Make the scroll area selectable only via border (not content area)
        let widget_rect = response;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 8.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Scroll Area Settings");
        ui.label(format!("ID: {}", self.id));

        ui.separator();

        ui.checkbox(&mut self.scroll_horizontal, "Horizontal Scroll");
        ui.checkbox(&mut self.scroll_vertical, "Vertical Scroll");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Max Height:");
            let mut has_max_height = self.max_height.is_some();
            if ui.checkbox(&mut has_max_height, "").changed() {
                self.max_height = if has_max_height { Some(200.0) } else { None };
            }
            if let Some(ref mut h) = self.max_height {
                ui.add(egui::DragValue::new(h).speed(1.0).range(50.0..=1000.0));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max Width:");
            let mut has_max_width = self.max_width.is_some();
            if ui.checkbox(&mut has_max_width, "").changed() {
                self.max_width = if has_max_width { Some(300.0) } else { None };
            }
            if let Some(ref mut w) = self.max_width {
                ui.add(egui::DragValue::new(w).speed(1.0).range(50.0..=1000.0));
            }
        });

        ui.label(format!("Children count: {}", self.children.len()));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let child_streams: Vec<_> = self.children.iter().map(|c| c.codegen()).collect();

        let h_scroll = self.scroll_horizontal;
        let v_scroll = self.scroll_vertical;

        let max_height_token = if let Some(h) = self.max_height {
            quote! { .max_height(#h) }
        } else {
            quote! {}
        };

        let max_width_token = if let Some(w) = self.max_width {
            quote! { .max_width(#w) }
        } else {
            quote! {}
        };

        quote! {
            egui::ScrollArea::new([#h_scroll, #v_scroll])
                #max_height_token
                #max_width_token
                .show(ui, |ui| {
                    #(#child_streams)*
                });
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        Some(&mut self.children)
    }
}

// --- ColorPicker ---
/// A color selection widget
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColorPickerWidget {
    pub id: Uuid,
    pub color: [f32; 4],
    #[serde(default)]
    pub bindings: std::collections::HashMap<String, String>,
}

impl Default for ColorPickerWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            color: [1.0, 1.0, 1.0, 1.0], // White by default
            bindings: std::collections::HashMap::new(),
        }
    }
}

#[typetag::serde]
impl WidgetNode for ColorPickerWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Color Picker"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let response = ui.color_edit_button_rgba_unmultiplied(&mut self.color);
        let widget_rect = response.rect;

        // Create selection overlay for better hit detection
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        let hex = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            (self.color[0] * 255.0) as u8,
            (self.color[1] * 255.0) as u8,
            (self.color[2] * 255.0) as u8,
            (self.color[3] * 255.0) as u8,
        );
        overlay.on_hover_text(format!("Color: {}\nID: {}", hex, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Color Picker Properties");
        ui.label(format!("ID: {}", self.id));

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Color:");
            let is_bound = self.bindings.contains_key("color");
            let mut bound_mode = is_bound;

            if ui.checkbox(&mut bound_mode, "Bind").changed() {
                if bound_mode {
                    if !known_variables.is_empty() {
                        self.bindings
                            .insert("color".to_string(), known_variables[0].clone());
                    } else {
                        self.bindings.insert("color".to_string(), "".to_string());
                    }
                } else {
                    self.bindings.remove("color");
                }
            }

            if bound_mode {
                let current_var = self.bindings.get("color").cloned().unwrap_or_default();
                let mut selected_var = current_var.clone();

                egui::ComboBox::from_id_salt("color_picker_bind")
                    .selected_text(&selected_var)
                    .show_ui(ui, |ui| {
                        for var in known_variables {
                            ui.selectable_value(&mut selected_var, var.clone(), var);
                        }
                    });

                if selected_var != current_var {
                    self.bindings.insert("color".to_string(), selected_var);
                }
            } else {
                ui.color_edit_button_rgba_unmultiplied(&mut self.color);
            }
        });

        // Display hex color for reference
        let hex = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            (self.color[0] * 255.0) as u8,
            (self.color[1] * 255.0) as u8,
            (self.color[2] * 255.0) as u8,
            (self.color[3] * 255.0) as u8,
        );
        ui.label(format!("Hex: {}", hex));
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let r = (self.color[0] * 255.0) as u8;
        let g = (self.color[1] * 255.0) as u8;
        let b = (self.color[2] * 255.0) as u8;
        let a = (self.color[3] * 255.0) as u8;

        if let Some(var_name) = self.bindings.get("color") {
            let ident = quote::format_ident!("{}", var_name);
            quote! {
                ui.color_edit_button_rgba_unmultiplied(&mut self.#ident);
            }
        } else {
            quote! {
                let mut color = [#r as f32 / 255.0, #g as f32 / 255.0, #b as f32 / 255.0, #a as f32 / 255.0];
                ui.color_edit_button_rgba_unmultiplied(&mut color);
            }
        }
    }
}

// --- FreeformLayout ---
/// A container with absolute positioning for children
/// Each child can have its own x, y position within the container
#[derive(Debug, Serialize, Deserialize)]
pub struct FreeformLayout {
    pub id: Uuid,
    pub children: Vec<FreeformChild>,
    pub width: f32,
    pub height: f32,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

/// A child widget with position data
#[derive(Debug, Serialize, Deserialize)]
pub struct FreeformChild {
    pub widget: Box<dyn WidgetNode>,
    pub x: f32,
    pub y: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Clone for FreeformChild {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone_box(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl Default for FreeformLayout {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            width: 400.0,
            height: 300.0,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 10.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for FreeformLayout {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(Self {
            id: self.id,
            children: self.children.iter().map(|c| FreeformChild {
                widget: c.widget.clone_box(),
                x: c.x,
                y: c.y,
                width: c.width,
                height: c.height,
            }).collect(),
            width: self.width,
            height: self.height,
            show_grid: self.show_grid,
            snap_to_grid: self.snap_to_grid,
            grid_size: self.grid_size,
        })
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Freeform Layout"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        let _container_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(self.width, self.height),
        );

        // Reserve space for the freeform area
        let (response, painter) = ui.allocate_painter(
            egui::vec2(self.width, self.height),
            egui::Sense::click_and_drag(),
        );

        let container_origin = response.rect.min;

        // Background
        painter.rect_filled(
            response.rect,
            0.0,
            if ui.style().visuals.dark_mode {
                egui::Color32::from_gray(35)
            } else {
                egui::Color32::from_gray(250)
            },
        );

        // Draw grid if enabled
        if self.show_grid {
            let grid_color = if ui.style().visuals.dark_mode {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20)
            };

            let mut x = 0.0;
            while x <= self.width {
                painter.line_segment(
                    [
                        container_origin + egui::vec2(x, 0.0),
                        container_origin + egui::vec2(x, self.height),
                    ],
                    egui::Stroke::new(1.0, grid_color),
                );
                x += self.grid_size;
            }

            let mut y = 0.0;
            while y <= self.height {
                painter.line_segment(
                    [
                        container_origin + egui::vec2(0.0, y),
                        container_origin + egui::vec2(self.width, y),
                    ],
                    egui::Stroke::new(1.0, grid_color),
                );
                y += self.grid_size;
            }
        }

        // Border
        painter.rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Inside,
        );

        // Track which child is being dragged
        let mut dragged_child_idx: Option<usize> = None;

        // Render children at their absolute positions
        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_pos = container_origin + egui::vec2(child.x, child.y);

            // Create a child area at the specified position
            let child_id = egui::Id::new("freeform_child").with(child.widget.id());
            let child_area = egui::Area::new(child_id)
                .fixed_pos(child_pos)
                .order(egui::Order::Foreground);

            let area_response = child_area.show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(2))
                    .show(ui, |ui| {
                        child.widget.render_editor(ui, selection);
                    })
            });

            // Check if this child should be dragged
            let drag_id = egui::Id::new("freeform_drag").with(child.widget.id());
            let is_selected = selection.contains(&child.widget.id());

            if is_selected {
                // Allow dragging when selected
                let drag_response = ui.interact(
                    area_response.response.rect,
                    drag_id,
                    egui::Sense::drag(),
                );

                if drag_response.dragged() {
                    dragged_child_idx = Some(idx);
                }
            }
        }

        // Handle drag movement
        if let Some(idx) = dragged_child_idx {
            if let Some(_pointer_pos) = ui.ctx().pointer_interact_pos() {
                let delta = ui.ctx().input(|i| i.pointer.delta());
                let child = &mut self.children[idx];

                child.x += delta.x;
                child.y += delta.y;

                // Snap to grid if enabled
                if self.snap_to_grid {
                    child.x = (child.x / self.grid_size).round() * self.grid_size;
                    child.y = (child.y / self.grid_size).round() * self.grid_size;
                }

                // Clamp to container bounds
                child.x = child.x.max(0.0).min(self.width - 20.0);
                child.y = child.y.max(0.0).min(self.height - 20.0);
            }
        }

        // Handle selection of the container itself only via border (not content area)
        let widget_rect = response.rect;
        let border_clicked = create_container_selection_overlay(ui, widget_rect, 10.0, self.id);
        handle_selection(ui, self.id, border_clicked, selection);

        // Draw drop zone indicator when dragging over
        let is_dragging = ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_widget_type")).is_some());
        if is_dragging && ui.rect_contains_pointer(widget_rect) {
            draw_drop_zone_indicator(ui, widget_rect);
        }

        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }

        // Drop zone for adding widgets
        let drop_rect = egui::Rect::from_min_size(
            container_origin + egui::vec2(0.0, self.height - 24.0),
            egui::vec2(self.width, 24.0),
        );

        let _drop_response = ui.allocate_rect(drop_rect, egui::Sense::hover());

        // Check for drops
        let (_drop_response, payload_option) = ui.dnd_drop_zone::<String, _>(egui::Frame::NONE, |ui| {
            ui.allocate_space(egui::vec2(0.0, 0.0)); // Invisible drop zone
        });

        if let Some(payload) = payload_option {
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(widget) = create_widget_by_name(&payload) {
                    let drop_pos = ui.ctx().pointer_hover_pos().unwrap_or(container_origin);
                    let relative_pos = drop_pos - container_origin;

                    let x = if self.snap_to_grid {
                        (relative_pos.x / self.grid_size).round() * self.grid_size
                    } else {
                        relative_pos.x
                    };
                    let y = if self.snap_to_grid {
                        (relative_pos.y / self.grid_size).round() * self.grid_size
                    } else {
                        relative_pos.y
                    };

                    self.children.push(FreeformChild {
                        widget,
                        x: x.max(0.0),
                        y: y.max(0.0),
                        width: None,
                        height: None,
                    });
                }
            }
        }
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Freeform Layout Properties");
        ui.label(format!("ID: {}", self.id));

        ui.separator();
        ui.label("Container Size:");

        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut self.width).speed(1.0).range(100.0..=2000.0));
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut self.height).speed(1.0).range(100.0..=2000.0));
        });

        ui.separator();
        ui.label("Grid Options:");

        ui.checkbox(&mut self.show_grid, "Show Grid");
        ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");

        ui.horizontal(|ui| {
            ui.label("Grid Size:");
            ui.add(egui::DragValue::new(&mut self.grid_size).speed(1.0).range(5.0..=50.0));
        });

        ui.separator();
        ui.label(format!("Children count: {}", self.children.len()));

        // Show child positions
        if !self.children.is_empty() {
            ui.collapsing("Child Positions", |ui| {
                for (_idx, child) in self.children.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", child.widget.name()));
                        ui.label("x:");
                        ui.add(egui::DragValue::new(&mut child.x).speed(1.0));
                        ui.label("y:");
                        ui.add(egui::DragValue::new(&mut child.y).speed(1.0));
                    });
                }
            });
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        // For code generation, we use egui::Area for absolute positioning
        let child_streams: Vec<_> = self.children.iter().map(|child| {
            let child_code = child.widget.codegen();
            let x = child.x;
            let y = child.y;
            let child_id = format!("freeform_child_{}", child.widget.id());

            quote! {
                egui::Area::new(egui::Id::new(#child_id))
                    .fixed_pos(egui::pos2(#x, #y))
                    .show(ctx, |ui| {
                        #child_code
                    });
            }
        }).collect();

        let width = self.width;
        let height = self.height;

        quote! {
            // Freeform container
            ui.allocate_space(egui::vec2(#width, #height));
            #(#child_streams)*
        }
    }

    fn children(&self) -> Option<&Vec<Box<dyn WidgetNode>>> {
        None // Freeform uses FreeformChild instead of Box<dyn WidgetNode>
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn WidgetNode>>> {
        None
    }
}

// --- Table ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableColumn {
    pub header: String,
    pub width: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableWidget {
    pub id: Uuid,
    pub columns: Vec<TableColumn>,
    pub row_count: usize,
    pub striped: bool,
    pub resizable: bool,
}

impl Default for TableWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            columns: vec![
                TableColumn { header: "ID".to_string(), width: Some(60.0) },
                TableColumn { header: "Name".to_string(), width: None },
                TableColumn { header: "Role".to_string(), width: None },
            ],
            row_count: 5,
            striped: true,
            resizable: true,
        }
    }
}

#[typetag::serde]
impl WidgetNode for TableWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Table"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        // Wrap table in a Frame so we have a bounding box for selection
        let frame = egui::Frame::NONE.inner_margin(0.0);
        let response = frame.show(ui, |ui| {
             let available_height = 150.0; // Fixed height for editor preview
             let mut table = egui_extras::TableBuilder::new(ui)
                .striped(self.striped)
                .resizable(self.resizable)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            for col in &self.columns {
                if let Some(w) = col.width {
                    table = table.column(egui_extras::Column::exact(w));
                } else {
                    table = table.column(egui_extras::Column::initial(100.0).at_least(40.0));
                }
            }
            
            table.header(20.0, |mut header| {
                for col in &self.columns {
                    header.col(|ui| {
                        ui.strong(&col.header);
                    });
                }
             })
             .body(|mut body| {
                 for row_idx in 0..self.row_count {
                     body.row(18.0, |mut row| {
                        for col_idx in 0..self.columns.len() {
                            row.col(|ui| {
                                ui.label(format!("Cell {},{}", row_idx, col_idx));
                            });
                        }
                     });
                 }
             });
        });

        let widget_rect = response.response.rect;
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);
        
        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }
        
        overlay.on_hover_text(format!("Table\nID: {}", self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Table Properties");
        ui.label(format!("ID: {}", self.id));
        ui.separator();
        
        ui.checkbox(&mut self.striped, "Striped Rows");
        ui.checkbox(&mut self.resizable, "Resizable Columns");
        ui.horizontal(|ui| {
            ui.label("Preview Rows:");
             ui.add(egui::DragValue::new(&mut self.row_count).range(0..=100));
        });
        
        ui.separator();
        ui.label("Columns:");
        
        let mut remove_idx = None;
        for (idx, col) in self.columns.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("#{}", idx+1));
                ui.text_edit_singleline(&mut col.header);
                if ui.button("🗑").clicked() {
                    remove_idx = Some(idx);
                }
            });
            ui.horizontal(|ui| {
                 ui.label("Width:");
                 let mut fixed = col.width.is_some();
                 if ui.checkbox(&mut fixed, "Fixed").changed() {
                     col.width = if fixed { Some(100.0) } else { None };
                 }
                 if let Some(w) = &mut col.width {
                      ui.add(egui::DragValue::new(w).speed(1.0).range(10.0..=1000.0));
                 }
            });
            ui.separator();
        }
        
        if let Some(idx) = remove_idx {
            self.columns.remove(idx);
        }
        
        if ui.button("+ Add Column").clicked() {
            self.columns.push(TableColumn { header: "New Col".to_string(), width: None });
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let striped = self.striped;
        let resizable = self.resizable;
        let col_defs: Vec<_> = self.columns.iter().map(|col| {
            if let Some(w) = col.width {
                quote! { .column(egui_extras::Column::exact(#w)) }
            } else {
                quote! { .column(egui_extras::Column::initial(100.0).at_least(40.0)) }
            }
        }).collect();
        
        let headers: Vec<_> = self.columns.iter().map(|col| {
            let h = &col.header;
            quote! {
                header.col(|ui| { ui.strong(#h); });
            }
        }).collect();
        
        let col_count = self.columns.len();
        
        quote! {
            egui_extras::TableBuilder::new(ui)
                .striped(#striped)
                .resizable(#resizable)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0)
                #(#col_defs)*
                .header(20.0, |mut header| {
                    #(#headers)*
                })
                .body(|mut body| {
                    for i in 0..5 {
                        body.row(18.0, |mut row| {
                            for j in 0..#col_count {
                                row.col(|ui| {
                                    ui.label(format!("Cell {},{}", i, j));
                                });
                            }
                        });
                    }
                });
        }
    }
}

// --- Plot ---
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlotType {
    Line,
    Bar,
    Points,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotSeries {
    pub name: String,
    pub plot_type: PlotType,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotWidget {
    pub id: Uuid,
    pub title: String,
    pub show_x_axis: bool,
    pub show_y_axis: bool,
    pub show_legend: bool,
    pub series: Vec<PlotSeries>,
    pub height: f32,
}

impl Default for PlotWidget {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Performance".to_string(),
            show_x_axis: true,
            show_y_axis: true,
            show_legend: true,
            series: vec![
                PlotSeries { name: "Series A".to_string(), plot_type: PlotType::Line, color: Some([0.0, 0.5, 1.0]) },
            ],
            height: 200.0,
        }
    }
}

#[typetag::serde]
impl WidgetNode for PlotWidget {
    fn clone_box(&self) -> Box<dyn WidgetNode> {
        Box::new(self.clone())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Plot"
    }

    fn render_editor(&mut self, ui: &mut Ui, selection: &mut HashSet<Uuid>) {
        use egui_plot::{Line, Plot, PlotPoints};

        let mut plot = Plot::new(format!("plot_{}", self.id))
            .height(self.height)
            .show_x(self.show_x_axis)
            .show_y(self.show_y_axis);
        
        if self.show_legend {
            plot = plot.legend(egui_plot::Legend::default());
        }

        let response = plot.show(ui, |plot_ui| {
            for (idx, s) in self.series.iter().enumerate() {
                let color = s.color.map(|c| egui::Color32::from_rgb(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8
                )).unwrap_or(egui::Color32::from_gray(150));

                match s.plot_type {
                    PlotType::Line => {
                        let sin: PlotPoints = (0..1000).map(|i| {
                            let x = i as f64 * 0.01;
                            [x, (x + (idx as f64)).sin()]
                        }).collect();
                        plot_ui.line(Line::new(&s.name, sin).color(color));
                    }
                    PlotType::Bar => {
                        let bars: Vec<egui_plot::Bar> = (0..10).map(|i| {
                            egui_plot::Bar::new(i as f64, (i as f64 + (idx as f64)).cos().abs() * 5.0)
                        }).collect();
                        plot_ui.bar_chart(egui_plot::BarChart::new(&s.name, bars).color(color));
                    }
                    PlotType::Points => {
                        let points: PlotPoints = (0..50).map(|i| {
                            let x = i as f64 * 0.2;
                            [x, (x * (idx as f64 + 1.0)).cos() * 2.0]
                        }).collect();
                        plot_ui.points(egui_plot::Points::new(&s.name, points).color(color));
                    }
                }
            }
        });

        let widget_rect = response.response.rect;
        let overlay = create_selection_overlay(ui, widget_rect, self.id);
        handle_selection(ui, self.id, overlay.clicked(), selection);
        
        if selection.contains(&self.id) {
            draw_gizmo(ui, widget_rect);
        }
        
        overlay.on_hover_text(format!("Plot: {}\nID: {}", self.title, self.id));
    }

    fn inspect(&mut self, ui: &mut Ui, _known_variables: &[String], _known_assets: &[(String, String)]) {
        ui.heading("Plot Properties");
        ui.label(format!("ID: {}", self.id));
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut self.height).speed(1.0).range(50.0..=1000.0));
        });

        ui.checkbox(&mut self.show_x_axis, "Show X Axis");
        ui.checkbox(&mut self.show_y_axis, "Show Y Axis");
        ui.checkbox(&mut self.show_legend, "Show Legend");

        ui.separator();
        ui.label("Series:");
        
        let mut remove_idx = None;
        for (idx, s) in self.series.iter_mut().enumerate() {
            ui.collapsing(format!("Series #{} ({})", idx + 1, s.name), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut s.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt(format!("plot_type_{}_{}", self.id, idx))
                        .selected_text(format!("{:?}", s.plot_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut s.plot_type, PlotType::Line, "Line");
                            ui.selectable_value(&mut s.plot_type, PlotType::Bar, "Bar");
                            ui.selectable_value(&mut s.plot_type, PlotType::Points, "Points");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color = s.color.unwrap_or([1.0, 1.0, 1.0]);
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        s.color = Some(color);
                    }
                });

                if ui.button("🗑 Remove Series").clicked() {
                    remove_idx = Some(idx);
                }
            });
        }

        if let Some(idx) = remove_idx {
            self.series.remove(idx);
        }

        if ui.button("+ Add Series").clicked() {
            self.series.push(PlotSeries {
                name: format!("Series {}", self.series.len() + 1),
                plot_type: PlotType::Line,
                color: Some([1.0, 1.0, 1.0]),
            });
        }
    }

    fn codegen(&self) -> proc_macro2::TokenStream {
        let title = &self.title;
        let height = self.height;
        let show_x = self.show_x_axis;
        let show_y = self.show_y_axis;
        let show_legend = self.show_legend;

        let mut series_code = Vec::new();
        for (idx, s) in self.series.iter().enumerate() {
            let name = &s.name;
            let color = s.color.unwrap_or([1.0, 1.0, 1.0]);
            let r = color[0];
            let g = color[1];
            let b = color[2];

            match s.plot_type {
                PlotType::Line => {
                    series_code.push(quote! {
                        let sin: egui_plot::PlotPoints = (0..1000).map(|i| {
                            let x = i as f64 * 0.01;
                            [x, (x + (#idx as f64)).sin()]
                        }).collect();
                        plot_ui.line(egui_plot::Line::new(#name, sin).color(egui::Color32::from_rgb_f32(#r, #g, #b)));
                    });
                }
                PlotType::Bar => {
                    series_code.push(quote! {
                        let bars: Vec<egui_plot::Bar> = (0..10).map(|i| {
                            egui_plot::Bar::new(i as f64, (i as f64 + (#idx as f64)).cos().abs() * 5.0)
                        }).collect();
                        plot_ui.bar_chart(egui_plot::BarChart::new(#name, bars).color(egui::Color32::from_rgb_f32(#r, #g, #b)));
                    });
                }
                PlotType::Points => {
                    series_code.push(quote! {
                        let points: egui_plot::PlotPoints = (0..50).map(|i| {
                            let x = i as f64 * 0.2;
                            [x, (x * (#idx as f64 + 1.0)).cos() * 2.0]
                        }).collect();
                        plot_ui.points(egui_plot::Points::new(#name, points).color(egui::Color32::from_rgb_f32(#r, #g, #b)));
                    });
                }
            }
        }

        let mut plot_init = quote! {
            egui_plot::Plot::new(#title)
                .height(#height)
                .show_x(#show_x)
                .show_y(#show_y)
        };

        if show_legend {
            plot_init = quote! { #plot_init.legend(egui_plot::Legend::default()) };
        }

        quote! {
            #plot_init.show(ui, |plot_ui| {
                #(#series_code)*
            });
        }
    }
}

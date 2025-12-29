use crate::model::ProjectState;
use crate::theme::ThemeMode;
use crate::validator::ValidationStatus;
use egui::Vec2;

pub mod assets;
pub mod canvas;
pub mod code_preview;
pub mod hierarchy;
pub mod inspector;
pub mod palette;
pub mod variables;

/// Shared context passed to all UI panels
#[allow(dead_code)]
pub struct EditorContext<'a> {
    pub project_state: &'a mut ProjectState,
    pub validation_status: &'a mut ValidationStatus,
    pub theme_mode: &'a mut ThemeMode,
    pub canvas_zoom: &'a mut f32,
    pub canvas_pan: &'a mut Vec2,
    pub clipboard: &'a mut Option<String>,
}
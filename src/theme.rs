use egui::{Color32, CornerRadius, Stroke};

/// Theme mode enum for light/dark theming
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
    }
}

/// Aether RAD color palette
pub struct AetherColors;

impl AetherColors {
    // Primary accent color (blue)
    pub const ACCENT: Color32 = Color32::from_rgb(66, 150, 250);
    pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(100, 170, 255);
    pub const ACCENT_DARK: Color32 = Color32::from_rgb(45, 120, 210);

    // Selection/Gizmo colors
    pub const SELECTION: Color32 = Color32::from_rgb(255, 165, 0); // Orange

    // Status colors
    pub const SUCCESS: Color32 = Color32::from_rgb(80, 200, 120);
    pub const WARNING: Color32 = Color32::from_rgb(255, 180, 50);
    pub const ERROR: Color32 = Color32::from_rgb(255, 85, 85);

    // Widget category colors
    pub const LAYOUT_COLOR: Color32 = Color32::from_rgb(150, 120, 255); // Purple
    pub const INPUT_COLOR: Color32 = Color32::from_rgb(80, 180, 255); // Blue
    pub const DISPLAY_COLOR: Color32 = Color32::from_rgb(120, 200, 150); // Green

    // Text hierarchy
    pub const HEADING: Color32 = Color32::from_rgb(230, 230, 230);
    pub const SUBHEADING: Color32 = Color32::from_rgb(180, 180, 180);
    pub const MUTED: Color32 = Color32::from_rgb(130, 130, 130);

    // Backgrounds
    pub const PANEL_BG: Color32 = Color32::from_rgb(35, 35, 40);
    pub const SECTION_BG: Color32 = Color32::from_rgb(45, 45, 52);

    // Helper for semi-transparent colors (not const)
    pub fn selection_fill() -> Color32 {
        Color32::from_rgba_unmultiplied(255, 165, 0, 30)
    }

    pub fn drop_zone_hover() -> Color32 {
        Color32::from_rgba_unmultiplied(66, 150, 250, 40)
    }
}

/// Widget type labels
pub struct WidgetLabels;

impl WidgetLabels {
    pub fn get(widget_name: &str) -> &'static str {
        match widget_name {
            // Layouts
            "Vertical Layout" => "Vertical",
            "Horizontal Layout" => "Horizontal",
            "Grid Layout" => "Grid",

            // Inputs
            "Button" => "Button",
            "Checkbox" => "Checkbox",
            "Slider" => "Slider",
            "Text Edit" => "Text Edit",
            "ComboBox" => "ComboBox",

            // Display
            "Label" => "Label",
            "Progress Bar" => "Progress Bar",
            "Image" => "Image",
            "Separator" => "Separator",
            "Spinner" => "Spinner",
            "Hyperlink" => "Hyperlink",

            // Default
            _ => "Widget",
        }
    }

    pub fn get_category_label(category: &str) -> &'static str {
        match category {
            "Layouts" => "Layouts",
            "Inputs" => "Inputs",
            "Display" => "Display",
            _ => "Other",
        }
    }

    /// Get the widget type group (for color coding)
    pub fn get_category(widget_name: &str) -> &'static str {
        match widget_name {
            "Vertical Layout" | "Horizontal Layout" | "Grid Layout" => "Layouts",
            "Button" | "Checkbox" | "Slider" | "Text Edit" | "ComboBox" => "Inputs",
            "Label" | "Progress Bar" | "Image" | "Separator" | "Spinner" | "Hyperlink" => "Display",
            _ => "Other",
        }
    }
}

/// Light mode color palette
pub struct LightModeColors;

impl LightModeColors {
    // Primary accent color (blue)
    pub const ACCENT: Color32 = Color32::from_rgb(45, 120, 210);
    pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(66, 150, 250);
    pub const ACCENT_DARK: Color32 = Color32::from_rgb(30, 80, 180);

    // Selection/Gizmo colors
    pub const SELECTION: Color32 = Color32::from_rgb(255, 165, 0); // Orange

    // Status colors
    pub const SUCCESS: Color32 = Color32::from_rgb(50, 160, 90);
    pub const WARNING: Color32 = Color32::from_rgb(230, 140, 30);
    pub const ERROR: Color32 = Color32::from_rgb(220, 50, 50);

    // Widget category colors
    pub const LAYOUT_COLOR: Color32 = Color32::from_rgb(120, 80, 200); // Purple
    pub const INPUT_COLOR: Color32 = Color32::from_rgb(45, 120, 210); // Blue
    pub const DISPLAY_COLOR: Color32 = Color32::from_rgb(70, 150, 100); // Green

    // Text hierarchy
    pub const HEADING: Color32 = Color32::from_rgb(30, 30, 40);
    pub const SUBHEADING: Color32 = Color32::from_rgb(80, 80, 90);
    pub const MUTED: Color32 = Color32::from_rgb(120, 120, 130);

    // Backgrounds
    pub const PANEL_BG: Color32 = Color32::from_rgb(245, 245, 248);
    pub const SECTION_BG: Color32 = Color32::from_rgb(235, 235, 242);

    // Helper for semi-transparent colors
    pub fn selection_fill() -> Color32 {
        Color32::from_rgba_unmultiplied(255, 165, 0, 30)
    }

    pub fn drop_zone_hover() -> Color32 {
        Color32::from_rgba_unmultiplied(66, 150, 250, 40)
    }
}

/// Configure the egui theme for Aether RAD based on theme mode
pub fn configure_aether_theme(ctx: &egui::Context, mode: ThemeMode) {
    let mut style = (*ctx.style()).clone();

    // Spacing improvements
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.indent = 18.0;

    // Rounded corners for all widgets
    let rounding = CornerRadius::same(6);
    style.visuals.widgets.noninteractive.corner_radius = rounding;
    style.visuals.widgets.inactive.corner_radius = rounding;
    style.visuals.widgets.hovered.corner_radius = rounding;
    style.visuals.widgets.active.corner_radius = rounding;
    style.visuals.widgets.open.corner_radius = rounding;

    // Window/frame rounding
    style.visuals.window_corner_radius = CornerRadius::same(8);
    style.visuals.menu_corner_radius = CornerRadius::same(6);

    match mode {
        ThemeMode::Dark => {
            // Dark theme colors
            style.visuals.selection.bg_fill = AetherColors::ACCENT;
            style.visuals.selection.stroke = Stroke::new(1.0, AetherColors::ACCENT_LIGHT);
            style.visuals.hyperlink_color = AetherColors::ACCENT_LIGHT;

            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 70);
            style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(55, 55, 65);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(50, 50, 60);
            style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(50, 50, 58);
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(55, 55, 65);

            // Dark background
            style.visuals.dark_mode = true;
            style.visuals.panel_fill = Color32::from_rgb(35, 35, 40);
            style.visuals.window_fill = Color32::from_rgb(40, 40, 48);
            style.visuals.faint_bg_color = Color32::from_rgb(25, 25, 30);
        }
        ThemeMode::Light => {
            // Light theme colors
            style.visuals.selection.bg_fill = LightModeColors::ACCENT;
            style.visuals.selection.stroke = Stroke::new(1.0, LightModeColors::ACCENT_LIGHT);
            style.visuals.hyperlink_color = LightModeColors::ACCENT_LIGHT;

            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(240, 240, 245);
            style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(245, 245, 248);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(230, 240, 250);
            style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(245, 245, 248);
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(240, 240, 245);

            // Light background
            style.visuals.dark_mode = false;
            style.visuals.panel_fill = Color32::from_rgb(250, 250, 252);
            style.visuals.window_fill = Color32::from_rgb(248, 248, 250);
            style.visuals.faint_bg_color = Color32::from_rgb(240, 240, 245);
        }
    }

    // Collapsing header styling
    style.visuals.collapsing_header_frame = true;

    // Striped tables
    style.visuals.striped = true;

    ctx.set_style(style);
}

/// Create a styled section frame (theme-aware)
pub fn section_frame(ctx: &egui::Context) -> egui::Frame {
    let is_light = !ctx.style().visuals.dark_mode;
    let bg_color = if is_light {
        LightModeColors::SECTION_BG
    } else {
        AetherColors::SECTION_BG
    };
    let stroke_color = if is_light {
        Color32::from_rgb(200, 200, 210)
    } else {
        Color32::from_rgb(70, 70, 80)
    };

    egui::Frame::new()
        .fill(bg_color)
        .inner_margin(egui::Margin::same(12))
        .outer_margin(egui::Margin::symmetric(0, 6))
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.5, stroke_color))
}

/// Create a panel header frame (theme-aware)
pub fn panel_header_frame(ctx: &egui::Context) -> egui::Frame {
    let is_light = !ctx.style().visuals.dark_mode;
    let bg_color = if is_light {
        Color32::from_rgb(240, 240, 245)
    } else {
        Color32::from_rgb(40, 40, 48)
    };

    egui::Frame::new()
        .fill(bg_color)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(CornerRadius::same(4))
}

/// Styled heading text (theme-aware)
pub fn heading(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(16.0)
        .strong()
}

/// Styled subheading text (theme-aware)
pub fn subheading(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .strong()
}

/// Muted/secondary text (theme-aware)
#[allow(dead_code)]
pub fn muted(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
}

/// Code/monospace text
#[allow(dead_code)]
pub fn code_text(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
        .monospace()
        .color(AetherColors::ACCENT_LIGHT)
}

/// Category label text
#[allow(dead_code)]
pub fn category_label(category: &str) -> egui::RichText {
    egui::RichText::new(WidgetLabels::get_category_label(category))
        .size(13.0)
        .strong()
        .color(AetherColors::SUBHEADING)
}

/// Widget label for hierarchy/palette
pub fn widget_label(widget_name: &str) -> String {
    WidgetLabels::get(widget_name).to_string()
}

/// Get color for widget category
pub fn widget_category_color(widget_name: &str) -> Color32 {
    match widget_name {
        "Vertical Layout" | "Horizontal Layout" | "Grid Layout" => AetherColors::LAYOUT_COLOR,
        "Button" | "Checkbox" | "Slider" | "Text Edit" | "ComboBox" => AetherColors::INPUT_COLOR,
        "Label" | "Progress Bar" | "Image" | "Separator" | "Spinner" | "Hyperlink" => {
            AetherColors::DISPLAY_COLOR
        }
        _ => AetherColors::MUTED,
    }
}

/// Get the muted text color based on current theme
pub fn muted_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        AetherColors::MUTED
    } else {
        LightModeColors::MUTED
    }
}

/// Get the accent light color based on current theme
pub fn accent_light_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        AetherColors::ACCENT_LIGHT
    } else {
        LightModeColors::ACCENT_LIGHT
    }
}

/// Get the error color based on current theme
pub fn error_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        AetherColors::ERROR
    } else {
        LightModeColors::ERROR
    }
}

/// Get the success color based on current theme
pub fn success_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        AetherColors::SUCCESS
    } else {
        LightModeColors::SUCCESS
    }
}

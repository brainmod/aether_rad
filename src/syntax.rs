use egui::text::LayoutJob;
use egui::{Color32, TextFormat};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;

/// Lazily-loaded syntax highlighting system
static SYNTAX_HIGHLIGHTER: OnceLock<SyntaxHighlighter> = OnceLock::new();

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    fn get() -> &'static SyntaxHighlighter {
        SYNTAX_HIGHLIGHTER.get_or_init(Self::new)
    }

    /// Convert syntect Style to egui Color32
    fn style_to_color(style: Style) -> Color32 {
        let color = style.foreground;
        Color32::from_rgb(color.r, color.g, color.b)
    }

    /// Create a LayoutJob with per-token syntax highlighting
    fn highlight_with_layout_job(
        code: &str,
        syntax_name: &str,
        theme_name: &str,
        fallback_color: Color32,
    ) -> LayoutJob {
        let highlighter = Self::get();

        let syntax = match highlighter.syntax_set.find_syntax_by_name(syntax_name) {
            Some(s) => s,
            None => {
                // Fallback: return plain text with fallback color
                let mut job = LayoutJob::default();
                let mut format = TextFormat::default();
                format.color = fallback_color;
                job.append(code, 0.0, format);
                return job;
            }
        };

        let theme = &highlighter.theme_set.themes[theme_name];
        let mut highlight_lines = HighlightLines::new(syntax, theme);
        let mut job = LayoutJob::default();

        for line in code.lines() {
            if let Ok(highlighted) = highlight_lines.highlight_line(line, &highlighter.syntax_set) {
                for (style, text) in highlighted {
                    let color = if style.foreground.a > 0 {
                        Self::style_to_color(style)
                    } else {
                        fallback_color
                    };

                    let mut format = TextFormat::default();
                    format.color = color;
                    format.font_id.family = egui::FontFamily::Monospace;

                    job.append(text, 0.0, format);
                }
            } else {
                let mut format = TextFormat::default();
                format.color = fallback_color;
                format.font_id.family = egui::FontFamily::Monospace;

                job.append(line, 0.0, format);
            }

            // Add newline at end of line
            let mut format = TextFormat::default();
            format.color = fallback_color;
            format.font_id.family = egui::FontFamily::Monospace;

            job.append("\n", 0.0, format);
        }

        job
    }

    /// Highlight Rust code with per-token colors using LayoutJob
    pub fn highlight_rust(code: &str, is_light: bool) -> LayoutJob {
        let theme_name = if is_light {
            "Solarized (light)"
        } else {
            "Solarized (dark)"
        };

        let fallback_color = if is_light {
            Color32::from_rgb(50, 50, 60)
        } else {
            Color32::from_rgb(220, 220, 220)
        };

        Self::highlight_with_layout_job(code, "Rust", theme_name, fallback_color)
    }

    /// Highlight TOML code with per-token colors using LayoutJob
    pub fn highlight_toml(code: &str, is_light: bool) -> LayoutJob {
        let theme_name = if is_light {
            "Solarized (light)"
        } else {
            "Solarized (dark)"
        };

        let fallback_color = if is_light {
            Color32::from_rgb(50, 50, 60)
        } else {
            Color32::from_rgb(220, 220, 220)
        };

        Self::highlight_with_layout_job(code, "TOML", theme_name, fallback_color)
    }
}

/// Highlight Rust code for display in the code preview
pub fn highlight_rust(code: &str, is_light: bool) -> LayoutJob {
    SyntaxHighlighter::highlight_rust(code, is_light)
}

/// Highlight TOML code for display in the code preview
pub fn highlight_toml(code: &str, is_light: bool) -> LayoutJob {
    SyntaxHighlighter::highlight_toml(code, is_light)
}

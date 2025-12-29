use super::EditorContext;
use crate::model::{Variable, VariableType};
use crate::theme;
use egui::{RichText, Ui};

pub fn render_variables(ui: &mut Ui, ctx: &mut EditorContext) {
    ui.add_space(4.0);
    // Add variable button
    if ui
        .add(egui::Button::new(
            RichText::new("+ Add Variable").color(theme::success_color(ui.ctx())),
        ))
        .clicked()
    {
        let name = format!("var_{}", ctx.project_state.variables.len());
        ctx.project_state.variables.insert(
            name.clone(),
            Variable {
                name,
                v_type: VariableType::String,
                value: "".to_string(),
            },
        );
    }

    ui.add_space(8.0);

    // Variable list
    let mut keys: Vec<String> = ctx.project_state.variables.keys().cloned().collect();
    keys.sort();

    let mut to_remove = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for key in keys {
            theme::section_frame(ui.ctx()).show(ui, |ui| {
                if let Some(var) = ctx.project_state.variables.get_mut(&key) {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&var.name).strong());
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("✕").color(theme::error_color(ui.ctx())),
                                        )
                                        .small(),
                                    )
                                    .clicked()
                                {
                                    to_remove = Some(key.clone());
                                }
                            },
                        );
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Type:").size(11.0).color(theme::muted_color(ui.ctx())));
                        egui::ComboBox::from_id_salt(format!("type_{}", key))
                            .selected_text(format!("{}", var.v_type))
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut var.v_type,
                                    VariableType::String,
                                    "String",
                                );
                                ui.selectable_value(
                                    &mut var.v_type,
                                    VariableType::Integer,
                                    "Integer",
                                );
                                ui.selectable_value(
                                    &mut var.v_type,
                                    VariableType::Boolean,
                                    "Boolean",
                                );
                                ui.selectable_value(
                                    &mut var.v_type,
                                    VariableType::Float,
                                    "Float",
                                );
                            });

                        ui.label(
                            RichText::new("Value:")
                                .size(11.0)
                                .color(theme::muted_color(ui.ctx())),
                        );
                        ui.add(egui::TextEdit::singleline(&mut var.value).desired_width(80.0));
                    });
                }
            });
            ui.add_space(4.0);
        }
    });

    if let Some(key) = to_remove {
        ctx.project_state.variables.remove(&key);
    }
}
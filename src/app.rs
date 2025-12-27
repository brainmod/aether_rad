use crate::model::ProjectState;
use crate::ui::{default_layout, AetherTab, AetherTabViewer};
use crate::widgets::{ButtonWidget, VerticalLayout};
use eframe::App;
use egui_dock::{DockArea, DockState};

pub struct AetherApp {
    // The visual state of the docking area (layout, tabs, sizes)
    dock_state: DockState<AetherTab>,

    // The data state of the user's project (SOM)
    project_state: ProjectState,
}

impl AetherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize with a default project
        let mut root = VerticalLayout::default();
        root.children.push(Box::new(ButtonWidget {
            text: "Hello Aether".into(),
            ..Default::default()
        }));
        root.children.push(Box::new(ButtonWidget {
            text: "Edit Me".into(),
            ..Default::default()
        }));

        Self {
            dock_state: default_layout(),
            project_state: ProjectState::new(Box::new(root)),
        }
    }
}

impl App for AetherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Project").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aether Project", &["json"])
                            .save_file()
                        {
                            if let Ok(file) = std::fs::File::create(path) {
                                let _ = serde_json::to_writer_pretty(file, &self.project_state);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Load Project").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aether Project", &["json"])
                            .pick_file()
                        {
                            if let Ok(file) = std::fs::File::open(path) {
                                let reader = std::io::BufReader::new(file);
                                if let Ok(state) = serde_json::from_reader(reader) {
                                    self.project_state = state;
                                } else {
                                    eprintln!("Failed to parse project file");
                                }
                            }
                        }
                        ui.close();
                    }
                });
            });
        });

        // Create the viewer, passing mutable access to the project data
        let mut viewer = AetherTabViewer {
            project_state: &mut self.project_state,
        };

        // Render the docking area
        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut viewer);
    }
}

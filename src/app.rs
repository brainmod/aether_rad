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

use std::path::PathBuf;

use egui::{Color32, RichText};

use crate::types::ExecutionState;

#[derive(Default)]
pub struct ControlPanelResponse {
    pub load_binary: Option<PathBuf>,
    pub step_in: bool,
    pub step_over: bool,
    pub continue_exec: bool,
    pub refresh_state: bool,
}

pub fn show_control_panel(
    ui: &mut egui::Ui,
    execution_state: ExecutionState,
    loaded_binary: Option<&PathBuf>,
) -> ControlPanelResponse {
    let mut response = ControlPanelResponse::default();

    ui.horizontal_wrapped(|ui| {
        if ui.button("Open Binary").clicked() {
            response.load_binary = rfd::FileDialog::new().pick_file();
        }

        ui.separator();

        let has_target = loaded_binary.is_some();
        if ui
            .add_enabled(has_target, egui::Button::new("Step In"))
            .clicked()
        {
            response.step_in = true;
        }
        if ui
            .add_enabled(has_target, egui::Button::new("Step Over"))
            .clicked()
        {
            response.step_over = true;
        }
        if ui
            .add_enabled(has_target, egui::Button::new("Continue"))
            .clicked()
        {
            response.continue_exec = true;
        }
        if ui
            .add_enabled(has_target, egui::Button::new("Refresh"))
            .clicked()
        {
            response.refresh_state = true;
        }

        ui.separator();
        let (label, color) = state_badge(execution_state);
        ui.label(RichText::new(label).strong().color(color));

        if let Some(path) = loaded_binary {
            ui.monospace(
                RichText::new(path.display().to_string()).color(Color32::from_rgb(175, 193, 208)),
            );
        } else {
            ui.monospace(
                RichText::new("No binary loaded").color(Color32::from_rgb(122, 138, 150)),
            );
        }
    });

    response
}

fn state_badge(state: ExecutionState) -> (&'static str, Color32) {
    match state {
        ExecutionState::NoTarget => ("State: No Target", Color32::from_rgb(170, 170, 170)),
        ExecutionState::Loaded => ("State: Loaded", Color32::from_rgb(117, 196, 237)),
        ExecutionState::Running => ("State: Running", Color32::from_rgb(135, 218, 129)),
        ExecutionState::Paused => ("State: Paused", Color32::from_rgb(243, 209, 121)),
        ExecutionState::Exited => ("State: Exited", Color32::from_rgb(242, 142, 142)),
    }
}

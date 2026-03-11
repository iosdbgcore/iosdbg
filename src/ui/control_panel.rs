use std::path::PathBuf;

use egui::{Color32, RichText};

use crate::types::{AttachRequest, ExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachMode {
    Pid,
    ProcessName,
}

#[derive(Debug, Clone)]
pub struct ControlPanelState {
    pub attach_mode: AttachMode,
    pub pid_input: String,
    pub process_name_input: String,
}

impl Default for ControlPanelState {
    fn default() -> Self {
        Self {
            attach_mode: AttachMode::Pid,
            pid_input: String::new(),
            process_name_input: String::new(),
        }
    }
}

#[derive(Default)]
pub struct ControlPanelResponse {
    pub load_binary: Option<PathBuf>,
    pub attach_request: Option<AttachRequest>,
    pub step_in: bool,
    pub step_over: bool,
    pub continue_exec: bool,
    pub refresh_state: bool,
}

pub fn show_control_panel(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    execution_state: ExecutionState,
    loaded_binary: Option<&PathBuf>,
    attached_target: Option<&str>,
) -> ControlPanelResponse {
    let mut response = ControlPanelResponse::default();

    ui.horizontal_wrapped(|ui| {
        if ui.button("Open Binary").clicked() {
            response.load_binary = rfd::FileDialog::new().pick_file();
        }

        ui.separator();
        ui.label(RichText::new("Attach").strong());

        ui.radio_value(&mut state.attach_mode, AttachMode::Pid, "PID");
        ui.radio_value(&mut state.attach_mode, AttachMode::ProcessName, "Process");

        let attach_request = match state.attach_mode {
            AttachMode::Pid => {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut state.pid_input)
                        .hint_text("pid")
                        .desired_width(90.0),
                );
                if edit.changed() {
                    state.process_name_input.clear();
                }
                derive_attach_request(
                    state.attach_mode,
                    &state.pid_input,
                    &state.process_name_input,
                )
            }
            AttachMode::ProcessName => {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut state.process_name_input)
                        .hint_text("process name")
                        .desired_width(140.0),
                );
                if edit.changed() {
                    state.pid_input.clear();
                }
                derive_attach_request(
                    state.attach_mode,
                    &state.pid_input,
                    &state.process_name_input,
                )
            }
        };

        let attach_label = if attach_request.is_some() {
            "Attach"
        } else {
            "Attach (disabled)"
        };
        if ui
            .add_enabled(attach_request.is_some(), egui::Button::new(attach_label))
            .clicked()
        {
            response.attach_request = attach_request;
        }

        ui.separator();

        let has_target = loaded_binary.is_some() || attached_target.is_some();
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

        if let Some(target) = attached_target {
            ui.monospace(RichText::new(target).color(Color32::from_rgb(200, 210, 220)));
        } else if let Some(path) = loaded_binary {
            ui.monospace(
                RichText::new(path.display().to_string()).color(Color32::from_rgb(200, 210, 220)),
            );
        } else {
            ui.monospace(
                RichText::new("No target attached").color(Color32::from_rgb(200, 200, 200)),
            );
        }
    });

    response
}

fn derive_attach_request(
    mode: AttachMode,
    pid_input: &str,
    process_name_input: &str,
) -> Option<AttachRequest> {
    match mode {
        AttachMode::Pid => pid_input
            .trim()
            .parse::<u32>()
            .ok()
            .map(AttachRequest::by_pid),
        AttachMode::ProcessName => {
            let name = process_name_input.trim();
            if name.is_empty() {
                None
            } else {
                Some(AttachRequest::by_process_name(name.to_string()))
            }
        }
    }
}

fn state_badge(state: ExecutionState) -> (&'static str, Color32) {
    match state {
        ExecutionState::NoTarget => ("State: No Target", Color32::from_rgb(200, 200, 200)),
        ExecutionState::Loaded => ("State: Loaded", Color32::from_rgb(150, 210, 245)),
        ExecutionState::Running => ("State: Running", Color32::from_rgb(180, 230, 180)),
        ExecutionState::Paused => ("State: Paused", Color32::from_rgb(250, 220, 150)),
        ExecutionState::Exited => ("State: Exited", Color32::from_rgb(250, 180, 180)),
    }
}

#[cfg(test)]
mod tests {
    use super::{derive_attach_request, AttachMode};
    use crate::types::AttachTarget;

    #[test]
    fn derives_pid_attach_request_for_numeric_input() {
        let request = derive_attach_request(AttachMode::Pid, "1234", "")
            .expect("numeric pid should produce attach request");
        assert!(matches!(request.target, AttachTarget::Pid(1234)));
    }

    #[test]
    fn rejects_blank_process_name_for_attach_request() {
        let request = derive_attach_request(AttachMode::ProcessName, "", "   ");
        assert!(request.is_none());
    }
}

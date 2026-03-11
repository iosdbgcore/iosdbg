use std::path::PathBuf;

use egui::{Color32, RichText};

use crate::types::{
    AttachRequest, ExecutionState, RemoteConfig, RemoteSessionState, RemoteSessionStatus,
};

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
    pub remote_endpoint: String,
    pub remote_token: String,
    pub remote_timeout_ms: String,
    pub remote_retry_count: String,
}

impl Default for ControlPanelState {
    fn default() -> Self {
        let config = RemoteConfig::default();
        Self {
            attach_mode: AttachMode::Pid,
            pid_input: String::new(),
            process_name_input: String::new(),
            remote_endpoint: config.endpoint,
            remote_token: String::new(),
            remote_timeout_ms: config.timeout_ms.to_string(),
            remote_retry_count: config.retry_count.to_string(),
        }
    }
}

#[derive(Default)]
pub struct ControlPanelResponse {
    pub load_binary: Option<PathBuf>,
    pub attach_request: Option<AttachRequest>,
    pub connect_remote: Option<RemoteConfig>,
    pub disconnect_remote: bool,
    pub step_in: bool,
    pub step_over: bool,
    pub continue_exec: bool,
    pub pause_exec: bool,
    pub read_registers: bool,
    pub read_memory: bool,
    pub refresh_state: bool,
}

pub fn show_control_panel(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    execution_state: ExecutionState,
    loaded_binary: Option<&PathBuf>,
    attached_target: Option<&str>,
    remote_status: &RemoteSessionStatus,
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
        ui.label(RichText::new("Remote").strong());

        ui.add(
            egui::TextEdit::singleline(&mut state.remote_endpoint)
                .hint_text("127.0.0.1:27400")
                .desired_width(150.0),
        );
        ui.add(
            egui::TextEdit::singleline(&mut state.remote_token)
                .hint_text("token(optional)")
                .desired_width(120.0),
        );
        ui.add(
            egui::TextEdit::singleline(&mut state.remote_timeout_ms)
                .hint_text("timeout ms")
                .desired_width(88.0),
        );
        ui.add(
            egui::TextEdit::singleline(&mut state.remote_retry_count)
                .hint_text("retry")
                .desired_width(60.0),
        );

        let connect_config = derive_remote_config(
            &state.remote_endpoint,
            &state.remote_token,
            &state.remote_timeout_ms,
            &state.remote_retry_count,
        );
        let can_connect = connect_config.is_some()
            && remote_status.state != RemoteSessionState::Connected
            && remote_status.state != RemoteSessionState::Connecting;
        if ui
            .add_enabled(can_connect, egui::Button::new("Connect"))
            .clicked()
        {
            response.connect_remote = connect_config;
        }

        if ui
            .add_enabled(
                remote_status.state == RemoteSessionState::Connected
                    || remote_status.state == RemoteSessionState::Degraded,
                egui::Button::new("Disconnect"),
            )
            .clicked()
        {
            response.disconnect_remote = true;
        }

        ui.separator();

        let remote_ready = remote_status.state == RemoteSessionState::Connected;
        let has_target = loaded_binary.is_some() || attached_target.is_some() || remote_ready;
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
            .add_enabled(has_target, egui::Button::new("Pause"))
            .clicked()
        {
            response.pause_exec = true;
        }
        if ui
            .add_enabled(has_target, egui::Button::new("Read Registers"))
            .clicked()
        {
            response.read_registers = true;
        }
        if ui
            .add_enabled(has_target, egui::Button::new("Read Memory"))
            .clicked()
        {
            response.read_memory = true;
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

        if remote_ready {
            ui.monospace(
                RichText::new(format!(
                    "Remote {}",
                    remote_status
                        .endpoint
                        .as_deref()
                        .unwrap_or("unknown-endpoint")
                ))
                .color(Color32::from_rgb(200, 210, 220)),
            );
        } else if let Some(target) = attached_target {
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

fn derive_remote_config(
    endpoint_input: &str,
    token_input: &str,
    timeout_input: &str,
    retry_input: &str,
) -> Option<RemoteConfig> {
    let endpoint = endpoint_input.trim();
    if endpoint.is_empty() {
        return None;
    }

    let timeout_ms = timeout_input.trim().parse::<u64>().ok()?;
    if timeout_ms == 0 {
        return None;
    }

    let retry_count = retry_input.trim().parse::<u8>().ok()?;
    let token = token_input.trim();

    Some(RemoteConfig {
        endpoint: endpoint.to_string(),
        token: if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        },
        timeout_ms,
        retry_count,
    })
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
    use super::{derive_attach_request, derive_remote_config, AttachMode};
    use crate::types::{AttachTarget, RemoteConfig};

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

    #[test]
    fn derives_remote_config_from_valid_inputs() {
        let config = derive_remote_config("127.0.0.1:27400", "", "1200", "2")
            .expect("valid remote config expected");
        assert_eq!(config.endpoint, "127.0.0.1:27400");
        assert_eq!(config.timeout_ms, 1200);
    }

    #[test]
    fn rejects_remote_config_with_invalid_timeout() {
        let config = derive_remote_config("127.0.0.1:27400", "", "0", "2");
        assert_eq!(config, Option::<RemoteConfig>::None);
    }
}

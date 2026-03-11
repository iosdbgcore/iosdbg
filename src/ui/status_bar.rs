use egui::{Color32, RichText};

use crate::core::types::SessionLifecycle;
use crate::types::{AttachErrorKind, RemoteSessionState, RemoteSessionStatus};

pub fn show_status_bar(
    ui: &mut egui::Ui,
    status_message: Option<&str>,
    lifecycle: SessionLifecycle,
    attach_error: Option<AttachErrorKind>,
    attached_target: Option<&str>,
    remote_status: &RemoteSessionStatus,
) {
    ui.horizontal_wrapped(|ui| {
        let (label, color) = lifecycle_badge(lifecycle);
        ui.label(RichText::new(label).strong().color(color));

        ui.separator();
        let (remote_label, remote_color) = remote_badge(remote_status.state);
        ui.label(RichText::new(remote_label).strong().color(remote_color));
        if let Some(endpoint) = remote_status.endpoint.as_deref() {
            ui.monospace(RichText::new(format!("Endpoint: {endpoint}")).color(Color32::from_rgb(
                220, 225, 230,
            )));
        }

        if let Some(target) = attached_target {
            ui.separator();
            ui.monospace(
                RichText::new(format!("Target: {target}")).color(Color32::from_rgb(220, 225, 230)),
            );
        }

        if let Some(kind) = attach_error {
            ui.separator();
            ui.label(
                RichText::new(format!("Attach error: {kind}"))
                    .color(Color32::from_rgb(229, 88, 88)),
            );
        }

        if let Some(kind) = remote_status.error {
            ui.separator();
            ui.label(
                RichText::new(format!("Remote error: {kind}"))
                    .color(Color32::from_rgb(229, 88, 88)),
            );
        }

        if let Some(message) = status_message {
            ui.separator();
            ui.label(RichText::new(message).color(Color32::from_rgb(220, 225, 230)));
        }
    });
}

fn lifecycle_badge(lifecycle: SessionLifecycle) -> (&'static str, Color32) {
    match lifecycle {
        SessionLifecycle::Detached => ("Attach: Detached", Color32::from_rgb(200, 200, 200)),
        SessionLifecycle::Attaching => ("Attach: Attaching", Color32::from_rgb(250, 210, 140)),
        SessionLifecycle::Attached => ("Attach: Attached", Color32::from_rgb(180, 220, 180)),
        SessionLifecycle::Failed => ("Attach: Failed", Color32::from_rgb(240, 140, 140)),
    }
}

fn remote_badge(state: RemoteSessionState) -> (&'static str, Color32) {
    match state {
        RemoteSessionState::Disconnected => ("Remote: Disconnected", Color32::from_rgb(200, 200, 200)),
        RemoteSessionState::Connecting => ("Remote: Connecting", Color32::from_rgb(242, 183, 95)),
        RemoteSessionState::Connected => ("Remote: Connected", Color32::from_rgb(121, 201, 139)),
        RemoteSessionState::Degraded => ("Remote: Degraded", Color32::from_rgb(229, 88, 88)),
    }
}

#[cfg(test)]
mod tests {
    use super::{lifecycle_badge, remote_badge};
    use crate::core::types::SessionLifecycle;
    use crate::types::RemoteSessionState;

    #[test]
    fn lifecycle_badge_labels_match_expected_states() {
        let (detached, _) = lifecycle_badge(SessionLifecycle::Detached);
        let (attached, _) = lifecycle_badge(SessionLifecycle::Attached);
        let (failed, _) = lifecycle_badge(SessionLifecycle::Failed);

        assert_eq!(detached, "Attach: Detached");
        assert_eq!(attached, "Attach: Attached");
        assert_eq!(failed, "Attach: Failed");
    }

    #[test]
    fn remote_badge_labels_match_expected_states() {
        let (disconnected, _) = remote_badge(RemoteSessionState::Disconnected);
        let (connected, _) = remote_badge(RemoteSessionState::Connected);
        let (degraded, _) = remote_badge(RemoteSessionState::Degraded);

        assert_eq!(disconnected, "Remote: Disconnected");
        assert_eq!(connected, "Remote: Connected");
        assert_eq!(degraded, "Remote: Degraded");
    }
}

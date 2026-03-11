use egui::{Color32, RichText};

use crate::core::types::SessionLifecycle;
use crate::types::AttachErrorKind;

pub fn show_status_bar(
    ui: &mut egui::Ui,
    status_message: Option<&str>,
    lifecycle: SessionLifecycle,
    attach_error: Option<AttachErrorKind>,
    attached_target: Option<&str>,
) {
    ui.horizontal_wrapped(|ui| {
        let (label, color) = lifecycle_badge(lifecycle);
        ui.label(RichText::new(label).strong().color(color));

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

#[cfg(test)]
mod tests {
    use super::lifecycle_badge;
    use crate::core::types::SessionLifecycle;

    #[test]
    fn lifecycle_badge_labels_match_expected_states() {
        let (detached, _) = lifecycle_badge(SessionLifecycle::Detached);
        let (attached, _) = lifecycle_badge(SessionLifecycle::Attached);
        let (failed, _) = lifecycle_badge(SessionLifecycle::Failed);

        assert_eq!(detached, "Attach: Detached");
        assert_eq!(attached, "Attach: Attached");
        assert_eq!(failed, "Attach: Failed");
    }
}

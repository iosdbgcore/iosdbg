use std::collections::HashSet;

use egui::{Color32, RichText};

use crate::types::AssemblyInstruction;

pub fn show_assembly_view(
    ui: &mut egui::Ui,
    instructions: &[AssemblyInstruction],
    current_pc: Option<u64>,
    breakpoints: &HashSet<u64>,
) -> Option<u64> {
    let mut clicked_breakpoint = None;

    ui.heading("Assembly");
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for instruction in instructions {
                let is_current = current_pc == Some(instruction.address);
                let has_breakpoint = breakpoints.contains(&instruction.address);

                let frame = egui::Frame::none()
                    .fill(if is_current {
                        Color32::from_rgb(42, 58, 74)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .inner_margin(egui::Margin::symmetric(6.0, 2.0));

                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let marker = if has_breakpoint { "●" } else { "○" };
                        let marker_color = if has_breakpoint {
                            Color32::from_rgb(225, 72, 72)
                        } else {
                            Color32::from_gray(90)
                        };
                        let marker_clicked = ui
                            .add(
                                egui::Button::new(RichText::new(marker).color(marker_color))
                                    .frame(false),
                            )
                            .clicked();
                        if marker_clicked {
                            clicked_breakpoint = Some(instruction.address);
                        }

                        ui.monospace(
                            RichText::new(format!("0x{:016x}", instruction.address))
                                .color(Color32::from_rgb(150, 165, 180)),
                        );
                        ui.monospace(
                            RichText::new(format!("{: <8}", instruction.mnemonic))
                                .color(mnemonic_color(&instruction.mnemonic)),
                        );
                        ui.monospace(
                            RichText::new(&instruction.operands)
                                .color(Color32::from_rgb(200, 212, 226)),
                        );
                    });
                });
            }
        });

    clicked_breakpoint
}

fn mnemonic_color(mnemonic: &str) -> Color32 {
    match mnemonic {
        "bl" | "b.ne" => Color32::from_rgb(239, 178, 67),
        "ret" => Color32::from_rgb(236, 125, 98),
        "cmp" => Color32::from_rgb(121, 201, 176),
        _ => Color32::from_rgb(135, 191, 249),
    }
}

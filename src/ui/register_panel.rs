use egui::{Color32, RichText};

use crate::types::RegisterValue;

pub fn show_register_panel(ui: &mut egui::Ui, registers: &[RegisterValue]) {
    ui.heading("Registers");
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("register-grid")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    for register in registers {
                        ui.monospace(
                            RichText::new(format!("{: >3}", register.name))
                                .color(Color32::from_rgb(147, 195, 255)),
                        );
                        ui.monospace(
                            RichText::new(format!("0x{:016x}", register.value))
                                .color(Color32::from_rgb(210, 218, 227)),
                        );
                        ui.end_row();
                    }
                });
        });
}

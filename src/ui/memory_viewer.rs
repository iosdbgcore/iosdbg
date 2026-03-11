use egui::{Color32, RichText};

pub struct MemoryViewerState {
    pub address_input: String,
    page_size: usize,
}

impl MemoryViewerState {
    pub fn new() -> Self {
        Self {
            address_input: "0x1000".to_string(),
            page_size: 0x100,
        }
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }
}

pub fn show_memory_viewer(
    ui: &mut egui::Ui,
    state: &mut MemoryViewerState,
    memory_address: u64,
    memory_bytes: &[u8],
) -> Option<u64> {
    let mut requested_address = None;

    ui.heading("Memory");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Address");
        let response = ui.text_edit_singleline(&mut state.address_input);
        let enter_pressed =
            response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

        if ui.button("Read").clicked() || enter_pressed {
            if let Some(parsed) = parse_address(&state.address_input) {
                requested_address = Some(parsed);
            }
        }

        if ui.button("← Prev").clicked() {
            let candidate = memory_address.saturating_sub(state.page_size as u64);
            state.address_input = format!("0x{candidate:x}");
            requested_address = Some(candidate);
        }

        if ui.button("Next →").clicked() {
            let candidate = memory_address.saturating_add(state.page_size as u64);
            state.address_input = format!("0x{candidate:x}");
            requested_address = Some(candidate);
        }
    });

    ui.label(
        RichText::new(format!("Current region: 0x{memory_address:016x}"))
            .color(Color32::from_rgb(157, 174, 191)),
    );

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (line_index, chunk) in memory_bytes.chunks(16).enumerate() {
            let line_address = memory_address + ((line_index * 16) as u64);
            let hex = chunk
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join(" ");
            let ascii = chunk
                .iter()
                .map(|byte| {
                    let ch = *byte as char;
                    if ch.is_ascii_graphic() {
                        ch
                    } else {
                        '.'
                    }
                })
                .collect::<String>();

            ui.monospace(
                RichText::new(format!("0x{line_address:016x}  {hex:<47}  |{ascii}|"))
                    .color(Color32::from_rgb(204, 216, 228)),
            );
        }
    });

    requested_address
}

fn parse_address(text: &str) -> Option<u64> {
    let normalized = text.trim();
    if let Some(hex) = normalized
        .strip_prefix("0x")
        .or_else(|| normalized.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).ok();
    }
    normalized.parse::<u64>().ok()
}

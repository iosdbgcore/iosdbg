use egui::Color32;

#[derive(Debug, Clone, Copy)]
pub struct UiPalette {
    pub panel_bg: Color32,
    pub panel_border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent_blue: Color32,
    pub accent_red: Color32,
    pub accent_green: Color32,
    pub accent_orange: Color32,
}

impl UiPalette {
    pub fn x64dbg_parity() -> Self {
        Self {
            panel_bg: Color32::from_rgb(26, 30, 36),
            panel_border: Color32::from_rgb(54, 62, 70),
            text_primary: Color32::from_rgb(214, 221, 230),
            text_muted: Color32::from_rgb(141, 153, 167),
            accent_blue: Color32::from_rgb(121, 186, 255),
            accent_red: Color32::from_rgb(229, 88, 88),
            accent_green: Color32::from_rgb(121, 201, 139),
            accent_orange: Color32::from_rgb(242, 183, 95),
        }
    }
}

pub fn apply_x64dbg_theme(ctx: &egui::Context) {
    let palette = UiPalette::x64dbg_parity();
    let mut style = (*ctx.style()).clone();
    style.visuals.panel_fill = palette.panel_bg;
    style.visuals.window_fill = palette.panel_bg;
    style.visuals.widgets.noninteractive.bg_fill = palette.panel_bg;
    style.visuals.widgets.noninteractive.bg_stroke.color = palette.panel_border;
    style.visuals.widgets.noninteractive.fg_stroke.color = palette.text_primary;
    style.visuals.widgets.inactive.fg_stroke.color = palette.text_primary;
    style.visuals.widgets.active.fg_stroke.color = palette.accent_blue;
    style.visuals.widgets.hovered.fg_stroke.color = palette.accent_orange;
    style.visuals.extreme_bg_color = Color32::from_rgb(19, 22, 28);
    style.visuals.selection.bg_fill = palette.accent_blue.gamma_multiply(0.25);
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::UiPalette;

    #[test]
    fn x64dbg_palette_has_distinct_status_colors() {
        let palette = UiPalette::x64dbg_parity();
        assert_ne!(palette.accent_red, palette.accent_green);
        assert_ne!(palette.accent_blue, palette.accent_orange);
    }
}

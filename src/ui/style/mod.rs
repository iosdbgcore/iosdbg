use egui::Color32;

#[derive(Debug, Clone, Copy)]
pub struct UiPalette {
    pub panel_bg: Color32,
    pub panel_border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub button_bg_normal: Color32,
    pub button_bg_hover: Color32,
    pub button_bg_active: Color32,
    pub button_bg_disabled: Color32,
    pub button_fg_normal: Color32,
    pub button_fg_hover: Color32,
    pub button_fg_active: Color32,
    pub button_fg_disabled: Color32,
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
            button_bg_normal: Color32::from_rgb(49, 57, 67),
            button_bg_hover: Color32::from_rgb(58, 67, 78),
            button_bg_active: Color32::from_rgb(67, 78, 90),
            button_bg_disabled: Color32::from_rgb(37, 43, 50),
            button_fg_normal: Color32::from_rgb(226, 232, 239),
            button_fg_hover: Color32::from_rgb(252, 208, 133),
            button_fg_active: Color32::from_rgb(172, 214, 255),
            button_fg_disabled: Color32::from_rgb(151, 162, 176),
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
    style.visuals.widgets.inactive.bg_fill = palette.button_bg_normal;
    style.visuals.widgets.inactive.fg_stroke.color = palette.button_fg_normal;
    style.visuals.widgets.hovered.bg_fill = palette.button_bg_hover;
    style.visuals.widgets.hovered.fg_stroke.color = palette.button_fg_hover;
    style.visuals.widgets.active.bg_fill = palette.button_bg_active;
    style.visuals.widgets.active.fg_stroke.color = palette.button_fg_active;
    style.visuals.widgets.open.bg_fill = palette.button_bg_active;
    style.visuals.widgets.open.fg_stroke.color = palette.button_fg_active;
    // Disabled buttons use the noninteractive visual set in egui.
    style.visuals.widgets.noninteractive.bg_fill = palette.button_bg_disabled;
    style.visuals.widgets.noninteractive.fg_stroke.color = palette.button_fg_disabled;
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

    #[test]
    fn button_foreground_never_matches_button_background() {
        let palette = UiPalette::x64dbg_parity();
        assert_ne!(palette.button_bg_normal, palette.button_fg_normal);
        assert_ne!(palette.button_bg_hover, palette.button_fg_hover);
        assert_ne!(palette.button_bg_active, palette.button_fg_active);
        assert_ne!(palette.button_bg_disabled, palette.button_fg_disabled);
    }
}

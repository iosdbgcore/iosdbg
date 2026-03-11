pub const PANEL_ASSEMBLY: &str = "panel-assembly";
pub const PANEL_REGISTERS: &str = "panel-registers";
pub const PANEL_MEMORY: &str = "panel-memory";
pub const PANEL_CONTROL: &str = "panel-control";

#[derive(Debug, Clone, Copy)]
pub struct DockLayout {
    pub left_width_ratio: f32,
    pub right_top_ratio: f32,
}

impl DockLayout {
    pub fn x64dbg_default() -> Self {
        Self {
            left_width_ratio: 0.62,
            right_top_ratio: 0.43,
        }
    }
}

pub trait UiParityAdapter {
    fn dock_layout(&self) -> DockLayout;
    fn theme_name(&self) -> &'static str;
}

pub struct EguiParityAdapter;

impl UiParityAdapter for EguiParityAdapter {
    fn dock_layout(&self) -> DockLayout {
        DockLayout::x64dbg_default()
    }

    fn theme_name(&self) -> &'static str {
        "x64dbg-parity-egui"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DockLayout, EguiParityAdapter, UiParityAdapter, PANEL_ASSEMBLY, PANEL_CONTROL,
        PANEL_MEMORY, PANEL_REGISTERS,
    };

    #[test]
    fn panel_ids_are_stable_for_ui_acceptance_checks() {
        assert_eq!(PANEL_CONTROL, "panel-control");
        assert_eq!(PANEL_ASSEMBLY, "panel-assembly");
        assert_eq!(PANEL_REGISTERS, "panel-registers");
        assert_eq!(PANEL_MEMORY, "panel-memory");
    }

    #[test]
    fn default_layout_prioritizes_assembly_panel() {
        let layout = DockLayout::x64dbg_default();
        assert!(layout.left_width_ratio > 0.5);
    }

    #[test]
    fn adapter_exposes_stable_theme_contract() {
        let adapter = EguiParityAdapter;
        assert_eq!(adapter.theme_name(), "x64dbg-parity-egui");
        assert!(adapter.dock_layout().left_width_ratio > 0.5);
    }
}

use rust_lldb_visual_debugger::app::DebuggerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust LLDB Visual Debugger",
        native_options,
        Box::new(|cc| Ok(Box::new(DebuggerApp::new(cc)))),
    )
}

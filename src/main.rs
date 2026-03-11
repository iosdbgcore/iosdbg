mod app;
mod core;
mod types;
mod ui;

use app::DebuggerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust LLDB Visual Debugger",
        native_options,
        Box::new(|cc| Ok(Box::new(DebuggerApp::new(cc)))),
    )
}

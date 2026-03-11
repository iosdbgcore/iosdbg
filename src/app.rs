use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use eframe::egui;

use crate::core::events::DebugEvent;
use crate::core::{spawn_debugger_core, CoreChannels};
use crate::types::{AssemblyInstruction, DebugCommand, ExecutionState, RegisterValue};
use crate::ui::assembly_view::show_assembly_view;
use crate::ui::control_panel::show_control_panel;
use crate::ui::memory_viewer::{show_memory_viewer, MemoryViewerState};
use crate::ui::register_panel::show_register_panel;

pub struct DebuggerApp {
    command_tx: Sender<DebugCommand>,
    event_rx: Receiver<DebugEvent>,
    loaded_binary: Option<PathBuf>,
    execution_state: ExecutionState,
    instructions: Vec<AssemblyInstruction>,
    breakpoints: HashSet<u64>,
    current_pc: Option<u64>,
    registers: Vec<RegisterValue>,
    memory_address: u64,
    memory_bytes: Vec<u8>,
    memory_viewer_state: MemoryViewerState,
    status_message: Option<String>,
}

impl DebuggerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let CoreChannels {
            command_tx,
            event_rx,
        } = spawn_debugger_core().expect("failed to initialize debugger core");

        Self {
            command_tx,
            event_rx,
            loaded_binary: None,
            execution_state: ExecutionState::NoTarget,
            instructions: Vec::new(),
            breakpoints: HashSet::new(),
            current_pc: None,
            registers: Vec::new(),
            memory_address: 0x1000,
            memory_bytes: vec![0; 0x100],
            memory_viewer_state: MemoryViewerState::new(),
            status_message: Some("Ready. Load a processed binary to start debugging.".to_string()),
        }
    }

    fn send_command(&self, command: DebugCommand) {
        let _ = self.command_tx.send(command);
    }

    fn poll_events(&mut self) {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => self.apply_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.status_message = Some("Debugger core disconnected".to_string());
                    break;
                }
            }
        }
    }

    fn apply_event(&mut self, event: DebugEvent) {
        match event {
            DebugEvent::TargetLoaded(path) => {
                self.loaded_binary = Some(path.clone());
                self.status_message = Some(format!("Loaded target: {}", path.display()));
            }
            DebugEvent::AssemblyUpdated(instructions) => {
                self.instructions = instructions;
            }
            DebugEvent::BreakpointsChanged(breakpoints) => {
                self.breakpoints = breakpoints.into_iter().collect();
            }
            DebugEvent::InstructionPointerChanged(pc) => {
                self.current_pc = pc;
            }
            DebugEvent::RegistersUpdated(registers) => {
                self.registers = registers;
            }
            DebugEvent::MemoryUpdated { address, bytes } => {
                self.memory_address = address;
                self.memory_bytes = bytes;
            }
            DebugEvent::StateChanged(state) => {
                self.execution_state = state;
            }
            DebugEvent::BreakpointHit(address) => {
                self.status_message = Some(format!("Breakpoint hit at 0x{address:016x}"));
            }
            DebugEvent::Error(message) => {
                self.status_message = Some(format!("Error: {message}"));
            }
        }
    }
}

impl eframe::App for DebuggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events();

        egui::TopBottomPanel::top("control-panel").show(ctx, |ui| {
            let controls = show_control_panel(ui, self.execution_state, self.loaded_binary.as_ref());

            if let Some(path) = controls.load_binary {
                self.send_command(DebugCommand::LoadBinary(path));
            }
            if controls.step_in {
                self.send_command(DebugCommand::StepIn);
            }
            if controls.step_over {
                self.send_command(DebugCommand::StepOver);
            }
            if controls.continue_exec {
                self.send_command(DebugCommand::Continue);
            }
            if controls.refresh_state {
                self.send_command(DebugCommand::RefreshState);
            }
        });

        egui::TopBottomPanel::bottom("status-bar")
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(message) = &self.status_message {
                    ui.label(message);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let left_width = available_width * 0.6;
            let right_width = (available_width - left_width - 8.0).max(100.0);

            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(left_width, available_height), |ui| {
                    if let Some(address) = show_assembly_view(
                        ui,
                        &self.instructions,
                        self.current_pc,
                        &self.breakpoints,
                    ) {
                        self.send_command(DebugCommand::ToggleBreakpoint(address));
                    }
                });

                ui.separator();

                ui.allocate_ui(egui::vec2(right_width, available_height), |ui| {
                    let register_height = available_height * 0.42;
                    ui.allocate_ui(egui::vec2(right_width, register_height), |ui| {
                        show_register_panel(ui, &self.registers);
                    });
                    ui.separator();
                    ui.allocate_ui(egui::vec2(right_width, available_height - register_height), |ui| {
                        if let Some(address) = show_memory_viewer(
                            ui,
                            &mut self.memory_viewer_state,
                            self.memory_address,
                            &self.memory_bytes,
                        ) {
                            self.send_command(DebugCommand::ReadMemory {
                                address,
                                size: self.memory_viewer_state.page_size(),
                            });
                        }
                    });
                });
            });
        });
    }
}

impl Drop for DebuggerApp {
    fn drop(&mut self) {
        let _ = self.command_tx.send(DebugCommand::Stop);
    }
}

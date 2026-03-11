pub mod engine;
pub mod events;
pub mod session;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use engine::{DebugEngine, MockLldbEngine};
use events::DebugEvent;

use crate::types::{CoreError, CoreResult, DebugCommand};

pub struct CoreChannels {
    pub command_tx: Sender<DebugCommand>,
    pub event_rx: Receiver<DebugEvent>,
}

struct DebuggerCore {
    engine: Box<dyn DebugEngine>,
    event_tx: Sender<DebugEvent>,
}

impl DebuggerCore {
    fn new(engine: Box<dyn DebugEngine>, event_tx: Sender<DebugEvent>) -> Self {
        Self { engine, event_tx }
    }

    fn send(&self, event: DebugEvent) {
        let _ = self.event_tx.send(event);
    }

    fn emit_snapshot(&mut self) {
        let snapshot = self.engine.snapshot();

        self.send(DebugEvent::StateChanged(snapshot.state));
        self.send(DebugEvent::InstructionPointerChanged(snapshot.current_pc));
        self.send(DebugEvent::AssemblyUpdated(snapshot.instructions));

        let mut breakpoints = snapshot.breakpoints.into_iter().collect::<Vec<_>>();
        breakpoints.sort_unstable();
        self.send(DebugEvent::BreakpointsChanged(breakpoints));

        self.send(DebugEvent::RegistersUpdated(snapshot.registers));
        self.send(DebugEvent::MemoryUpdated {
            address: snapshot.memory.address,
            bytes: snapshot.memory.bytes,
        });
    }

    fn process_step_result(&mut self, step_result: CoreResult<Option<u64>>, emit_breakpoint_hit: bool) {
        match step_result {
            Ok(hit_address) => {
                if emit_breakpoint_hit {
                    if let Some(address) = hit_address {
                        self.send(DebugEvent::BreakpointHit(address));
                    }
                }
                self.emit_snapshot();
            }
            Err(error) => self.send(DebugEvent::Error(error.message)),
        }
    }

    fn process_continue_result(&mut self, continue_result: CoreResult<Option<u64>>) {
        match continue_result {
            Ok(hit_address) => {
                if let Some(address) = hit_address {
                    self.send(DebugEvent::BreakpointHit(address));
                }
                self.emit_snapshot();
            }
            Err(error) => self.send(DebugEvent::Error(error.message)),
        }
    }

    fn handle_command(&mut self, command: DebugCommand) -> bool {
        match command {
            DebugCommand::LoadBinary(path) => {
                match self.engine.load_binary(path.clone()) {
                    Ok(()) => {
                        self.send(DebugEvent::TargetLoaded(path));
                        self.emit_snapshot();
                    }
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
                true
            }
            DebugCommand::ToggleBreakpoint(address) => {
                match self.engine.toggle_breakpoint(address) {
                    Ok(()) => self.emit_snapshot(),
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
                true
            }
            DebugCommand::StepIn => {
                let result = self.engine.step_in();
                self.process_step_result(result, false);
                true
            }
            DebugCommand::StepOver => {
                let result = self.engine.step_over();
                self.process_step_result(result, false);
                true
            }
            DebugCommand::Continue => {
                let result = self.engine.continue_exec();
                self.process_continue_result(result);
                true
            }
            DebugCommand::RefreshState => {
                self.emit_snapshot();
                true
            }
            DebugCommand::ReadMemory { address, size } => {
                match self.engine.read_memory(address, size) {
                    Ok(memory) => self.send(DebugEvent::MemoryUpdated {
                        address: memory.address,
                        bytes: memory.bytes,
                    }),
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
                true
            }
            DebugCommand::Stop => false,
        }
    }

    fn run_loop(&mut self, command_rx: Receiver<DebugCommand>) {
        for command in command_rx {
            if !self.handle_command(command) {
                break;
            }
        }
    }
}

pub fn spawn_debugger_core() -> CoreResult<CoreChannels> {
    let (command_tx, command_rx) = mpsc::channel::<DebugCommand>();
    let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();

    let mut engine = MockLldbEngine::new()?;

    // Prime the UI with a clean default state.
    let snapshot = engine.snapshot();
    event_tx
        .send(DebugEvent::StateChanged(snapshot.state))
        .map_err(|error| CoreError::new(format!("Failed to send initial state: {error}")))?;

    thread::spawn(move || {
        let mut core = DebuggerCore::new(Box::new(engine), event_tx);
        core.run_loop(command_rx);
    });

    Ok(CoreChannels {
        command_tx,
        event_rx,
    })
}

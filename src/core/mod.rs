pub mod engine;
pub mod events;
pub mod remote;
pub mod session;
pub mod types;

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
        self.send(DebugEvent::AttachLifecycleChanged(
            self.engine.attach_lifecycle(),
        ));
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
        self.send(DebugEvent::RemoteSessionChanged(snapshot.remote));
    }

    fn process_step_result(
        &mut self,
        step_result: CoreResult<Option<u64>>,
        emit_breakpoint_hit: bool,
    ) {
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
            DebugCommand::AttachProcess(request) => {
                self.send(DebugEvent::AttachLifecycleChanged(
                    types::SessionLifecycle::Attaching,
                ));
                match self.engine.attach_process(request) {
                    Ok(result) => {
                        self.send(DebugEvent::AttachLifecycleChanged(
                            self.engine.attach_lifecycle(),
                        ));
                        self.send(DebugEvent::AttachUpdated(result.clone()));
                        if result.attached {
                            self.emit_snapshot();
                        } else {
                            // Keep existing snapshot untouched on failed attach; only surface diagnostics.
                            self.send(DebugEvent::Error(result.message));
                        }
                    }
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
                true
            }
            DebugCommand::ConnectRemote(config) => {
                match self.engine.connect_remote(config) {
                    Ok(status) => {
                        self.send(DebugEvent::RemoteSessionChanged(status.clone()));
                        if status.state == crate::types::RemoteSessionState::Connected {
                            self.send(DebugEvent::AttachLifecycleChanged(
                                self.engine.attach_lifecycle(),
                            ));
                            self.emit_snapshot();
                        } else {
                            self.send(DebugEvent::Error(status.message));
                        }
                    }
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
                true
            }
            DebugCommand::DisconnectRemote => {
                self.engine.disconnect_remote();
                self.send(DebugEvent::RemoteSessionChanged(self.engine.remote_session_status()));
                self.emit_snapshot();
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
            DebugCommand::Pause => {
                let result = self.engine.pause_exec();
                self.process_step_result(result, false);
                true
            }
            DebugCommand::ReadRegisters => {
                match self.engine.read_registers() {
                    Ok(registers) => self.send(DebugEvent::RegistersUpdated(registers)),
                    Err(error) => self.send(DebugEvent::Error(error.message)),
                }
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

    let engine = MockLldbEngine::new()?;

    // Prime the UI with a clean default state.
    let snapshot = engine.snapshot();
    event_tx
        .send(DebugEvent::StateChanged(snapshot.state))
        .map_err(|error| CoreError::new(format!("Failed to send initial state: {error}")))?;
    event_tx
        .send(DebugEvent::AttachLifecycleChanged(
            engine.attach_lifecycle(),
        ))
        .map_err(|error| CoreError::new(format!("Failed to send initial attach state: {error}")))?;
    event_tx
        .send(DebugEvent::RemoteSessionChanged(engine.remote_session_status()))
        .map_err(|error| CoreError::new(format!("Failed to send initial remote state: {error}")))?;

    thread::spawn(move || {
        let mut core = DebuggerCore::new(Box::new(engine), event_tx);
        core.run_loop(command_rx);
    });

    Ok(CoreChannels {
        command_tx,
        event_rx,
    })
}

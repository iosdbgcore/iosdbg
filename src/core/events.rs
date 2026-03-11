use std::path::PathBuf;

use crate::core::types::SessionLifecycle;
use crate::types::{
    AssemblyInstruction, AttachResult, ExecutionState, RegisterValue, RemoteCommandResult,
    RemoteSessionStatus,
};

#[derive(Debug, Clone)]
pub enum DebugEvent {
    TargetLoaded(PathBuf),
    AttachUpdated(AttachResult),
    AttachLifecycleChanged(SessionLifecycle),
    AssemblyUpdated(Vec<AssemblyInstruction>),
    BreakpointsChanged(Vec<u64>),
    InstructionPointerChanged(Option<u64>),
    RegistersUpdated(Vec<RegisterValue>),
    MemoryUpdated { address: u64, bytes: Vec<u8> },
    RemoteSessionChanged(RemoteSessionStatus),
    RemoteCommandDispatched(RemoteCommandResult),
    StateChanged(ExecutionState),
    BreakpointHit(u64),
    Error(String),
}

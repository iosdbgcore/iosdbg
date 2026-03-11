use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    NoTarget,
    Loaded,
    Running,
    Paused,
    Exited,
}

impl fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ExecutionState::NoTarget => "No Target",
            ExecutionState::Loaded => "Loaded",
            ExecutionState::Running => "Running",
            ExecutionState::Paused => "Paused",
            ExecutionState::Exited => "Exited",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyInstruction {
    pub address: u64,
    pub mnemonic: String,
    pub operands: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterValue {
    pub name: String,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySnapshot {
    pub address: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DebuggerSnapshot {
    pub binary_path: Option<PathBuf>,
    pub state: ExecutionState,
    pub current_pc: Option<u64>,
    pub instructions: Vec<AssemblyInstruction>,
    pub breakpoints: HashSet<u64>,
    pub registers: Vec<RegisterValue>,
    pub memory: MemorySnapshot,
}

#[derive(Debug, Clone)]
pub enum DebugCommand {
    LoadBinary(PathBuf),
    StepIn,
    StepOver,
    Continue,
    ToggleBreakpoint(u64),
    RefreshState,
    ReadMemory { address: u64, size: usize },
    Stop,
}

#[derive(Debug, Clone)]
pub struct CoreError {
    pub message: String,
}

impl CoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;

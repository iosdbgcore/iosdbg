use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachTarget {
    Pid(u32),
    ProcessName(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachRequest {
    pub target: AttachTarget,
}

impl AttachRequest {
    pub fn by_pid(pid: u32) -> Self {
        Self {
            target: AttachTarget::Pid(pid),
        }
    }

    pub fn by_process_name(name: impl Into<String>) -> Self {
        Self {
            target: AttachTarget::ProcessName(name.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachErrorKind {
    PermissionDenied,
    TargetNotFound,
    Timeout,
    LldbError,
}

impl fmt::Display for AttachErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            AttachErrorKind::PermissionDenied => "permission_denied",
            AttachErrorKind::TargetNotFound => "target_not_found",
            AttachErrorKind::Timeout => "timeout",
            AttachErrorKind::LldbError => "lldb_error",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteSessionState {
    Disconnected,
    Connecting,
    Connected,
    Degraded,
}

impl fmt::Display for RemoteSessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            RemoteSessionState::Disconnected => "disconnected",
            RemoteSessionState::Connecting => "connecting",
            RemoteSessionState::Connected => "connected",
            RemoteSessionState::Degraded => "degraded",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteErrorKind {
    ConnectionFailed,
    AuthFailed,
    Timeout,
    ProtocolError,
}

impl fmt::Display for RemoteErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            RemoteErrorKind::ConnectionFailed => "connection_failed",
            RemoteErrorKind::AuthFailed => "auth_failed",
            RemoteErrorKind::Timeout => "timeout",
            RemoteErrorKind::ProtocolError => "protocol_error",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteConfig {
    pub endpoint: String,
    pub token: Option<String>,
    pub timeout_ms: u64,
    pub retry_count: u8,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:27400".to_string(),
            token: None,
            timeout_ms: 1200,
            retry_count: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSessionStatus {
    pub state: RemoteSessionState,
    pub endpoint: Option<String>,
    pub session_id: Option<String>,
    pub error: Option<RemoteErrorKind>,
    pub message: String,
}

impl RemoteSessionStatus {
    pub fn disconnected() -> Self {
        Self {
            state: RemoteSessionState::Disconnected,
            endpoint: None,
            session_id: None,
            error: None,
            message: "Remote disconnected".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteCommand {
    Continue,
    StepOver,
    StepIn,
    Pause,
    ReadRegisters,
    ReadMemory { address: u64, size: usize },
}

impl RemoteCommand {
    pub fn method_name(&self) -> &'static str {
        match self {
            RemoteCommand::Continue => "debug.continue",
            RemoteCommand::StepOver => "debug.step_over",
            RemoteCommand::StepIn => "debug.step_in",
            RemoteCommand::Pause => "debug.pause",
            RemoteCommand::ReadRegisters => "debug.read_registers",
            RemoteCommand::ReadMemory { .. } => "debug.read_memory",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommandResult {
    pub command: RemoteCommand,
    pub success: bool,
    pub error: Option<RemoteErrorKind>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachResult {
    pub attached: bool,
    pub target_label: String,
    pub error: Option<AttachErrorKind>,
    pub message: String,
}

impl AttachResult {
    pub fn success(target_label: impl Into<String>) -> Self {
        Self {
            attached: true,
            target_label: target_label.into(),
            error: None,
            message: "Attached successfully".to_string(),
        }
    }

    pub fn failure(
        target_label: impl Into<String>,
        error: AttachErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            attached: false,
            target_label: target_label.into(),
            error: Some(error),
            message: message.into(),
        }
    }
}

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
    pub attached_target: Option<String>,
    pub state: ExecutionState,
    pub current_pc: Option<u64>,
    pub instructions: Vec<AssemblyInstruction>,
    pub breakpoints: HashSet<u64>,
    pub registers: Vec<RegisterValue>,
    pub memory: MemorySnapshot,
    pub remote: RemoteSessionStatus,
}

#[derive(Debug, Clone)]
pub enum DebugCommand {
    LoadBinary(PathBuf),
    AttachProcess(AttachRequest),
    ConnectRemote(RemoteConfig),
    DisconnectRemote,
    StepIn,
    StepOver,
    Continue,
    Pause,
    ReadRegisters,
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

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::core::remote::xdbg_adapter::XdbgRemoteAdapter;
use crate::core::session::{LldbSession, RemoteSessionManager};
use crate::core::types::{
    classify_remote_error, describe_attach_target, validate_attach_request, validate_remote_config,
    SessionLifecycle,
};
use crate::types::{
    AssemblyInstruction, AttachErrorKind, AttachRequest, AttachResult, CoreError, CoreResult,
    DebuggerSnapshot, ExecutionState, MemorySnapshot, RegisterValue, RemoteCommand, RemoteConfig,
    RemoteSessionStatus,
};

pub trait DebugEngine: Send {
    fn load_binary(&mut self, binary_path: PathBuf) -> CoreResult<()>;
    fn attach_process(&mut self, request: AttachRequest) -> CoreResult<AttachResult>;
    fn connect_remote(&mut self, config: RemoteConfig) -> CoreResult<RemoteSessionStatus>;
    fn disconnect_remote(&mut self);
    fn remote_session_status(&self) -> RemoteSessionStatus;
    fn attach_lifecycle(&self) -> SessionLifecycle;
    fn step_in(&mut self) -> CoreResult<Option<u64>>;
    fn step_over(&mut self) -> CoreResult<Option<u64>>;
    fn continue_exec(&mut self) -> CoreResult<Option<u64>>;
    fn pause_exec(&mut self) -> CoreResult<Option<u64>>;
    fn toggle_breakpoint(&mut self, address: u64) -> CoreResult<()>;
    fn read_registers(&self) -> CoreResult<Vec<RegisterValue>>;
    fn read_memory(&mut self, address: u64, size: usize) -> CoreResult<MemorySnapshot>;
    fn fetch_disassembly(&self) -> CoreResult<Vec<AssemblyInstruction>>;
    fn snapshot(&self) -> DebuggerSnapshot;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EngineBackend {
    Local,
    Remote,
}

pub struct MockLldbEngine {
    session: LldbSession,
    remote_session: RemoteSessionManager,
    remote_adapter: XdbgRemoteAdapter,
    remote_config: Option<RemoteConfig>,
    backend: EngineBackend,
    binary_path: Option<PathBuf>,
    binary_bytes: Vec<u8>,
    instructions: Vec<AssemblyInstruction>,
    breakpoints: HashSet<u64>,
    pc_index: usize,
    state: ExecutionState,
    memory: MemorySnapshot,
    attached_target_label: Option<String>,
}

impl MockLldbEngine {
    pub fn new() -> CoreResult<Self> {
        Ok(Self {
            session: LldbSession::initialize()?,
            remote_session: RemoteSessionManager::new(),
            remote_adapter: XdbgRemoteAdapter::new(),
            remote_config: None,
            backend: EngineBackend::Local,
            binary_path: None,
            binary_bytes: Vec::new(),
            instructions: Vec::new(),
            breakpoints: HashSet::new(),
            pc_index: 0,
            state: ExecutionState::NoTarget,
            memory: MemorySnapshot {
                address: 0x1000,
                bytes: vec![0; 0x100],
            },
            attached_target_label: None,
        })
    }

    fn current_pc(&self) -> Option<u64> {
        self.instructions.get(self.pc_index).map(|ins| ins.address)
    }

    fn base_address() -> u64 {
        0x1000
    }

    fn decode_disassembly(binary_bytes: &[u8]) -> Vec<AssemblyInstruction> {
        let mnemonics = ["stp", "ldr", "str", "mov", "bl", "cmp", "b.ne", "ret"];

        binary_bytes
            .chunks(4)
            .enumerate()
            .map(|(idx, chunk)| {
                let seed = chunk.iter().fold(0usize, |acc, b| acc + (*b as usize));
                let mnemonic = mnemonics[seed % mnemonics.len()].to_string();
                let register = seed % 29;
                let immediate = (seed % 0x200) as u64;
                let operands = format!("x{register}, #0x{immediate:x}");

                AssemblyInstruction {
                    address: Self::base_address() + ((idx as u64) * 4),
                    mnemonic,
                    operands,
                }
            })
            .collect()
    }

    fn ensure_target(&self) -> CoreResult<()> {
        if self.binary_path.is_none() && self.attached_target_label.is_none() {
            return Err(CoreError::new("No binary loaded"));
        }
        Ok(())
    }

    fn refresh_state_window(&mut self) {
        let address = self.current_pc().unwrap_or(Self::base_address());
        self.memory = self.read_memory(address, 0x100).unwrap_or(MemorySnapshot {
            address,
            bytes: vec![],
        });
    }

    fn register_names() -> Vec<String> {
        let mut names = (0..=28).map(|idx| format!("x{idx}")).collect::<Vec<_>>();
        names.extend(["fp", "lr", "sp", "pc"].iter().map(|name| name.to_string()));
        names
    }

    fn computed_registers(&self) -> Vec<RegisterValue> {
        let pc = self.current_pc().unwrap_or(Self::base_address());

        Self::register_names()
            .into_iter()
            .enumerate()
            .map(|(idx, name)| {
                let value = if name == "pc" {
                    pc
                } else {
                    pc.wrapping_add((idx as u64) * 0x10)
                };
                RegisterValue { name, value }
            })
            .collect()
    }

    fn advance_one_instruction(&mut self) -> Option<u64> {
        if self.instructions.is_empty() {
            self.state = ExecutionState::Exited;
            return None;
        }

        if self.pc_index + 1 >= self.instructions.len() {
            self.state = ExecutionState::Exited;
            return None;
        }

        self.pc_index += 1;
        self.state = ExecutionState::Paused;
        self.current_pc()
    }

    fn disassembly_seed_from_target(target_label: &str) -> Vec<u8> {
        let mut bytes = target_label.as_bytes().to_vec();
        if bytes.is_empty() {
            bytes = vec![0x90; 64];
        }
        while bytes.len() < 64 {
            bytes.extend_from_slice(target_label.as_bytes());
            if target_label.is_empty() {
                bytes.push(0x90);
            }
        }
        bytes.truncate(64);
        bytes
    }

    fn process_exists_by_pid(pid: u32) -> bool {
        if pid == std::process::id() {
            return true;
        }

        Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "pid="])
            .output()
            .map(|output| {
                output.status.success()
                    && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
            })
            .unwrap_or(false)
    }

    fn process_exists_by_name(name: &str) -> bool {
        let needle = name.trim().to_ascii_lowercase();
        if needle.is_empty() {
            return false;
        }

        Command::new("ps")
            .args(["-A", "-o", "comm="])
            .output()
            .map(|output| {
                if !output.status.success() {
                    return false;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .any(|line| line.trim().to_ascii_lowercase().contains(&needle))
            })
            .unwrap_or(false)
    }

    fn classify_attach_error(error_message: &str) -> AttachErrorKind {
        let normalized = error_message.to_ascii_lowercase();
        if normalized.contains("permission") || normalized.contains("task_for_pid") {
            AttachErrorKind::PermissionDenied
        } else if normalized.contains("timeout") {
            AttachErrorKind::Timeout
        } else if normalized.contains("not found") {
            AttachErrorKind::TargetNotFound
        } else {
            AttachErrorKind::LldbError
        }
    }

    fn current_remote_config(&self) -> CoreResult<&RemoteConfig> {
        self.remote_config
            .as_ref()
            .ok_or_else(|| CoreError::new("Remote config is unavailable"))
    }

    fn dispatch_remote_command(&mut self, command: RemoteCommand) -> CoreResult<()> {
        let config = self.current_remote_config()?.clone();
        match self.remote_adapter.dispatch(command) {
            Ok(result) => {
                self.remote_session.mark_connected(
                    &config,
                    self.remote_adapter
                        .session_id()
                        .unwrap_or("xdbg-session")
                        .to_string(),
                );
                self.state = if matches!(command, RemoteCommand::Continue) {
                    ExecutionState::Running
                } else {
                    ExecutionState::Paused
                };
                if !result.success {
                    return Err(CoreError::new(result.message));
                }
                Ok(())
            }
            Err(error) => {
                self.remote_session
                    .mark_failed(&config, error.kind, error.message.clone());
                self.backend = EngineBackend::Local;
                Err(CoreError::new(error.message))
            }
        }
    }
}

impl DebugEngine for MockLldbEngine {
    fn load_binary(&mut self, binary_path: PathBuf) -> CoreResult<()> {
        if !Path::new(&binary_path).exists() {
            return Err(CoreError::new(format!(
                "Binary does not exist: {}",
                binary_path.display()
            )));
        }

        let bytes = fs::read(&binary_path)
            .map_err(|error| CoreError::new(format!("Failed to read binary: {error}")))?;

        self.session.load_target(&binary_path)?;
        self.session.launch_process()?;

        self.binary_bytes = if bytes.is_empty() { vec![0; 64] } else { bytes };
        self.instructions = Self::decode_disassembly(&self.binary_bytes);
        self.breakpoints.clear();
        self.backend = EngineBackend::Local;
        self.remote_config = None;
        self.remote_adapter.disconnect();
        self.remote_session.disconnect();
        self.binary_path = Some(binary_path);
        self.attached_target_label = None;
        self.session.detach();
        self.pc_index = 0;
        self.state = ExecutionState::Paused;
        self.refresh_state_window();

        Ok(())
    }

    fn attach_process(&mut self, request: AttachRequest) -> CoreResult<AttachResult> {
        if let Err(error_kind) = validate_attach_request(&request) {
            return Ok(AttachResult::failure(
                describe_attach_target(&request),
                error_kind,
                "Invalid attach target",
            ));
        }

        let target_exists = match &request.target {
            crate::types::AttachTarget::Pid(pid) => Self::process_exists_by_pid(*pid),
            crate::types::AttachTarget::ProcessName(name) => Self::process_exists_by_name(name),
        };
        if !target_exists {
            return Ok(AttachResult::failure(
                describe_attach_target(&request),
                AttachErrorKind::TargetNotFound,
                "Target process not found",
            ));
        }

        let target_label = describe_attach_target(&request);
        match self.session.attach_to_process(&request) {
            Ok(attached_target) => {
                self.backend = EngineBackend::Local;
                self.remote_config = None;
                self.remote_adapter.disconnect();
                self.remote_session.disconnect();
                self.attached_target_label = Some(attached_target.clone());
                self.binary_path = None;
                self.binary_bytes = Self::disassembly_seed_from_target(&target_label);
                self.instructions = Self::decode_disassembly(&self.binary_bytes);
                self.breakpoints.clear();
                self.pc_index = 0;
                self.state = ExecutionState::Paused;
                self.refresh_state_window();
                Ok(AttachResult::success(attached_target))
            }
            Err(error) => Ok(AttachResult::failure(
                target_label,
                Self::classify_attach_error(&error.message),
                error.message,
            )),
        }
    }

    fn connect_remote(&mut self, config: RemoteConfig) -> CoreResult<RemoteSessionStatus> {
        validate_remote_config(&config)
            .map_err(|error| CoreError::new(format!("Invalid remote config: {error}")))?;
        self.remote_session.begin_connect(&config);
        self.remote_config = Some(config.clone());

        loop {
            match self.remote_adapter.connect(&config) {
                Ok(session_id) => {
                    self.remote_session.mark_connected(&config, session_id);
                    self.backend = EngineBackend::Remote;
                    self.binary_path = None;
                    self.attached_target_label = Some(format!("xdbg-remote@{}", config.endpoint));
                    self.binary_bytes = Self::disassembly_seed_from_target(&config.endpoint);
                    self.instructions = Self::decode_disassembly(&self.binary_bytes);
                    self.breakpoints.clear();
                    self.pc_index = 0;
                    self.state = ExecutionState::Paused;
                    self.refresh_state_window();
                    return Ok(self.remote_session.status());
                }
                Err(error) => {
                    self.remote_session
                        .mark_failed(&config, error.kind, error.message.clone());
                    if self.remote_session.can_retry() {
                        thread::sleep(Duration::from_millis(self.remote_session.retry_delay_ms()));
                        continue;
                    }
                    self.backend = EngineBackend::Local;
                    return Ok(self.remote_session.status());
                }
            }
        }
    }

    fn disconnect_remote(&mut self) {
        self.remote_adapter.disconnect();
        self.remote_session.disconnect();
        self.remote_config = None;
        if self.backend == EngineBackend::Remote {
            self.backend = EngineBackend::Local;
            self.attached_target_label = None;
            self.state = ExecutionState::NoTarget;
            self.instructions.clear();
            self.breakpoints.clear();
            self.memory = MemorySnapshot {
                address: Self::base_address(),
                bytes: vec![0; 0x100],
            };
        }
    }

    fn remote_session_status(&self) -> RemoteSessionStatus {
        self.remote_session.status()
    }

    fn attach_lifecycle(&self) -> SessionLifecycle {
        self.session.lifecycle()
    }

    fn step_in(&mut self) -> CoreResult<Option<u64>> {
        self.ensure_target()?;
        if self.backend == EngineBackend::Remote {
            self.dispatch_remote_command(RemoteCommand::StepIn)?;
        }
        let pc = self.advance_one_instruction();
        self.refresh_state_window();
        Ok(pc)
    }

    fn step_over(&mut self) -> CoreResult<Option<u64>> {
        self.step_in()
    }

    fn continue_exec(&mut self) -> CoreResult<Option<u64>> {
        self.ensure_target()?;
        if self.backend == EngineBackend::Remote {
            self.dispatch_remote_command(RemoteCommand::Continue)?;
        }
        self.state = ExecutionState::Running;

        while let Some(pc) = self.advance_one_instruction() {
            if self.breakpoints.contains(&pc) {
                self.state = ExecutionState::Paused;
                self.refresh_state_window();
                return Ok(Some(pc));
            }

            if self.state == ExecutionState::Exited {
                self.refresh_state_window();
                return Ok(None);
            }
        }

        self.refresh_state_window();
        Ok(None)
    }

    fn pause_exec(&mut self) -> CoreResult<Option<u64>> {
        self.ensure_target()?;
        if self.backend == EngineBackend::Remote {
            self.dispatch_remote_command(RemoteCommand::Pause)?;
        }
        self.state = ExecutionState::Paused;
        self.refresh_state_window();
        Ok(self.current_pc())
    }

    fn toggle_breakpoint(&mut self, address: u64) -> CoreResult<()> {
        self.ensure_target()?;

        if self.breakpoints.contains(&address) {
            self.breakpoints.remove(&address);
        } else {
            self.breakpoints.insert(address);
        }

        Ok(())
    }

    fn read_registers(&self) -> CoreResult<Vec<RegisterValue>> {
        self.ensure_target()?;
        if self.backend == EngineBackend::Remote {
            if let Err(error) = self.remote_adapter.dispatch(RemoteCommand::ReadRegisters) {
                let kind = classify_remote_error(&error.message);
                return Err(CoreError::new(format!(
                    "Remote read-register failed ({kind}): {}",
                    error.message
                )));
            }
        }
        Ok(self.computed_registers())
    }

    fn read_memory(&mut self, address: u64, size: usize) -> CoreResult<MemorySnapshot> {
        self.ensure_target()?;
        if self.backend == EngineBackend::Remote {
            self.dispatch_remote_command(RemoteCommand::ReadMemory { address, size })?;
        }

        let base = Self::base_address();
        let offset = address.saturating_sub(base) as usize;
        let mut bytes = Vec::with_capacity(size);

        for idx in 0..size {
            let source_index = offset + idx;
            bytes.push(*self.binary_bytes.get(source_index).unwrap_or(&0));
        }

        Ok(MemorySnapshot { address, bytes })
    }

    fn fetch_disassembly(&self) -> CoreResult<Vec<AssemblyInstruction>> {
        self.ensure_target()?;
        Ok(self.instructions.clone())
    }

    fn snapshot(&self) -> DebuggerSnapshot {
        DebuggerSnapshot {
            binary_path: self.binary_path.clone(),
            attached_target: self.attached_target_label.clone(),
            state: self.state,
            current_pc: self.current_pc(),
            instructions: self.instructions.clone(),
            breakpoints: self.breakpoints.clone(),
            registers: self.computed_registers(),
            memory: self.memory.clone(),
            remote: self.remote_session.status(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use tempfile::tempdir;

    use super::{DebugEngine, MockLldbEngine};
    use crate::core::types::SessionLifecycle;
    use crate::types::{
        AttachErrorKind, AttachRequest, ExecutionState, RemoteConfig, RemoteSessionState,
    };

    #[test]
    fn loads_arm64_sample_and_disassembles() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let dir = tempdir().expect("tempdir should work");
        let binary_path = dir.path().join("sample_arm64.bin");
        let arm64_sample = include_bytes!("../../tests/fixtures/sample_arm64.bin");
        fs::write(&binary_path, arm64_sample).expect("binary fixture write should succeed");

        engine
            .load_binary(binary_path)
            .expect("binary load should succeed");
        let disassembly = engine
            .fetch_disassembly()
            .expect("disassembly should be available");

        assert!(!disassembly.is_empty());
        assert_eq!(disassembly[0].address, 0x1000);
    }

    #[test]
    fn sets_and_removes_breakpoints() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let dir = tempdir().expect("tempdir should work");
        let binary_path = dir.path().join("sample.bin");
        fs::write(&binary_path, [1, 2, 3, 4, 5, 6, 7, 8])
            .expect("binary fixture write should succeed");
        engine
            .load_binary(binary_path)
            .expect("binary load should succeed");

        engine
            .toggle_breakpoint(0x1004)
            .expect("set breakpoint should succeed");
        assert!(engine.snapshot().breakpoints.contains(&0x1004));

        engine
            .toggle_breakpoint(0x1004)
            .expect("remove breakpoint should succeed");
        assert!(!engine.snapshot().breakpoints.contains(&0x1004));
    }

    #[test]
    fn stepping_updates_program_counter_and_state_views() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let dir = tempdir().expect("tempdir should work");
        let binary_path = dir.path().join("sample.bin");
        fs::write(&binary_path, [10, 11, 12, 13, 14, 15, 16, 17])
            .expect("binary fixture write should succeed");
        engine
            .load_binary(binary_path)
            .expect("binary load should succeed");

        let old_pc = engine
            .snapshot()
            .current_pc
            .expect("pc should exist after load");
        engine.step_in().expect("step should succeed");
        let new_pc = engine
            .snapshot()
            .current_pc
            .expect("pc should exist after stepping");

        assert!(new_pc > old_pc);

        let registers = engine
            .read_registers()
            .expect("registers should be readable");
        assert_eq!(registers.iter().filter(|reg| reg.name == "pc").count(), 1);

        let memory = engine
            .read_memory(new_pc, 0x40)
            .expect("memory should be readable");
        assert_eq!(memory.bytes.len(), 0x40);
    }

    #[test]
    fn attach_rejects_invalid_target() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let result = engine
            .attach_process(AttachRequest::by_process_name(" "))
            .expect("attach command should return structured result");

        assert!(!result.attached);
        assert_eq!(result.error, Some(AttachErrorKind::TargetNotFound));
    }

    #[test]
    fn attach_to_existing_pid_updates_lifecycle() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let result = engine
            .attach_process(AttachRequest::by_pid(std::process::id()))
            .expect("attach should return result");

        assert!(
            result.attached,
            "expected attach success: {}",
            result.message
        );
        assert_eq!(engine.attach_lifecycle(), SessionLifecycle::Attached);
        assert_eq!(engine.snapshot().state, ExecutionState::Paused);
    }

    #[test]
    fn attach_to_unknown_pid_returns_target_not_found() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let pid = u32::MAX - 1;
        let result = engine
            .attach_process(AttachRequest::by_pid(pid))
            .expect("attach should return structured result");

        assert!(!result.attached);
        assert_eq!(result.error, Some(AttachErrorKind::TargetNotFound));
    }

    #[test]
    fn attached_session_supports_step_and_continue_controls() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let attach = engine
            .attach_process(AttachRequest::by_pid(std::process::id()))
            .expect("attach should return structured result");
        assert!(attach.attached);

        let first_pc = engine.snapshot().current_pc.expect("pc should exist");
        let step_pc = engine
            .step_in()
            .expect("step should work")
            .expect("pc should advance");
        assert!(step_pc > first_pc);

        engine
            .toggle_breakpoint(step_pc + 4)
            .expect("set breakpoint should work");
        let _ = engine.continue_exec().expect("continue should work");
        thread::sleep(Duration::from_millis(5));
    }

    #[test]
    fn remote_connect_and_disconnect_update_session_status() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let config = RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            ..RemoteConfig::default()
        };

        let status = engine
            .connect_remote(config.clone())
            .expect("remote connect should produce status");
        assert_eq!(status.state, RemoteSessionState::Connected);

        engine.disconnect_remote();
        assert_eq!(
            engine.remote_session_status().state,
            RemoteSessionState::Disconnected
        );
    }

    #[test]
    fn remote_connect_reports_degraded_status_for_bad_token() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let config = RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            token: Some("bad-token".to_string()),
            retry_count: 0,
            ..RemoteConfig::default()
        };

        let status = engine
            .connect_remote(config)
            .expect("remote connect should return status");
        assert_eq!(status.state, RemoteSessionState::Degraded);
    }
}

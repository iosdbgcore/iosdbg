use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::session::LldbSession;
use crate::types::{
    AssemblyInstruction, CoreError, CoreResult, DebuggerSnapshot, ExecutionState, MemorySnapshot,
    RegisterValue,
};

pub trait DebugEngine: Send {
    fn load_binary(&mut self, binary_path: PathBuf) -> CoreResult<()>;
    fn step_in(&mut self) -> CoreResult<Option<u64>>;
    fn step_over(&mut self) -> CoreResult<Option<u64>>;
    fn continue_exec(&mut self) -> CoreResult<Option<u64>>;
    fn toggle_breakpoint(&mut self, address: u64) -> CoreResult<()>;
    fn read_registers(&self) -> CoreResult<Vec<RegisterValue>>;
    fn read_memory(&mut self, address: u64, size: usize) -> CoreResult<MemorySnapshot>;
    fn fetch_disassembly(&self) -> CoreResult<Vec<AssemblyInstruction>>;
    fn snapshot(&self) -> DebuggerSnapshot;
}

pub struct MockLldbEngine {
    session: LldbSession,
    binary_path: Option<PathBuf>,
    binary_bytes: Vec<u8>,
    instructions: Vec<AssemblyInstruction>,
    breakpoints: HashSet<u64>,
    pc_index: usize,
    state: ExecutionState,
    memory: MemorySnapshot,
}

impl MockLldbEngine {
    pub fn new() -> CoreResult<Self> {
        Ok(Self {
            session: LldbSession::initialize()?,
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
        if self.binary_path.is_none() {
            return Err(CoreError::new("No binary loaded"));
        }
        Ok(())
    }

    fn refresh_state_window(&mut self) {
        let address = self.current_pc().unwrap_or(Self::base_address());
        self.memory = self
            .read_memory(address, 0x100)
            .unwrap_or(MemorySnapshot { address, bytes: vec![] });
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
        self.binary_path = Some(binary_path);
        self.pc_index = 0;
        self.state = ExecutionState::Paused;
        self.refresh_state_window();

        Ok(())
    }

    fn step_in(&mut self) -> CoreResult<Option<u64>> {
        self.ensure_target()?;
        let pc = self.advance_one_instruction();
        self.refresh_state_window();
        Ok(pc)
    }

    fn step_over(&mut self) -> CoreResult<Option<u64>> {
        self.step_in()
    }

    fn continue_exec(&mut self) -> CoreResult<Option<u64>> {
        self.ensure_target()?;
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
        Ok(self.computed_registers())
    }

    fn read_memory(&mut self, address: u64, size: usize) -> CoreResult<MemorySnapshot> {
        self.ensure_target()?;

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
            state: self.state,
            current_pc: self.current_pc(),
            instructions: self.instructions.clone(),
            breakpoints: self.breakpoints.clone(),
            registers: self.computed_registers(),
            memory: self.memory.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{DebugEngine, MockLldbEngine};

    #[test]
    fn loads_arm64_sample_and_disassembles() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let dir = tempdir().expect("tempdir should work");
        let binary_path = dir.path().join("sample_arm64.bin");
        let arm64_sample = include_bytes!("../../tests/fixtures/sample_arm64.bin");
        fs::write(&binary_path, arm64_sample).expect("binary fixture write should succeed");

        engine.load_binary(binary_path).expect("binary load should succeed");
        let disassembly = engine.fetch_disassembly().expect("disassembly should be available");

        assert!(!disassembly.is_empty());
        assert_eq!(disassembly[0].address, 0x1000);
    }

    #[test]
    fn sets_and_removes_breakpoints() {
        let mut engine = MockLldbEngine::new().expect("engine should initialize");
        let dir = tempdir().expect("tempdir should work");
        let binary_path = dir.path().join("sample.bin");
        fs::write(&binary_path, [1, 2, 3, 4, 5, 6, 7, 8]).expect("binary fixture write should succeed");
        engine.load_binary(binary_path).expect("binary load should succeed");

        engine.toggle_breakpoint(0x1004).expect("set breakpoint should succeed");
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
        fs::write(&binary_path, [10, 11, 12, 13, 14, 15, 16, 17]).expect("binary fixture write should succeed");
        engine.load_binary(binary_path).expect("binary load should succeed");

        let old_pc = engine.snapshot().current_pc.expect("pc should exist after load");
        engine.step_in().expect("step should succeed");
        let new_pc = engine
            .snapshot()
            .current_pc
            .expect("pc should exist after stepping");

        assert!(new_pc > old_pc);

        let registers = engine.read_registers().expect("registers should be readable");
        assert_eq!(registers.iter().filter(|reg| reg.name == "pc").count(), 1);

        let memory = engine.read_memory(new_pc, 0x40).expect("memory should be readable");
        assert_eq!(memory.bytes.len(), 0x40);
    }
}

## Why

Current iOS debugging workflows lack a visual, x64dbg-style interface for post-processing (unpacked/decrypted) binaries. Developers need an LLDB-based debugger with rich assembly visualization, interactive breakpoints, and real-time state inspection to efficiently analyze iOS executables.

## What Changes

- Introduce a desktop debugger application built in Rust with LLDB as the debugging engine
- Provide x64dbg-inspired UI for assembly code display, execution control, and state visualization
- Support loading processed binary files and displaying disassembled code
- Enable interactive debugging with breakpoints, step execution, and execution flow highlighting
- Display real-time register values and memory contents during debugging sessions

## Capabilities

### New Capabilities

- `binary-loading`: Load and parse target binary files (post-unpacking/decryption) for debugging
- `assembly-display`: Render disassembled code with syntax highlighting and address mapping
- `breakpoint-management`: Set, remove, and track breakpoints at assembly instruction level
- `execution-control`: Step through code (step-in, step-over, continue) with visual feedback
- `execution-visualization`: Highlight current instruction and track execution flow changes
- `register-inspection`: Display and update CPU register values in real-time
- `memory-inspection`: Read and display memory contents at specified addresses
- `lldb-integration`: Interface with LLDB API for all debugging operations
- `ui-framework`: Desktop GUI with multi-panel layout (assembly, registers, memory, controls)

### Modified Capabilities

_None - this is a new project_

## Impact

**New Dependencies:**
- LLDB library and Rust bindings (lldb-sys or similar)
- Rust GUI framework (egui, iced, or tauri)
- Disassembly library (capstone-rs or LLDB's built-in disassembler)

**Architecture:**
- Two-layer design: debugger core (LLDB interaction) + presentation layer (UI)
- Event-driven communication between core and UI to avoid tight coupling

**Deliverables:**
- P0 (MVP): Binary loading, assembly display, basic breakpoints, step execution, current line highlighting
- P1 (Enhanced): Register panel, memory viewer, breakpoint hit context
- P2 (Polish): x64dbg-style UI refinements, interaction optimizations

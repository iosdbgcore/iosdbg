# rust-lldb-visual-debugger

> Visual debugger based on Rust + LLDB + egui

[中文](./README.md)

A modern visual debugging tool that provides an intuitive graphical interface for debugging binary programs. By integrating the LLDB debugging engine and egui graphics framework, it offers developers a smooth debugging experience.

## Features

- **Binary Loading** - Load and parse executable files
- **Breakpoint Management** - Set, delete, and manage breakpoints
- **Execution Control** - Step execution, continue, pause debugging
- **Register Inspection** - Real-time view and monitor CPU register states
- **Memory Inspection** - View and analyze process memory contents
- **Assembly Display** - Display disassembled code
- **Execution Visualization** - Graphical display of program execution flow
- **LLDB Integration** - Based on the powerful LLDB debugging engine
- **Modern UI** - Responsive graphical interface built with egui

## Installation

### Requirements

- Rust toolchain (edition 2021 or higher)
- Cargo package manager
- LLDB development libraries (optional, for real-lldb feature)

### Build Steps

1. Clone the repository:
```bash
git clone <repository-url>
cd iosDbg
```

2. Build the project (using mock-lldb, no LLDB dependencies required):
```bash
cargo build --release
```

3. Or use real LLDB (requires LLDB development libraries):
```bash
cargo build --release --features real-lldb
```

## Quick Start

Run the debugger:

```bash
cargo run --release
```

After launching, use the graphical interface:
1. Click the "Load Binary" button to select an executable file to debug
2. Set breakpoints and start debugging
3. Use the control panel for step execution, continue, and other operations

## Documentation

For detailed technical specifications and usage instructions, please refer to:

- [Binary Loading](./openspec/specs/binary-loading/spec.md)
- [Breakpoint Management](./openspec/specs/breakpoint-management/spec.md)
- [Execution Control](./openspec/specs/execution-control/spec.md)
- [Register Inspection](./openspec/specs/register-inspection/spec.md)
- [Memory Inspection](./openspec/specs/memory-inspection/spec.md)
- [Assembly Display](./openspec/specs/assembly-display/spec.md)
- [Execution Visualization](./openspec/specs/execution-visualization/spec.md)
- [LLDB Integration](./openspec/specs/lldb-integration/spec.md)
- [UI Framework](./openspec/specs/ui-framework/spec.md)

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Commit your changes (`git commit -m 'Add some feature'`)
4. Push to the branch (`git push origin feature/your-feature`)
5. Submit a Pull Request

**Note**: When modifying README, please update both `README.md` and `README.en.md` files synchronously.

## License

MIT License

# rust-lldb-visual-debugger

> Visual debugger based on Rust + LLDB + egui

[中文](./README.md)

A modern visual debugging tool that provides an intuitive graphical interface for debugging binary programs. By integrating the LLDB debugging engine and egui graphics framework, it offers developers a smooth debugging experience.

## Features

- **Binary Loading** - Load and parse executable files
- **Attach Process Debugging** - Attach by PID or process name to a running process
- **x64dbg Remote Integration** - Connect by endpoint/token and forward core debug actions to remote
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

## Build macOS Client

The repository includes an `.app` packaging script (must run on macOS):

```bash
./scripts/build_macos_client.sh --target aarch64-apple-darwin
```

Common usage:

- Build for current machine architecture: `./scripts/build_macos_client.sh`
- Intel macOS client: `./scripts/build_macos_client.sh --target x86_64-apple-darwin`
- Apple Silicon client: `./scripts/build_macos_client.sh --target aarch64-apple-darwin`
- Universal2 client: `./scripts/build_macos_client.sh --universal`
- Enable real LLDB: `./scripts/build_macos_client.sh --target aarch64-apple-darwin --features real-lldb`

Build outputs:

- `dist/macos/Rust LLDB Visual Debugger.app`
- `dist/macos/rust-lldb-visual-debugger-<version>-<target>.zip`

A GitHub Actions workflow is also included at `.github/workflows/build-macos-client.yml` so you can generate macOS artifacts from the Actions page.

### CI/CD Triggers and Publishing

- `pull_request`: validate workflow and build contract
- `push` to `main/master`: build dual-arch artifacts
- `push` tag (`v*`) or publish Release: upload x64/arm64 assets to GitHub Release
- `workflow_dispatch`: manual run with optional `features` and publish switch

See full details: [`docs/ci-cd.md`](./docs/ci-cd.md)

## Quick Start

Run the debugger:

```bash
cargo run --release
```

After launching, use the graphical interface:
1. Click the "Load Binary" button to select an executable file to debug
2. Or select PID/process name in the control bar and click Attach
3. Or configure remote endpoint/token in the control bar and click Connect
4. Set breakpoints and start debugging
5. Use the control panel for step execution, continue, and other operations

### Attach Preconditions and Troubleshooting

Prerequisites:

- LLDB is available on host (for `real-lldb` mode)
- Attach permission chain is satisfied (for macOS this often involves `task_for_pid`)
- PID/process name is valid and target process exists

Common attach error categories:

- `permission_denied`: insufficient permission
- `target_not_found`: target PID/name does not exist
- `timeout`: attach operation timed out
- `lldb_error`: uncategorized LLDB-side failure

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
- [x64dbg Parity Baseline](./docs/ui/x64dbg-parity-baseline.md)
- [x64dbg Reuse Feasibility and Deltas](./docs/ui/x64dbg-reuse-feasibility.md)
- [x64dbg Rewrite Regression Checklist](./docs/ui/x64dbg-regression-checklist.md)
- [x64dbg Remote Integration Guide](./docs/xdbg-remote-integration.md)
- [CI/CD Guide](./docs/ci-cd.md)

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

# macOS Dual Arch Build

## Purpose
Produce deterministic macOS build outputs for both Intel and Apple Silicon architectures in one CI run.

## Requirements

### Requirement: Build macOS x64 and arm64 artifacts in one workflow run
The CI system SHALL build macOS client artifacts for `x86_64-apple-darwin` and `aarch64-apple-darwin` in a single workflow execution.

#### Scenario: Matrix build succeeds for both targets
- **WHEN** the workflow is triggered by push, pull_request, or manual dispatch
- **THEN** the pipeline produces two successful build jobs, one per target architecture

### Requirement: Use deterministic artifact naming
The CI system SHALL name generated packages using a deterministic scheme containing application name, version, and target triple.

#### Scenario: Package naming validation
- **WHEN** arch-specific build packaging finishes
- **THEN** produced filenames match `rust-lldb-visual-debugger-<version>-<target>.zip`

### Requirement: Preserve build traceability metadata
The CI system SHALL expose commit and run metadata in build outputs for auditability.

#### Scenario: Metadata in workflow summary
- **WHEN** workflow run completes
- **THEN** summary includes source commit, run ID, and links to both architecture artifacts

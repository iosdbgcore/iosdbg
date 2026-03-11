## ADDED Requirements

### Requirement: Set breakpoint
The system SHALL allow users to set breakpoints at assembly instruction addresses.

#### Scenario: Set breakpoint on instruction
- **WHEN** user clicks on an assembly line
- **THEN** system sets a breakpoint at that address and marks it visually

### Requirement: Remove breakpoint
The system SHALL allow users to remove existing breakpoints.

#### Scenario: Remove breakpoint
- **WHEN** user clicks on a line with an active breakpoint
- **THEN** system removes the breakpoint and updates the visual indicator

### Requirement: Breakpoint hit notification
The system SHALL pause execution and notify the user when a breakpoint is hit.

#### Scenario: Hit breakpoint during execution
- **WHEN** execution reaches a breakpoint address
- **THEN** system pauses and highlights the breakpoint location

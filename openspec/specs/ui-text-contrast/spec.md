# UI Text Contrast

## Purpose

Ensure text elements in the macOS menu bar UI have sufficient contrast for readability.

## Requirements

### Requirement: Control panel text must have sufficient contrast
The control panel text colors SHALL provide minimum contrast ratio of 4.5:1 against the dark background (RGB 26,30,36) to ensure readability on macOS.

#### Scenario: Status badge text is readable
- **WHEN** user views the execution state badge in the control panel
- **THEN** the badge text SHALL use colors with luminance >= 180 (on 0-255 scale)

#### Scenario: Target information text is readable
- **WHEN** user views the attached target or loaded binary name
- **THEN** the text SHALL use RGB values with minimum component value >= 200

### Requirement: Status bar text must have sufficient contrast
The status bar text colors SHALL provide minimum contrast ratio of 4.5:1 against the dark background to ensure lifecycle and message information is clearly visible.

#### Scenario: Lifecycle badge text is readable
- **WHEN** user views the attach lifecycle status
- **THEN** the badge text SHALL use colors with luminance >= 180

#### Scenario: Status message text is readable
- **WHEN** user views status messages or error information
- **THEN** the text SHALL use RGB values with minimum component value >= 220

### Requirement: Color adjustments must preserve semantic meaning
Text color adjustments SHALL maintain the existing color hue and semantic associations while only increasing brightness.

#### Scenario: State colors remain distinguishable
- **WHEN** text colors are brightened for contrast
- **THEN** different states (success/warning/error) SHALL remain visually distinct by hue

#### Scenario: Theme consistency is maintained
- **WHEN** text colors are adjusted
- **THEN** the x64dbg theme aesthetic SHALL be preserved

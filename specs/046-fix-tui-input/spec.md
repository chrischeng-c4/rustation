# Feature Specification: Fix TUI Input Dialog Bug

**Feature Branch**: `046-fix-tui-input`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "Fix TUI input dialog bug - the Specify workflow input dialog appears but typing doesn't work, and add comprehensive unit/integration/E2E tests using ratatui TestBackend"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fix Input Dialog Keyboard Input (Priority: P1)

As a developer using the rstn TUI, I want to be able to type in the input dialog when running the Specify workflow, so that I can enter my feature description and proceed with spec-driven development.

**Why this priority**: This is a critical bug that completely blocks the SDD workflow in TUI mode. Users cannot use the Specify command at all when this bug is present.

**Independent Test**: Can be fully tested by launching rstn TUI, navigating to Worktree > SDD Workflow > Specify, pressing Enter, and verifying that typed characters appear in the input dialog.

**Acceptance Scenarios**:

1. **Given** the TUI is running and focused on the Specify command, **When** I press Enter, **Then** an input dialog should appear prompting for a feature description
2. **Given** the input dialog is visible, **When** I type alphanumeric characters, **Then** the characters should appear in the input field
3. **Given** the input dialog has text entered, **When** I press Backspace, **Then** the last character should be deleted
4. **Given** the input dialog has text entered, **When** I press Alt+Enter (multiline mode), **Then** the input should be submitted and the Specify phase should start

---

### User Story 2 - Unit Tests for Input Handling (Priority: P2)

As a developer maintaining rstn, I want comprehensive unit tests for the input dialog and keyboard handling, so that I can prevent regressions and verify behavior in isolation.

**Why this priority**: Unit tests are essential for maintaining code quality and catching regressions early. They should be added alongside the bug fix.

**Independent Test**: Can be tested by running `cargo test` and verifying all new unit tests pass.

**Acceptance Scenarios**:

1. **Given** the input dialog widget is created, **When** insert_char is called with a character, **Then** the character should be added to the input value
2. **Given** the app receives ViewAction::RequestInput, **When** the action is processed, **Then** input_mode should be true and input_dialog should be Some
3. **Given** input_mode is true and a key event arrives, **When** handle_key_event is called, **Then** the key should be routed to handle_key_event_in_input_mode
4. **Given** the input dialog is active, **When** Escape is pressed, **Then** the dialog should be cancelled and input_mode set to false

---

### User Story 3 - Integration Tests for SDD Workflow (Priority: P2)

As a developer maintaining rstn, I want integration tests that verify the complete Specify workflow, so that I can ensure all components work together correctly.

**Why this priority**: Integration tests catch issues that unit tests miss by testing component interactions. They are critical for workflow correctness.

**Independent Test**: Can be tested by running `cargo test --test sdd_workflow_test` and verifying all integration tests pass.

**Acceptance Scenarios**:

1. **Given** a test harness simulating TUI input, **When** the Specify workflow is triggered and text is typed, **Then** the characters should appear in the input dialog
2. **Given** the input dialog has content, **When** Enter/Alt+Enter is pressed, **Then** the input value should be passed to the Specify command

---

### User Story 4 - E2E Tests with TestBackend (Priority: P3)

As a developer maintaining rstn, I want E2E tests using ratatui's TestBackend, so that I can verify the complete TUI rendering and interaction flow.

**Why this priority**: E2E tests provide the highest confidence that the feature works correctly from the user's perspective, but are slower and more complex to maintain.

**Independent Test**: Can be tested by running `cargo test --test e2e` and verifying the E2E test suite passes.

**Acceptance Scenarios**:

1. **Given** a TUI test harness with TestBackend, **When** the Specify workflow is triggered, **Then** the input dialog should render in the terminal buffer
2. **Given** the input dialog is rendered, **When** characters are sent to the app, **Then** the buffer should update to show the typed characters
3. **Given** text is visible in the buffer, **When** Escape is pressed, **Then** the dialog should disappear from the buffer

---

### Edge Cases

- What happens when input dialog receives special characters (unicode, emoji)?
- What happens when input exceeds the visible width of the dialog?
- What happens when the terminal is resized while input dialog is open?
- What happens when input_mode is true but input_dialog is None (invalid state)?
- What happens when multiple rapid key events are received (race conditions)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept keyboard input when the input dialog is visible
- **FR-002**: System MUST display typed characters in the input field immediately
- **FR-003**: System MUST support cursor movement (left, right, home, end) in the input dialog
- **FR-004**: System MUST support backspace to delete the character before cursor
- **FR-005**: System MUST submit the input when the appropriate key is pressed (Enter for single-line, Alt+Enter for multiline)
- **FR-006**: System MUST cancel input and close the dialog when Escape is pressed
- **FR-007**: System MUST include unit tests for InputDialog character insertion and cursor movement
- **FR-008**: System MUST include unit tests for App.handle_key_event input mode routing
- **FR-009**: System MUST include integration tests for the complete Specify workflow
- **FR-010**: System MUST include E2E tests using ratatui TestBackend for visual verification

### Key Entities

- **InputDialog**: Modal dialog widget containing a TextInput, title, and optional description
- **TextInput**: Underlying text input widget handling character insertion, cursor movement, and multiline support
- **App.input_mode**: Boolean flag indicating whether the app is in input mode
- **ViewAction::RequestInput**: Action that triggers input dialog creation

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can type in the Specify input dialog and see characters appear immediately
- **SC-002**: All existing tests continue to pass (670+ tests)
- **SC-003**: At least 12 new unit tests are added for input handling
- **SC-004**: At least 6 new integration tests are added for SDD workflow
- **SC-005**: At least 3 new E2E tests are added using TestBackend
- **SC-006**: The bug fix does not introduce any new regressions in other TUI features
- **SC-007**: Input dialog responds to keyboard within normal user perception (instantaneous feedback)

## Assumptions

- The bug is in the event routing or state management, not in the underlying widget implementation
- TestBackend from ratatui is suitable for E2E testing without actual terminal rendering
- The existing test infrastructure can be extended without major refactoring
- Debug logging will be added first to identify the root cause before implementing the fix

## Out of Scope

- Mouse input support for the input dialog
- Input validation or formatting
- Autocomplete or suggestion features
- Performance optimization beyond fixing the bug

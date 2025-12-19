# Feature Specification: TUI Input Keybindings

**Feature Branch**: `048-tui-input-keybindings`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "TUI input dialog keyboard shortcut change: Replace alt+enter with shift+enter for new line, keep enter for submit. Fixes conflict with WezTerm terminal emulator."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Submit Input with Enter (Priority: P1)

Users can submit their input in the TUI input dialog by pressing Enter. This is the standard, expected behavior across most applications.

**Why this priority**: Submit is the primary action in an input dialog. Users expect Enter to submit.

**Independent Test**: Open rstn TUI, trigger an input dialog (e.g., service name input), type text, press Enter. Input should be submitted and dialog should close.

**Acceptance Scenarios**:

1. **Given** an input dialog is open with text entered, **When** user presses Enter, **Then** the input is submitted and the dialog closes
2. **Given** an input dialog is open with empty text, **When** user presses Enter, **Then** the input is submitted (empty string) and dialog closes
3. **Given** an input dialog is open, **When** user presses Enter multiple times rapidly, **Then** only the first press is processed (no duplicate submissions)

---

### User Story 2 - Insert Newline with Shift+Enter (Priority: P1)

Users can insert a newline character in multiline input dialogs by pressing Shift+Enter. This allows multi-line text entry while reserving Enter for submission.

**Why this priority**: Multiline input is essential for certain dialogs. Shift+Enter is a common convention (Slack, Discord, many chat apps).

**Independent Test**: Open rstn TUI, trigger a multiline input dialog, type "line1", press Shift+Enter, type "line2", press Enter. Submitted text should be "line1\nline2".

**Acceptance Scenarios**:

1. **Given** a multiline input dialog is open, **When** user presses Shift+Enter, **Then** a newline is inserted at the cursor position
2. **Given** cursor is in the middle of text, **When** user presses Shift+Enter, **Then** text after cursor moves to new line
3. **Given** a single-line input dialog is open, **When** user presses Shift+Enter, **Then** input is submitted (same as Enter, no newline in single-line mode)

---

### User Story 3 - No Conflict with Terminal Emulators (Priority: P1)

The keyboard shortcuts work correctly in WezTerm and other terminal emulators without triggering terminal-level shortcuts.

**Why this priority**: The current Alt+Enter shortcut conflicts with WezTerm's fullscreen toggle, making multiline input unusable.

**Independent Test**: Run rstn in WezTerm, open multiline input dialog, verify Shift+Enter creates newline and Enter submits without triggering WezTerm shortcuts.

**Acceptance Scenarios**:

1. **Given** rstn running in WezTerm, **When** user presses Shift+Enter in input dialog, **Then** newline is inserted (WezTerm does not intercept)
2. **Given** rstn running in WezTerm, **When** user presses Enter in input dialog, **Then** input is submitted (no terminal interference)
3. **Given** rstn running in iTerm2/Alacritty/Kitty, **When** user uses Enter and Shift+Enter, **Then** behavior is consistent with WezTerm

---

### Edge Cases

- What happens when Shift+Enter is pressed in a read-only or disabled input? → Input is ignored, no crash
- What happens when input dialog is not focused? → Key events are not captured by the dialog
- How does system handle very long multiline input (100+ lines)? → Scrollable input area, no performance degradation

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST submit input when Enter key is pressed in input dialogs
- **FR-002**: System MUST insert a newline when Shift+Enter is pressed in multiline input dialogs
- **FR-003**: System MUST treat Shift+Enter as Submit (same as Enter) in single-line input dialogs
- **FR-004**: System MUST NOT use Alt+Enter for any input dialog functionality (removes conflict)
- **FR-005**: System MUST update help text in input dialogs to show "Shift+Enter: new line" for multiline mode
- **FR-006**: System MUST update all existing tests to use the new keybindings

### Key Entities

- **InputDialog**: TUI widget that captures user text input, can be single-line or multiline
- **KeyEvent**: Keyboard event with key code and modifiers (Shift, Alt, Ctrl)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can submit input using Enter in under 1 keypress (no modifier required)
- **SC-002**: Users can create multiline text without triggering terminal emulator shortcuts
- **SC-003**: All existing TUI input dialog tests pass with updated keybindings
- **SC-004**: Help text accurately reflects available keyboard shortcuts

## Assumptions

- WezTerm does not intercept Shift+Enter (verified: Shift+Enter is not a default WezTerm shortcut)
- Other common terminal emulators (iTerm2, Alacritty, Kitty) also pass through Shift+Enter
- The crossterm library correctly reports Shift modifier with Enter key

## Out of Scope

- Custom keybinding configuration (users cannot remap these shortcuts)
- Alt+Enter functionality in non-input contexts (e.g., full-screen toggle for rstn itself)

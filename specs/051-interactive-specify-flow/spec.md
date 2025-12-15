# Feature 051: Interactive Specify Flow

## Overview

Transform the `/speckit.specify` workflow to use the drop dialog pattern (like Intelligent Commit in feature 050), keeping users in the TUI instead of shelling out to external processes. This feature focuses on the UI/UX transformation while still utilizing the existing `create-new-feature.sh` shell script.

## Problem Statement

Current specify workflow interrupts the TUI experience:

1. **Context switching**: User must leave TUI mental model to interact with external Claude Code process
2. **No preview/edit**: Generated spec is written directly to file with no review step
3. **Inconsistent UX**: Different pattern from Intelligent Commit workflow
4. **No inline editing**: Cannot edit spec before saving to disk

## User Stories

### As a developer creating a new feature spec
- I want to input the feature description directly in the TUI
- So that I stay in my workflow without context switching

### As a developer reviewing generated specs
- I want to see the generated spec in the Content area before it's saved
- So that I can verify it meets my needs

### As a developer editing specs
- I want to edit the generated spec inline before saving
- So that I can make quick adjustments without reopening files

### As a developer using rstn
- I want specify to work like Intelligent Commit
- So that I have a consistent, predictable UX pattern

## Requirements

### Functional Requirements

**FR-1: Specify Input Dialog**
- Add "Specify" action to Commands pane in Worktree view
- When triggered, Content area transforms to Spec Input view
- Show text input area for feature description
- Support multi-line input (Ctrl+Enter for newline)
- Enter key submits, Esc key cancels

**FR-2: Spec Generation**
- On submit, show "Generating spec..." status in Output area
- Call existing `create-new-feature.sh` shell script with feature description
- Capture generated spec content
- On success, transition to Spec Review view
- On error, show error in Output area and stay in Input view

**FR-3: Spec Review Dialog**
- Display generated spec in Content area (read-only initially)
- Show feature number and title at top
- Show action hints: [Enter] Save, [e] Edit, [Esc] Cancel
- Syntax highlighting for markdown (if possible)

**FR-4: Inline Editing**
- Press 'e' to enable edit mode
- Full text editing with cursor movement (arrows, Home, End)
- Multi-line editing support
- [Ctrl+S] Save, [Esc] Cancel editing (return to review)
- Show edit mode indicator in title

**FR-5: Save and Complete**
- Enter key (or Ctrl+S in edit mode) saves spec to spec.md
- Show success message: "Spec saved: specs/{NNN}-{name}/spec.md"
- Return Content area to normal Spec view
- Load the newly created spec for display

**FR-6: Status Updates**
- Clear status bar when entering input mode
- Show "Generating spec..." during generation
- Show "Review spec" in review mode
- Show "Editing spec" in edit mode
- Show "Spec saved" after successful save

### Non-Functional Requirements

**NFR-1: Performance**
- Input view should render instantly
- Spec generation time depends on Claude Code (no change)
- Review/edit transitions should be instant (<50ms)

**NFR-2: Usability**
- Drop dialog pattern consistent with feature 050
- Keyboard-first interaction (no mouse required)
- Clear visual feedback for each state
- Graceful error handling with actionable messages

**NFR-3: Compatibility**
- Continue using existing `create-new-feature.sh` script
- No changes to shell script required
- Works with current spec-kit infrastructure

**NFR-4: Maintainability**
- State management similar to CommitReview in feature 050
- Clear separation of concerns (UI vs business logic)
- Well-documented state transitions

## Architecture

### Content Types
Add new enum variants to `ContentType`:
```rust
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,
    SpecifyInput,    // New: Input feature description
    SpecifyReview,   // New: Review/edit generated spec
}
```

### State Management
Add to `WorktreeView`:
```rust
pub struct WorktreeView {
    // ... existing fields ...

    // Specify state (Feature 051)
    pub specify_input: String,
    pub specify_cursor: usize,
    pub specify_generated_spec: Option<String>,
    pub specify_feature_number: Option<String>,
    pub specify_feature_name: Option<String>,
    pub specify_edit_mode: bool,
    pub specify_error: Option<String>,
}
```

### Methods
```rust
impl WorktreeView {
    // Start specify input
    pub fn start_specify_input(&mut self);

    // Handle input in specify mode
    pub fn handle_specify_input(&mut self, key: KeyEvent) -> ViewAction;

    // Submit description for generation
    pub fn submit_specify_description(&mut self) -> ViewAction;

    // Load generated spec for review
    pub fn load_generated_spec(&mut self, spec: String, number: String, name: String);

    // Toggle edit mode
    pub fn toggle_specify_edit_mode(&mut self);

    // Save spec to file
    pub fn save_specify_spec(&mut self) -> ViewAction;

    // Cancel specify workflow
    pub fn cancel_specify(&mut self);
}
```

### Events
Add to `Event` enum:
```rust
pub enum Event {
    // ... existing events ...

    // Specify workflow events
    SpecifyGenerationStarted,
    SpecifyGenerationCompleted {
        spec: String,
        number: String,
        name: String
    },
    SpecifyGenerationFailed {
        error: String
    },
    SpecifySaved {
        path: String
    },
}
```

### Actions
Add to `ViewAction` enum:
```rust
pub enum ViewAction {
    // ... existing actions ...

    // Trigger specify generation
    GenerateSpec { description: String },

    // Save generated spec
    SaveSpec {
        content: String,
        number: String,
        name: String
    },
}
```

## Integration Points

### Shell Script Integration
- Call `create-new-feature.sh` via `tokio::process::Command`
- Pass feature description as argument
- Capture stdout/stderr
- Parse output to extract feature number and name
- Read generated spec.md content

### Commands Pane
- Add "Specify" to list of available commands
- Position after spec-kit phases, before git commands
- Same styling as other commands

### Keyboard Shortcuts
- In Input mode:
  - Ctrl+Enter: New line
  - Enter: Submit
  - Esc: Cancel
  - Standard text editing (arrows, backspace, delete, Home, End)
- In Review mode:
  - Enter: Save
  - e: Edit
  - Esc: Cancel
- In Edit mode:
  - Ctrl+S: Save
  - Esc: Cancel edit (back to review)
  - Standard text editing

## User Flow

```
1. User triggers "Specify" from Commands pane
   ↓
2. Content area → SpecifyInput view
   - Text input for feature description
   - User types description
   - User presses Enter
   ↓
3. Output shows "Generating spec..."
   - rstn calls create-new-feature.sh
   - Shell script generates spec
   ↓
4. On success → SpecifyReview view
   - Display generated spec
   - User reviews content
   - Options: [Enter] Save, [e] Edit, [Esc] Cancel
   ↓
5a. If user presses 'e' → Edit mode
    - Enable inline editing
    - User makes changes
    - Press Ctrl+S or Enter to save
    ↓
5b. If user presses Enter → Save
    - Write to spec.md
    - Show success message
    - Load spec in Content area
    ↓
6. Return to normal Worktree view with new spec loaded
```

## Error Handling

**Input validation:**
- Empty description → Show error "Description cannot be empty"
- Whitespace only → Show error "Please enter a meaningful description"

**Generation errors:**
- Shell script fails → Show error in Output area, stay in Input mode
- Timeout (>60s) → Show timeout error, allow retry or cancel
- Missing shell script → Show error with path to expected script

**Save errors:**
- File write fails → Show error, allow retry or copy to clipboard
- Directory creation fails → Show error with details

## Testing Strategy

### Unit Tests
- State management (start_specify_input, cancel_specify, etc.)
- Input handling (character input, cursor movement)
- Validation (empty input, whitespace)
- State transitions (Input → Review → Edit → Save)

### Integration Tests
- Full workflow (input → generate → review → save)
- Error handling (generation failure, save failure)
- Edit mode (enable, edit, save, cancel)
- Cancel at each stage

### Manual Tests
- Trigger from Commands pane
- Multi-line input with Ctrl+Enter
- Long descriptions (>1000 chars)
- Special characters in description
- Review and edit generated spec
- Cancel at various stages
- Error scenarios (script missing, write permission denied)

## Dependencies

**New:**
- None (uses existing dependencies)

**Modified:**
- `crates/rstn/src/tui/views/worktree.rs` - Add specify state and methods
- `crates/rstn/src/tui/views/mod.rs` - Add new ContentType variants
- `crates/rstn/src/tui/event.rs` - Add specify events
- `crates/rstn/src/tui/app.rs` - Handle specify events and actions

**Integrates with:**
- `.specify/scripts/bash/create-new-feature.sh` - Existing shell script
- spec-kit infrastructure - Feature directory structure

## Success Metrics

**UX improvements:**
- No context switching - user stays in TUI
- Immediate feedback on spec generation
- Ability to review before saving
- Inline editing capability

**Technical:**
- Response time: Input/review transitions <50ms
- Generation time: Same as current (depends on Claude Code)
- Success rate: 100% when shell script succeeds
- Error recovery: Clear messages, ability to retry

## Future Enhancements (Not in 051)

These will be addressed in feature 052:
- Implement spec generation in Rust (eliminate shell script dependency)
- Direct Claude Code CLI integration
- Streaming output during generation
- Multiple spec template options

## Notes

- This feature focuses on UX transformation, not implementation changes
- Still relies on `create-new-feature.sh` - no changes to shell script needed
- Feature 052 will internalize the spec generation logic
- Pattern is consistent with feature 050 (Commit Review drop dialog)
- Follows existing keyboard-first interaction model

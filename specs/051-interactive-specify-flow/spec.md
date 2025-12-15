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
- Single-line input with horizontal scrolling
- Enter key submits, Esc key cancels
- Cursor visible and tracks input position
- Global hotkeys (1-3, y, q, etc.) blocked during input

**FR-2: Spec Generation**
- On submit, show "Generating spec..." status in Content area
- Call Claude Code CLI directly with `/speckit.specify` command
- Stream output in real-time to Output pane
- Parse generated spec from Claude Code response
- On success, transition to Spec Review view
- On error, show error in Content area and stay in Input view
- Minimum description length: 10 characters (validated before generation)

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

**FR-7: Mouse Support** *(Added post-implementation)*
- Click on tabs (Worktree/Settings/Dashboard) to switch views
- Click on panes (Commands/Content/Output) to focus them
- Mouse support works during SpecifyInput mode without breaking input
- Only left-click handled (ignore drag, scroll, other buttons)
- Visual feedback: focused pane shows cyan border

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
Add new enum variant to `ContentType`:
```rust
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,
    SpecifyInput,    // New: Input/Review/Edit all use this single type
}
```

**Note**: Unlike the original spec, implementation uses a single `SpecifyInput` content type for all phases (Input, Generating, Review, Edit), with phase differentiation handled by the `SpecifyState` struct.

### State Management
Add `SpecifyState` struct to `WorktreeView`:
```rust
pub struct SpecifyState {
    // Input phase
    pub input_buffer: String,
    pub input_cursor: usize,
    pub validation_error: Option<String>,

    // Generation phase
    pub is_generating: bool,
    pub generation_error: Option<String>,

    // Review/Edit phase
    pub generated_spec: Option<String>,
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,
    pub edit_mode: bool,
    pub edit_text_input: Option<TextInput>,
}

pub struct WorktreeView {
    // ... existing fields ...
    pub specify_state: SpecifyState,
}
```

**State Machine**: The workflow progresses through phases using a single ContentType:
- **Input Phase**: `SpecifyInput` content type, `is_generating = false`, `generated_spec = None`
- **Generating Phase**: `SpecifyInput` content type, `is_generating = true`
- **Review Phase**: `SpecifyInput` content type, `is_generating = false`, `generated_spec = Some(...)`, `edit_mode = false`
- **Edit Phase**: `SpecifyInput` content type, `generated_spec = Some(...)`, `edit_mode = true`

### Methods
```rust
impl WorktreeView {
    // Start specify input
    pub fn start_specify_input(&mut self);

    // Handle input in specify mode
    pub fn handle_specify_input(&mut self, key: KeyEvent) -> ViewAction;

    // Submit description for generation
    pub fn submit_specify_description(&mut self) -> ViewAction;

    // Handle review mode input
    pub fn handle_specify_review_input(&mut self, key: KeyEvent) -> ViewAction;

    // Toggle edit mode
    pub fn toggle_specify_edit_mode(&mut self);

    // Handle edit mode input
    pub fn handle_specify_edit_input(&mut self, key: KeyEvent) -> ViewAction;

    // Save spec from edit mode
    pub fn save_from_edit(&mut self) -> ViewAction;

    // Cancel edit mode
    pub fn cancel_edit(&mut self);

    // Save spec to file
    pub fn save_specify_spec(&mut self) -> ViewAction;

    // Cancel specify workflow
    pub fn cancel_specify(&mut self);

    // Check if in specify input mode (for global hotkey blocking)
    pub fn is_in_specify_input_mode(&self) -> bool;
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

### Claude Code CLI Integration
**As Implemented**: Direct Claude Code CLI integration (not shell script)
- Call Claude Code CLI with `/speckit.specify` command
- Pass feature description as prompt
- Stream output to TUI in real-time
- Parse generated spec from Claude Code response
- Extract feature number and name from spec file path
- Load spec content for review

**Note**: Original spec planned to use `create-new-feature.sh`, but implementation uses direct Claude Code integration for better control and streaming output.

### Commands Pane
- Add "Specify" to list of available commands
- Position after spec-kit phases, before git commands
- Same styling as other commands

### Keyboard Shortcuts
- In Input mode:
  - Enter: Submit (no Ctrl+Enter, single-line input)
  - Esc: Cancel
  - Standard text editing (left/right arrows, backspace, delete, Home, End)
  - Global hotkeys (1-3, y, q, Y, [, ]) are blocked during input
- In Review mode:
  - Enter: Save
  - e: Edit
  - Esc: Cancel
  - Arrow keys: Scroll content
- In Edit mode:
  - Ctrl+S: Save edited spec
  - Enter: Insert newline (multi-line editor)
  - Esc: Cancel edit (back to review)
  - Full text editing (arrows, Home, End, backspace, delete)

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
- Trigger from Commands pane (keyboard and mouse click)
- Single-line input with horizontal scrolling
- Long descriptions (>1000 chars)
- Special characters in description
- Review and edit generated spec
- Cancel at various stages
- Error scenarios (Claude Code not available, write permission denied)
- Mouse interaction:
  - Click tabs to switch views during specify workflow
  - Click panes to focus during specify workflow
  - Verify mouse doesn't break input mode
- Global hotkey blocking:
  - Verify 1-3 keys insert digits (don't switch views)
  - Verify y, q, Y keys insert letters (don't trigger shortcuts)
  - Verify Esc exits input mode properly

## Dependencies

**New:**
- None (uses existing dependencies)

**Modified:**
- `crates/rstn/src/tui/views/worktree.rs` - Add specify state and methods
- `crates/rstn/src/tui/views/mod.rs` - Add new ContentType variants
- `crates/rstn/src/tui/event.rs` - Add specify events
- `crates/rstn/src/tui/app.rs` - Handle specify events and actions

**Integrates with:**
- Claude Code CLI - Direct integration for spec generation
- spec-kit infrastructure - Feature directory structure

### Mouse Support Implementation *(Post-051)*

**Architecture**:
- Store layout `Rect`s during render for click detection
- `App` struct stores `tab_bar_rect` for view switching
- `WorktreeView` stores pane rects for focus switching
- `point_in_rect()` helper function checks if click coordinates fall within rect bounds

**Files Modified**:
- `app.rs`: Added `handle_mouse_event()`, `tab_bar_rect` field, mouse event routing
- `worktree.rs`: Added `handle_mouse()`, pane rect fields, `is_in_specify_input_mode()`
- `mod.rs`: Changed View trait `render()` from `&self` to `&mut self`

**Behavior**:
- Click tab bar → switch views (Worktree/Settings/Dashboard)
- Click pane → focus that pane (Commands/Content/Output)
- During SpecifyInput → keyboard hotkeys blocked, mouse clicks still work
- Only left-click button handled (ignore drag, scroll, right-click)

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

**Implementation vs Original Spec**:
- ✅ Achieved: UX transformation - no context switching, inline workflow
- ✅ Achieved: Review and edit before saving
- ✅ Achieved: Consistent keyboard-first interaction
- ❌ Changed: Uses direct Claude Code CLI instead of `create-new-feature.sh` shell script
- ❌ Changed: Single-line input instead of multi-line (simpler UX)
- ❌ Changed: Single `SpecifyInput` ContentType instead of separate Input/Review types
- ➕ Added: Global hotkey blocking during input (1-3, y, q, Y don't trigger shortcuts)
- ➕ Added: Mouse click support for tabs and panes (post-implementation enhancement)
- ➕ Added: Cursor rendering and visibility fixes

**Future Work** (Feature 052):
- Internalize spec generation (eliminate dependency on external Claude Code process)
- Multiple spec templates
- Streaming generation with progress updates

**Pattern Notes**:
- Inline content transformation (not popup dialog like commit review)
- State machine pattern with single ContentType
- Follows existing keyboard-first + mouse interaction model

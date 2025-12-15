# Data Model: Interactive Specify Flow

**Feature**: 051-interactive-specify-flow
**Date**: 2025-12-15

## Overview

This document defines the state structures, types, and data flows for the Interactive Specify Flow feature. The feature adds temporary workflow state to the existing TUI architecture without persisting data beyond the spec file creation.

## Core Types

### ContentType Enum (Extend Existing)

```rust
// File: crates/rstn/src/tui/views/mod.rs
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,

    // New variants for feature 051
    SpecifyInput,    // User is entering feature description
    SpecifyReview,   // User is reviewing generated spec
}
```

**Purpose**: Discriminates between different Content area modes to control rendering and input handling.

**States**:
- `SpecifyInput`: Content area shows text input dialog for feature description
- `SpecifyReview`: Content area shows generated spec with save/edit/cancel options

### SpecifyState Structure (New)

```rust
// File: crates/rstn/src/tui/views/worktree.rs

#[derive(Debug, Clone, Default)]
pub struct SpecifyState {
    // Input phase
    pub input_buffer: String,
    pub input_cursor: usize,

    // Generation phase
    pub is_generating: bool,
    pub generation_error: Option<String>,

    // Review/Edit phase
    pub generated_spec: Option<String>,
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,
    pub edit_mode: bool,
    pub edit_cursor: usize,
    pub edit_scroll_offset: usize,

    // Validation
    pub validation_error: Option<String>,
}

impl SpecifyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        !self.input_buffer.is_empty()
            || self.is_generating
            || self.generated_spec.is_some()
    }

    pub fn validate_input(&self) -> Result<(), String> {
        let trimmed = self.input_buffer.trim();
        if trimmed.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        Ok(())
    }
}
```

**Purpose**: Encapsulates all state for the specify workflow, keeping WorktreeView clean.

**Lifecycle**:
1. Created when user triggers "Specify" action
2. Populated during input phase
3. Updated during generation
4. Modified during review/edit
5. Cleared after save or cancel

### WorktreeView Integration (Extend Existing)

```rust
// File: crates/rstn/src/tui/views/worktree.rs

pub struct WorktreeView {
    // ... existing fields ...

    // Feature 051: Specify workflow state
    pub specify_state: SpecifyState,
}

impl WorktreeView {
    // Entry points
    pub fn start_specify_input(&mut self) {
        self.specify_state = SpecifyState::new();
        self.content_type = ContentType::SpecifyInput;
        self.active_pane = ActivePane::Content; // Auto-focus Content area
    }

    pub fn cancel_specify(&mut self) {
        self.specify_state.clear();
        self.content_type = ContentType::Spec; // Return to normal view
        self.active_pane = ActivePane::Commands; // Return focus
    }

    // Workflow methods (implementations in separate sections below)
    pub fn handle_specify_input(&mut self, key: KeyEvent) -> ViewAction;
    pub fn submit_specify_description(&mut self) -> ViewAction;
    pub fn load_generated_spec(&mut self, spec: String, number: String, name: String);
    pub fn toggle_specify_edit_mode(&mut self);
    pub fn save_specify_spec(&mut self) -> ViewAction;
}
```

## Events (Extend Existing)

```rust
// File: crates/rstn/src/tui/event.rs

#[derive(Debug, Clone)]
pub enum Event {
    // ... existing events ...

    // Feature 051: Specify workflow events
    SpecifyGenerationStarted,
    SpecifyGenerationCompleted {
        spec: String,
        number: String,
        name: String,
    },
    SpecifyGenerationFailed {
        error: String,
    },
    SpecifySaved {
        path: String,
    },
}
```

**Event Flow**:
```
User presses Enter in input mode
  ↓
ViewAction::GenerateSpec { description }
  ↓
App spawns async task → shell script execution
  ↓
Event::SpecifyGenerationStarted
  ↓
WorktreeView updates state (is_generating = true)
  ↓
Shell script completes
  ↓
Event::SpecifyGenerationCompleted { spec, number, name }
  ↓
WorktreeView loads spec, switches to Review mode
  ↓
User presses Enter to save
  ↓
ViewAction::SaveSpec { content, number, name }
  ↓
App writes file
  ↓
Event::SpecifySaved { path }
  ↓
WorktreeView clears state, returns to normal view
```

## Actions (Extend Existing)

```rust
// File: crates/rstn/src/tui/actions.rs (or wherever ViewAction is defined)

#[derive(Debug, Clone)]
pub enum ViewAction {
    // ... existing actions ...

    // Feature 051: Specify workflow actions
    GenerateSpec {
        description: String,
    },
    SaveSpec {
        content: String,
        number: String,
        name: String,
    },
}
```

**Action Triggers**:
- `GenerateSpec`: Triggered when user presses Enter in SpecifyInput mode (after validation)
- `SaveSpec`: Triggered when user presses Enter in SpecifyReview mode, or Ctrl+S in edit mode

## State Transitions

### Workflow State Machine

```
┌─────────────┐
│   Normal    │
│    View     │
└──────┬──────┘
       │ trigger "Specify"
       │ start_specify_input()
       ↓
┌─────────────┐
│  Specify    │◄─── validation error
│   Input     │     stay in input mode
└──────┬──────┘
       │ Enter key
       │ submit_specify_description()
       ↓
┌─────────────┐
│ Generating  │
│   (async)   │
└──────┬──────┘
       │ on success        on error
       ↓                   ↓
┌─────────────┐     ┌─────────────┐
│  Specify    │     │   Input     │
│   Review    │     │ (w/ error)  │
└──────┬──────┘     └─────────────┘
       │
       ├─ 'e' key
       │  toggle_specify_edit_mode()
       ↓
┌─────────────┐
│   Edit      │
│   Mode      │◄─── Esc (cancel edits)
└──────┬──────┘     return to Review
       │
       │ Ctrl+S or Enter
       │ save_specify_spec()
       ↓
┌─────────────┐
│    Saved    │
│  (cleanup)  │
└──────┬──────┘
       │
       ↓
┌─────────────┐
│   Normal    │
│    View     │
└─────────────┘
```

### Mode-Specific Key Bindings

**SpecifyInput Mode**:
- `char` → append to input_buffer
- `Backspace` → remove last char
- `Delete` → remove char at cursor
- `Left/Right` → move cursor
- `Home/End` → move to start/end
- `Ctrl+Enter` → insert newline
- `Enter` → validate and submit
- `Esc` → cancel, return to normal

**SpecifyReview Mode**:
- `Enter` → save spec
- `e` → toggle edit mode
- `Esc` → cancel, discard spec
- `n/p` → scroll (if needed)

**SpecifyEdit Mode**:
- `char` → insert at cursor
- `Backspace` → delete before cursor
- `Delete` → delete at cursor
- `Left/Right/Up/Down` → move cursor
- `Home/End` → move to line start/end
- `Ctrl+S` → save spec
- `Enter` → save spec (alternative)
- `Esc` → cancel edit, return to review

## Data Validation

### Input Validation

```rust
impl SpecifyState {
    pub fn validate_input(&self) -> Result<(), String> {
        let trimmed = self.input_buffer.trim();

        if trimmed.is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        if trimmed.len() > 10_000 {
            return Err("Description too long (max 10,000 characters)".to_string());
        }

        Ok(())
    }
}
```

### Generation Validation

```rust
// In app.rs, after shell script execution

fn validate_generation_output(
    stdout: String,
    stderr: String,
    exit_code: i32,
) -> Result<(String, String, String), String> {
    if exit_code != 0 {
        return Err(format!("Generation failed: {}", stderr));
    }

    // Parse output to extract feature number and name
    // Read generated spec file
    // Return (spec_content, number, name)

    Ok((spec_content, number, name))
}
```

### Save Validation

```rust
// In app.rs, during save

fn validate_save(
    content: &str,
    number: &str,
    name: &str,
) -> Result<PathBuf, String> {
    if content.trim().is_empty() {
        return Err("Cannot save empty spec".to_string());
    }

    let path = PathBuf::from(format!("specs/{}-{}/spec.md", number, name));

    if path.exists() {
        return Err(format!("Spec already exists: {}", path.display()));
    }

    Ok(path)
}
```

## Memory Management

### State Cleanup

The specify workflow state is temporary and should be cleaned up:

**Cleanup Triggers**:
1. After successful save → `cancel_specify()`
2. User presses Esc → `cancel_specify()`
3. Navigation away from worktree view → automatic cleanup via view switch

**Memory Footprint**:
- Input buffer: ~1KB typical, 10KB max
- Generated spec: ~10-50KB typical
- Total: <100KB during workflow
- Zero memory after cleanup

### Resource Lifecycle

```
User triggers "Specify"
  ↓
Allocate SpecifyState (~1KB)
  ↓
User types description (~1-10KB)
  ↓
Generate spec (spawn async task, does not block)
  ↓
Load spec content (10-50KB)
  ↓
User reviews/edits (temporary modifications in memory)
  ↓
Save to disk (write once)
  ↓
Clean up SpecifyState (drop all buffers)
  ↓
Return to baseline memory
```

## Error Handling

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum SpecifyError {
    EmptyInput,
    InputTooLong { max: usize, actual: usize },
    GenerationTimeout,
    GenerationFailed { error: String },
    ScriptNotFound { path: String },
    SaveFailed { path: String, error: String },
}

impl fmt::Display for SpecifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Description cannot be empty"),
            Self::InputTooLong { max, actual } => {
                write!(f, "Description too long ({} chars, max {})", actual, max)
            }
            Self::GenerationTimeout => {
                write!(f, "Generation timed out after 60 seconds. Press Enter to retry or Esc to cancel.")
            }
            Self::GenerationFailed { error } => {
                write!(f, "Generation failed: {}", error)
            }
            Self::ScriptNotFound { path } => {
                write!(f, "Script not found: {}", path)
            }
            Self::SaveFailed { path, error } => {
                write!(f, "Failed to save {}: {}", path, error)
            }
        }
    }
}
```

### Error Recovery

- **Empty input**: Show error, stay in input mode, allow retry
- **Generation failure**: Show error in Output area, stay in input mode, allow retry
- **Timeout**: Show timeout message, offer retry or cancel
- **Save failure**: Show error, stay in review/edit mode, offer retry or copy to clipboard

## Integration Points

### With Feature 050 (Commit Review)

Feature 051 follows the same pattern as feature 050:
- Similar state structure (input → async operation → review → action)
- Consistent key bindings (Enter, Esc, edit mode)
- Parallel ContentType enum variants
- Same event/action architecture

### With Shell Script

```rust
// In app.rs

async fn execute_spec_generation(description: String) -> Result<GenerationResult, String> {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    let script_path = ".specify/scripts/bash/create-new-feature.sh";

    let child = Command::new(script_path)
        .arg("--json")
        .arg(&description)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn script: {}", e))?;

    let output = timeout(Duration::from_secs(60), child.wait_with_output())
        .await
        .map_err(|_| "Generation timed out after 60 seconds".to_string())?
        .map_err(|e| format!("Script execution failed: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Parse JSON output to get paths
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse script output: {}", e))?;

    let spec_file = json["SPEC_FILE"]
        .as_str()
        .ok_or("Missing SPEC_FILE in output")?;
    let feature_num = json["FEATURE_NUM"]
        .as_str()
        .ok_or("Missing FEATURE_NUM in output")?;
    let branch_name = json["BRANCH_NAME"]
        .as_str()
        .ok_or("Missing BRANCH_NAME in output")?;

    // Extract feature name from branch (e.g., "051-feature-name" -> "feature-name")
    let feature_name = branch_name
        .split('-')
        .skip(1)
        .collect::<Vec<_>>()
        .join("-");

    // Read generated spec
    let spec_content = tokio::fs::read_to_string(spec_file)
        .await
        .map_err(|e| format!("Failed to read generated spec: {}", e))?;

    Ok(GenerationResult {
        spec: spec_content,
        number: feature_num.to_string(),
        name: feature_name,
    })
}

struct GenerationResult {
    spec: String,
    number: String,
    name: String,
}
```

## Summary

This data model provides:
- ✅ Clean state encapsulation in `SpecifyState`
- ✅ Clear event/action flow
- ✅ Comprehensive error handling
- ✅ Minimal memory footprint
- ✅ Consistent with existing architecture (feature 050)
- ✅ Type-safe transitions between modes

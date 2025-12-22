//! Worktree view module
//!
//! This module is organized into submodules:
//! - `types`: Core type definitions (Command, ContentType, etc.)
//! - `task`: Task parsing and management
//! - `input`: Inline input widget
//! - `specify`: Specify state management
//! - `view`: Main WorktreeView implementation

mod input;
mod specify;
mod task;
mod types;
mod view;

// Re-export all public types
pub use input::InlineInput;
pub use specify::SpecifyState;
pub use task::{ParsedTask, TaskListState};
pub use types::{Command, ContentType, FeatureInfo, GitCommand, WorktreeFocus};
pub use view::WorktreeView;

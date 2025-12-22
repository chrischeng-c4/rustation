//! Specify state management for the Worktree view
//!
//! This module manages the state for the SDD workflow phases:
//! - Feature 051: Specify/Plan/Tasks generation with inline input
//! - Feature 055: Structured task list with interactive editing
//! - Feature 056: Implement mode for task execution

use super::{ParsedTask, TaskListState};
use crate::tui::views::SpecPhase;
use crate::tui::widgets::TextInput;

/// State for the Specify/Plan/Tasks workflow (Feature 051)
/// Represents the current SDD phase and any generated content
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpecifyState {
    // Phase tracking (Feature 053-058)
    /// Current SDD phase being executed
    pub current_phase: SpecPhase,

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
    pub edit_text_input: Option<TextInput>, // Feature 051: Multi-line editing widget (User Story 3)

    // Task list mode (Feature 055)
    /// Structured task list for reordering (Tasks phase only)
    pub task_list_state: Option<TaskListState>,

    // Implement mode (Feature 056)
    /// Index of task currently being executed
    pub executing_task_index: Option<usize>,
    /// Output from task execution
    pub execution_output: String,
    /// Auto-advance to next incomplete task after completion
    pub auto_advance: bool,

    // Validation
    pub validation_error: Option<String>,
}

impl Default for SpecifyState {
    fn default() -> Self {
        Self {
            current_phase: SpecPhase::Specify,
            input_buffer: String::new(),
            input_cursor: 0,
            is_generating: false,
            generation_error: None,
            generated_spec: None,
            feature_number: None,
            feature_name: None,
            edit_mode: false,
            edit_text_input: None,
            task_list_state: None,
            executing_task_index: None,
            execution_output: String::new(),
            auto_advance: false,
            validation_error: None,
        }
    }
}

impl SpecifyState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new state for a specific phase
    pub fn for_phase(phase: SpecPhase) -> Self {
        Self {
            current_phase: phase,
            ..Self::default()
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        !self.input_buffer.is_empty() || self.is_generating || self.generated_spec.is_some()
    }

    /// Get the target filename for the current phase
    pub fn target_filename(&self) -> &'static str {
        match self.current_phase {
            SpecPhase::Specify => "spec.md",
            SpecPhase::Clarify => "spec.md", // Clarify updates the spec
            SpecPhase::Plan => "plan.md",
            SpecPhase::Tasks => "tasks.md",
            SpecPhase::Analyze => "analysis.md",
            SpecPhase::Checklist => "checklist.md",
            SpecPhase::Implement => "tasks.md", // Updates task status
            SpecPhase::Review => "review.md",
        }
    }

    /// Get input prompt for the current phase
    pub fn input_prompt(&self) -> &'static str {
        match self.current_phase {
            SpecPhase::Specify => "Enter feature description:",
            SpecPhase::Clarify => "Review and confirm clarifications:",
            SpecPhase::Plan => "Enter implementation notes (optional):",
            SpecPhase::Tasks => "Enter task generation notes (optional):",
            SpecPhase::Analyze => "Enter analysis scope (optional):",
            SpecPhase::Checklist => "Enter checklist focus (optional):",
            SpecPhase::Implement => "Select task to implement:",
            SpecPhase::Review => "Enter review notes (optional):",
        }
    }

    pub fn validate_input(&self) -> Result<(), String> {
        let trimmed = self.input_buffer.trim();

        // Specify requires meaningful input
        if self.current_phase == SpecPhase::Specify {
            if trimmed.is_empty() {
                return Err("Description cannot be empty".to_string());
            }
            if trimmed.len() < 3 {
                return Err("Description must be at least 3 characters".to_string());
            }
        }
        // Other phases allow empty input (use existing spec/plan context)
        Ok(())
    }

    /// Set generated content and initialize task list if in Tasks phase (Feature 055)
    pub fn set_generated_content(&mut self, content: String, number: String, name: String) {
        self.generated_spec = Some(content.clone());
        self.feature_number = Some(number);
        self.feature_name = Some(name);
        self.is_generating = false;

        // For Tasks phase, parse into structured task list
        if self.current_phase == SpecPhase::Tasks {
            let task_list = TaskListState::from_markdown(&content);
            if !task_list.is_empty() {
                self.task_list_state = Some(task_list);
            }
        }
    }

    /// Check if in task list mode (Feature 055)
    pub fn is_task_list_mode(&self) -> bool {
        self.task_list_state
            .as_ref()
            .is_some_and(|t| t.list_mode && !t.is_empty())
    }

    /// Get current content (from task list if in list mode, otherwise raw)
    pub fn get_current_content(&self) -> Option<String> {
        if self.is_task_list_mode() {
            self.task_list_state.as_ref().map(|t| t.to_markdown())
        } else {
            self.generated_spec.clone()
        }
    }

    // ===== Feature 056: Implement Mode Methods =====

    /// Check if in implement mode (executing tasks)
    pub fn is_implement_mode(&self) -> bool {
        self.current_phase == SpecPhase::Implement && self.task_list_state.is_some()
    }

    /// Check if a task is currently executing
    pub fn is_executing(&self) -> bool {
        self.executing_task_index.is_some()
    }

    /// Load tasks from existing tasks.md file
    pub fn load_tasks_from_file(&mut self, content: &str, number: String, name: String) {
        self.feature_number = Some(number);
        self.feature_name = Some(name);
        self.generated_spec = Some(content.to_string());

        let task_list = TaskListState::from_markdown(content);
        if !task_list.is_empty() {
            self.task_list_state = Some(task_list);
        }
    }

    /// Toggle completion status of selected task
    pub fn toggle_task_completion(&mut self) {
        if let Some(ref mut task_list) = self.task_list_state {
            if let Some(task) = task_list.tasks.get_mut(task_list.selected) {
                task.completed = !task.completed;
            }
        }
    }

    /// Get the currently selected task (if any)
    pub fn get_selected_task(&self) -> Option<&ParsedTask> {
        self.task_list_state
            .as_ref()
            .and_then(|tl| tl.tasks.get(tl.selected))
    }

    /// Get count of completed vs total tasks
    pub fn get_task_progress(&self) -> (usize, usize) {
        self.task_list_state
            .as_ref()
            .map(|tl| {
                let completed = tl.tasks.iter().filter(|t| t.completed).count();
                (completed, tl.len())
            })
            .unwrap_or((0, 0))
    }

    /// Advance selection to next incomplete task
    pub fn advance_to_next_incomplete(&mut self) {
        if let Some(ref mut task_list) = self.task_list_state {
            // Find next incomplete task starting from current + 1
            for i in (task_list.selected + 1)..task_list.len() {
                if !task_list.tasks[i].completed {
                    task_list.selected = i;
                    return;
                }
            }
            // Wrap around to beginning
            for i in 0..task_list.selected {
                if !task_list.tasks[i].completed {
                    task_list.selected = i;
                    return;
                }
            }
        }
    }

    /// Mark a task as complete by its ID (Feature 063)
    pub fn complete_task_by_id(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(ref mut task_list) = self.task_list_state {
            task_list.complete_by_id(task_id)?;
            Ok(())
        } else {
            Err("No task list loaded".to_string())
        }
    }
}

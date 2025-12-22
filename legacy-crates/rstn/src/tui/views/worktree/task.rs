//! Task parsing and management for tasks.md
//!
//! This module provides structures for parsing and manipulating tasks
//! from the SDD tasks.md format.

/// Parsed task from tasks.md (Feature 055)
///
/// Represents a single task parsed from markdown format:
/// `- [ ] T001 [P] [US1] Description in src/path/file.rs`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedTask {
    /// Task ID (e.g., "T001")
    pub id: String,
    /// Whether task can run in parallel [P]
    pub is_parallel: bool,
    /// User story label (e.g., "US1")
    pub user_story: Option<String>,
    /// Task description
    pub description: String,
    /// File path if mentioned
    pub file_path: Option<String>,
    /// Whether task is completed [X] vs [ ]
    pub completed: bool,
    /// Original raw line (for preserving formatting)
    pub raw_line: String,
}

impl ParsedTask {
    /// Parse a task line from markdown format
    ///
    /// Expected format: `- [ ] T001 [P] [US1] Description in src/path/file.rs`
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();

        // Must start with checkbox
        if !trimmed.starts_with("- [") {
            return None;
        }

        // Check completion status
        let completed = trimmed.starts_with("- [X]") || trimmed.starts_with("- [x]");

        // Extract content after checkbox
        let content = if completed {
            trimmed
                .strip_prefix("- [X] ")
                .or_else(|| trimmed.strip_prefix("- [x] "))
        } else {
            trimmed.strip_prefix("- [ ] ")
        }?;

        // Parse task ID (T001, T002, etc.)
        let mut parts = content.split_whitespace();
        let id = parts.next()?;
        if !id.starts_with('T') || id.len() < 2 {
            return None;
        }

        // Collect remaining parts and parse markers
        let remaining: Vec<&str> = parts.collect();
        let mut is_parallel = false;
        let mut user_story = None;
        let mut description_parts = Vec::new();

        for part in remaining {
            if part == "[P]" {
                is_parallel = true;
            } else if part.starts_with("[US") && part.ends_with(']') {
                user_story = Some(part[1..part.len() - 1].to_string());
            } else {
                description_parts.push(part);
            }
        }

        let description = description_parts.join(" ");

        // Try to extract file path (looks for " in " followed by path)
        let file_path = if let Some(idx) = description.find(" in ") {
            let path_part = &description[idx + 4..];
            if path_part.contains('/') || path_part.ends_with(".rs") {
                Some(path_part.to_string())
            } else {
                None
            }
        } else {
            None
        };

        Some(Self {
            id: id.to_string(),
            is_parallel,
            user_story,
            description,
            file_path,
            completed,
            raw_line: line.to_string(),
        })
    }

    /// Convert back to markdown format
    pub fn to_markdown(&self) -> String {
        let checkbox = if self.completed { "- [X]" } else { "- [ ]" };
        let parallel = if self.is_parallel { " [P]" } else { "" };
        let us = self
            .user_story
            .as_ref()
            .map(|s| format!(" [{}]", s))
            .unwrap_or_default();

        format!(
            "{} {}{}{} {}",
            checkbox, self.id, parallel, us, self.description
        )
    }
}

/// State for interactive task list editing (Feature 055)
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskListState {
    /// Parsed tasks from generated content
    pub tasks: Vec<ParsedTask>,
    /// Currently selected task index
    pub selected: usize,
    /// Whether we're in structured list mode (vs raw text mode)
    pub list_mode: bool,
}

impl TaskListState {
    /// Parse tasks from markdown content
    pub fn from_markdown(content: &str) -> Self {
        let tasks: Vec<ParsedTask> = content.lines().filter_map(ParsedTask::parse).collect();

        Self {
            tasks,
            selected: 0,
            list_mode: true,
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected < self.tasks.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selected task up in the list
    pub fn reorder_up(&mut self) {
        if self.selected > 0 {
            self.tasks.swap(self.selected, self.selected - 1);
            self.selected -= 1;
        }
    }

    /// Move selected task down in the list
    pub fn reorder_down(&mut self) {
        if self.selected < self.tasks.len().saturating_sub(1) {
            self.tasks.swap(self.selected, self.selected + 1);
            self.selected += 1;
        }
    }

    /// Convert all tasks back to markdown
    pub fn to_markdown(&self) -> String {
        self.tasks
            .iter()
            .map(|t| t.to_markdown())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if there are any tasks
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Get number of tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Mark a task as complete by its ID (e.g., "T001")
    pub fn complete_by_id(&mut self, task_id: &str) -> Result<(), String> {
        // Find task by ID
        let task_index = self
            .tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        // Mark as complete
        self.tasks[task_index].completed = true;
        Ok(())
    }
}

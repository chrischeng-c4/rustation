//! Tasks view - Justfile command runner
//!
//! Based on desktop/src/renderer/src/features/tasks/TasksPage.tsx

use gpui::*;
use rstn_core::justfile::JustCommand;
use rstn_ui::{EmptyState, MaterialTheme, PageHeader, Themed};

/// Task execution state
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Idle,
    Running,
    Success,
    Failed,
}

/// Task card data
#[derive(Debug, Clone)]
pub struct TaskCard {
    pub command: JustCommand,
    pub state: TaskState,
    pub output: Vec<String>,
}

impl TaskCard {
    pub fn new(command: JustCommand) -> Self {
        Self {
            command,
            state: TaskState::Idle,
            output: Vec::new(),
        }
    }

    /// Render a single task card
    pub fn render(&self, theme: &MaterialTheme, _window: &mut Window, _cx: &mut App) -> Div {
        let state_color = match self.state {
            TaskState::Idle => theme.text.secondary,
            TaskState::Running => theme.primary.main,
            TaskState::Success => rgb(0x4CAF50), // Green
            TaskState::Failed => rgb(0xF44336),  // Red
        };

        div()
            .card(theme)
            .mb(theme.spacing(1.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb(theme.spacing(1.0))
                    .child(
                        // Command name
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.command.name.clone()),
                    )
                    .child(
                        // State indicator
                        div()
                            .px(theme.spacing(1.0))
                            .py(theme.spacing(0.5))
                            .bg(state_color)
                            .rounded(theme.shape.border_radius_xs)
                            .text_xs()
                            .child(match self.state {
                                TaskState::Idle => "Ready",
                                TaskState::Running => "Running...",
                                TaskState::Success => "Success",
                                TaskState::Failed => "Failed",
                            }),
                    ),
            )
            .children(
                // Description
                self.command.description.as_ref().map(|desc| {
                    div()
                        .text_sm()
                        .text_color(theme.text.secondary)
                        .mb(theme.spacing(1.0))
                        .child(desc.clone())
                })
            )
            .child(
                // Recipe preview (first line)
                div()
                    .text_xs()
                    .font_family("monospace")
                    .text_color(theme.text.secondary)
                    .child(
                        self.command
                            .recipe
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    ),
            )
    }
}

/// Log panel component for displaying command output
pub struct LogPanel {
    title: SharedString,
    logs: Vec<String>,
    theme: MaterialTheme,
}

impl LogPanel {
    pub fn new(title: impl Into<SharedString>, logs: Vec<String>, theme: MaterialTheme) -> Self {
        Self {
            title: title.into(),
            logs,
            theme,
        }
    }

    pub fn render(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .h_full()
            .border_1()
            .border_color(self.theme.border.divider)
            .rounded(self.theme.shape.border_radius_sm)
            .bg(self.theme.background.paper)
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(self.theme.spacing(2.0))
                    .py(self.theme.spacing(1.5))
                    .border_b_1()
                    .border_color(self.theme.border.divider)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    ),
            )
            .child(
                // Log content
                div()
                    .flex_1()
                    .p(self.theme.spacing(2.0))
                    .overflow_hidden()
                    .font_family("monospace")
                    .text_xs()
                    .children(
                        if self.logs.is_empty() {
                            vec![div()
                                .text_color(self.theme.text.disabled)
                                .child("No output yet...")]
                        } else {
                            self.logs
                                .iter()
                                .map(|line| {
                                    div()
                                        .text_color(self.theme.text.primary)
                                        .child(line.clone())
                                })
                                .collect()
                        },
                    ),
            )
    }
}

/// Tasks page view
///
/// Layout based on OLD_UI_ANALYSIS.md:
/// ```
/// ┌─────────────────────────────────────┐
/// │ PageHeader (Title + Refresh Button)│
/// ├─────────────────┬───────────────────┤
/// │ Commands List   │ Log Panel         │
/// │ (50% width)     │ (50% width)       │
/// │                 │                   │
/// │ - TaskCard 1    │ Output Lines      │
/// │ - TaskCard 2    │ ...               │
/// │ - TaskCard 3    │                   │
/// └─────────────────┴───────────────────┘
/// ```
pub struct TasksView {
    tasks: Vec<TaskCard>,
    active_task_index: Option<usize>,
    theme: MaterialTheme,
}

impl TasksView {
    pub fn new(commands: Vec<JustCommand>, theme: MaterialTheme) -> Self {
        Self {
            tasks: commands.into_iter().map(TaskCard::new).collect(),
            active_task_index: None,
            theme,
        }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        let page_header = PageHeader::new(
            "Tasks",
            Some("Run justfile commands"),
            self.theme.clone(),
        );

        // Get active task output
        let output = self
            .active_task_index
            .and_then(|idx| self.tasks.get(idx))
            .map(|task| task.output.clone())
            .unwrap_or_default();

        let log_panel = LogPanel::new("Output", output, self.theme.clone());

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                page_header.render(Some(
                    div()
                        .secondary_button(&self.theme)
                        .child("Refresh"),
                )),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .gap(self.theme.spacing(2.0))
                    .child(
                        // Commands list (left panel)
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_hidden()
                            .children(if self.tasks.is_empty() {
                                vec![EmptyState::new(
                                    "📋",
                                    "No Commands",
                                    "No justfile found in this project",
                                    self.theme.clone(),
                                )
                                .render(Some(
                                    div()
                                        .primary_button(&self.theme)
                                        .child("Scan Again"),
                                ))]
                            } else {
                                self.tasks
                                    .iter()
                                    .map(|task| task.render(&self.theme, window, cx))
                                    .collect()
                            }),
                    )
                    .child(
                        // Log panel (right panel)
                        div().flex().flex_col().flex_1().child(log_panel.render()),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_card_creation() {
        let command = JustCommand {
            name: "test".to_string(),
            description: Some("Run tests".to_string()),
            recipe: "cargo test".to_string(),
        };

        let card = TaskCard::new(command);
        assert_eq!(card.state, TaskState::Idle);
        assert_eq!(card.output.len(), 0);
        assert_eq!(card.command.name, "test");
    }

    #[test]
    fn test_tasks_view_creation() {
        let commands = vec![JustCommand {
            name: "build".to_string(),
            description: Some("Build project".to_string()),
            recipe: "cargo build".to_string(),
        }];

        let theme = MaterialTheme::dark();
        let view = TasksView::new(commands, theme);
        assert_eq!(view.tasks.len(), 1);
        assert_eq!(view.active_task_index, None);
    }

    #[test]
    fn test_log_panel_empty() {
        let theme = MaterialTheme::dark();
        let panel = LogPanel::new("Output", vec![], theme);
        assert_eq!(panel.logs.len(), 0);
    }
}

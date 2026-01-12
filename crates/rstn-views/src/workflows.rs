//! Workflows view - Constitution and Change Management
//!
//! Provides UI for:
//! - Constitution: Coding rules management
//! - Change Management: OpenSpec proposals and reviews
//! - Review Gate: Human approval workflow
//! - Context Engine: AI context aggregation

use gpui::*;
use rstn_ui::MaterialTheme;

/// Workflow panel type
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowPanel {
    Constitution,
    ChangeManagement,
    ReviewGate,
    ContextEngine,
}

impl WorkflowPanel {
    pub fn label(&self) -> &'static str {
        match self {
            WorkflowPanel::Constitution => "Constitution",
            WorkflowPanel::ChangeManagement => "Changes",
            WorkflowPanel::ReviewGate => "Reviews",
            WorkflowPanel::ContextEngine => "Context",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            WorkflowPanel::Constitution => "📜",
            WorkflowPanel::ChangeManagement => "🔄",
            WorkflowPanel::ReviewGate => "✓",
            WorkflowPanel::ContextEngine => "🧠",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WorkflowPanel::Constitution => "Coding rules and project guidelines",
            WorkflowPanel::ChangeManagement => "OpenSpec proposals and implementation tracking",
            WorkflowPanel::ReviewGate => "Human approval workflow for critical changes",
            WorkflowPanel::ContextEngine => "AI context aggregation and injection",
        }
    }
}

/// Constitution rule item
#[derive(Debug, Clone)]
pub struct ConstitutionRule {
    pub name: String,
    pub enabled: bool,
    pub description: String,
}

impl ConstitutionRule {
    pub fn new(name: impl Into<String>, enabled: bool, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled,
            description: description.into(),
        }
    }

    pub fn render(&self, theme: &MaterialTheme, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .p(theme.spacing(1.5))
            .mb(theme.spacing(1.0))
            .bg(theme.background.paper)
            .border_1()
            .border_color(theme.border.divider)
            .rounded(theme.shape.border_radius)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .mb(theme.spacing(0.5))
                            .child(self.name.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text.secondary)
                            .child(self.description.clone()),
                    ),
            )
            .child(
                // Toggle indicator
                div()
                    .px(theme.spacing(1.0))
                    .py(theme.spacing(0.5))
                    .bg(if self.enabled {
                        rgb(0x4CAF50) // Green
                    } else {
                        theme.text.disabled
                    })
                    .rounded(theme.shape.border_radius_xs)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xFFFFFF))
                    .child(if self.enabled { "ON" } else { "OFF" }),
            )
    }
}

/// Change/proposal status
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    Draft,
    Proposed,
    Approved,
    Implementing,
    Complete,
}

impl ChangeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ChangeStatus::Draft => "Draft",
            ChangeStatus::Proposed => "Proposed",
            ChangeStatus::Approved => "Approved",
            ChangeStatus::Implementing => "In Progress",
            ChangeStatus::Complete => "Complete",
        }
    }

    pub fn color(&self, theme: &MaterialTheme) -> Rgba {
        match self {
            ChangeStatus::Draft => theme.text.disabled,
            ChangeStatus::Proposed => rgb(0x2196F3), // Blue
            ChangeStatus::Approved => rgb(0x4CAF50), // Green
            ChangeStatus::Implementing => rgb(0xFF9800), // Amber
            ChangeStatus::Complete => theme.secondary.main,
        }
    }
}

/// Change item
#[derive(Debug, Clone)]
pub struct ChangeItem {
    pub title: String,
    pub status: ChangeStatus,
    pub description: String,
}

impl ChangeItem {
    pub fn new(title: impl Into<String>, status: ChangeStatus, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status,
            description: description.into(),
        }
    }

    pub fn render(&self, theme: &MaterialTheme, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .p(theme.spacing(1.5))
            .mb(theme.spacing(1.0))
            .bg(theme.background.paper)
            .border_1()
            .border_color(theme.border.divider)
            .rounded(theme.shape.border_radius)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb(theme.spacing(0.5))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .px(theme.spacing(1.0))
                            .py(theme.spacing(0.5))
                            .bg(self.status.color(theme))
                            .rounded(theme.shape.border_radius_xs)
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xFFFFFF))
                            .child(self.status.label()),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text.secondary)
                    .child(self.description.clone()),
            )
    }
}

/// Main Workflows view
pub struct WorkflowsView {
    pub active_panel: WorkflowPanel,
    pub constitution_rules: Vec<ConstitutionRule>,
    pub changes: Vec<ChangeItem>,
    pub context_files: Vec<String>,
    pub theme: MaterialTheme,
}

impl WorkflowsView {
    pub fn new(
        constitution_rules: Vec<ConstitutionRule>,
        changes: Vec<ChangeItem>,
        context_files: Vec<String>,
        theme: MaterialTheme,
    ) -> Self {
        Self {
            active_panel: WorkflowPanel::Constitution,
            constitution_rules,
            changes,
            context_files,
            theme,
        }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        let panels = vec![
            WorkflowPanel::Constitution,
            WorkflowPanel::ChangeManagement,
            WorkflowPanel::ReviewGate,
            WorkflowPanel::ContextEngine,
        ];

        div()
            .flex()
            .size_full()
            .child(
                // Left sidebar - panel selector
                div()
                    .flex()
                    .flex_col()
                    .w(px(240.0))
                    .h_full()
                    .bg(self.theme.background.default)
                    .border_r_1()
                    .border_color(self.theme.border.divider)
                    .p(self.theme.spacing(2.0))
                    .children(
                        panels.into_iter().map(|panel| {
                            let is_active = panel == self.active_panel;
                            let mut item = div()
                                .flex()
                                .items_center()
                                .px(self.theme.spacing(1.5))
                                .py(self.theme.spacing(1.0))
                                .mb(self.theme.spacing(0.5))
                                .rounded(self.theme.shape.border_radius_sm)
                                .child(
                                    div()
                                        .text_base()
                                        .mr(self.theme.spacing(1.0))
                                        .child(panel.icon()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if is_active {
                                            self.theme.secondary.on_secondary_container
                                        } else {
                                            self.theme.text.secondary
                                        })
                                        .child(panel.label()),
                                );

                            if is_active {
                                item = item.bg(self.theme.secondary.container);
                            }

                            item
                        }),
                    ),
            )
            .child(
                // Right panel - content
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .p(self.theme.spacing(3.0))
                    .child(self.render_panel_content(window, cx)),
            )
    }

    fn render_panel_content(&self, window: &mut Window, cx: &mut App) -> Div {
        match self.active_panel {
            WorkflowPanel::Constitution => self.render_constitution(window, cx),
            WorkflowPanel::ChangeManagement => self.render_changes(window, cx),
            WorkflowPanel::ReviewGate => self.render_reviews(window, cx),
            WorkflowPanel::ContextEngine => self.render_context(window, cx),
        }
    }

    fn render_constitution(&self, window: &mut Window, cx: &mut App) -> Div {
        let rules = &self.constitution_rules;

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .mb(self.theme.spacing(1.0))
                    .child("📜 Constitution"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(2.0))
                    .child(self.active_panel.description()),
            )
            .children(
                rules
                    .iter()
                    .map(|rule| rule.render(&self.theme, window, cx)),
            )
    }

    fn render_changes(&self, window: &mut Window, cx: &mut App) -> Div {
        let changes = &self.changes;

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .mb(self.theme.spacing(1.0))
                    .child("🔄 Change Management"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(2.0))
                    .child(self.active_panel.description()),
            )
            .children(
                changes
                    .iter()
                    .map(|change| change.render(&self.theme, window, cx)),
            )
    }

    fn render_reviews(&self, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .mb(self.theme.spacing(1.0))
                    .child("✓ Review Gate"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(2.0))
                    .child(self.active_panel.description()),
            )
            .child(
                div()
                    .text_base()
                    .text_color(self.theme.text.secondary)
                    .child("No pending reviews"),
            )
    }

    fn render_context(&self, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .mb(self.theme.spacing(1.0))
                    .child("🧠 Context Engine"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(2.0))
                    .child(self.active_panel.description()),
            )
            .child(
                div()
                    .text_base()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(1.0))
                    .child(format!("{} context files loaded", self.context_files.len())),
            )
            .children(
                self.context_files
                    .iter()
                    .map(|file| {
                        div()
                            .p(self.theme.spacing(1.0))
                            .mb(self.theme.spacing(0.5))
                            .bg(self.theme.background.paper)
                            .border_1()
                            .border_color(self.theme.border.divider)
                            .rounded(self.theme.shape.border_radius_sm)
                            .child(file.clone())
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_panel_labels() {
        assert_eq!(WorkflowPanel::Constitution.label(), "Constitution");
        assert_eq!(WorkflowPanel::ChangeManagement.label(), "Changes");
    }

    #[test]
    fn test_change_status_labels() {
        assert_eq!(ChangeStatus::Draft.label(), "Draft");
        assert_eq!(ChangeStatus::Complete.label(), "Complete");
    }

    #[test]
    fn test_workflows_view_creation() {
        let theme = MaterialTheme::dark();
        let view = WorkflowsView::new(theme);
        assert_eq!(view.active_panel, WorkflowPanel::Constitution);
    }
}

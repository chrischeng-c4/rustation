//! MCP view - MCP server inspector
//!
//! Provides UI for inspecting MCP (Model Context Protocol) server:
//! - Server status
//! - Available tools list
//! - Tool execution logs
//! - Server configuration

use gpui::*;
use rstn_ui::MaterialTheme;

/// MCP tool definition
#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
}

impl McpTool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Vec<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
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
                // Tool name
                div()
                    .text_base()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text.primary)
                    .mb(theme.spacing(0.5))
                    .child(self.name.clone()),
            )
            .child(
                // Description
                div()
                    .text_sm()
                    .text_color(theme.text.secondary)
                    .mb(theme.spacing(1.0))
                    .child(self.description.clone()),
            )
            .children(
                // Parameters
                if !self.parameters.is_empty() {
                    Some(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .children(self.parameters.iter().map(|param| {
                                div()
                                    .px(theme.spacing(1.0))
                                    .py(theme.spacing(0.5))
                                    .bg(theme.secondary.container)
                                    .rounded(theme.shape.border_radius_xs)
                                    .text_xs()
                                    .text_color(theme.secondary.on_secondary_container)
                                    .child(param.clone())
                            })),
                    )
                } else {
                    None
                },
            )
    }
}

/// Server status indicator
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Running,
    Stopped,
    Starting,
    Error,
}

impl ServerStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ServerStatus::Running => "Running",
            ServerStatus::Stopped => "Stopped",
            ServerStatus::Starting => "Starting",
            ServerStatus::Error => "Error",
        }
    }

    pub fn color(&self, theme: &MaterialTheme) -> Rgba {
        match self {
            ServerStatus::Running => rgb(0x4CAF50), // Green
            ServerStatus::Stopped => theme.text.disabled,
            ServerStatus::Starting => rgb(0xFFC107), // Amber/Yellow
            ServerStatus::Error => rgb(0xF44336), // Red
        }
    }
}

/// Main MCP view
pub struct McpView {
    pub status: ServerStatus,
    pub tools: Vec<McpTool>,
    pub server_url: String,
    pub theme: MaterialTheme,
}

impl McpView {
    pub fn new(status: ServerStatus, tools: Vec<McpTool>, server_url: impl Into<String>, theme: MaterialTheme) -> Self {
        Self {
            status,
            tools,
            server_url: server_url.into(),
            theme,
        }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Header with status
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p(self.theme.spacing(2.0))
                    .bg(self.theme.background.paper)
                    .border_b_1()
                    .border_color(self.theme.border.divider)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child(
                                // Status indicator
                                div()
                                    .w(px(12.0))
                                    .h(px(12.0))
                                    .rounded_full()
                                    .bg(self.status.color(&self.theme))
                                    .mr(self.theme.spacing(1.0)),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(self.theme.text.primary)
                                    .child(format!("MCP Server: {}", self.status.label())),
                            ),
                    )
                    .child(
                        // Server URL
                        div()
                            .text_sm()
                            .text_color(self.theme.text.secondary)
                            .child(self.server_url.clone()),
                    ),
            )
            .child(
                // Tools list
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p(self.theme.spacing(2.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(self.theme.text.primary)
                            .mb(self.theme.spacing(1.5))
                            .child(format!("Available Tools ({})", self.tools.len())),
                    )
                    .children(
                        self.tools
                            .iter()
                            .map(|tool| tool.render(&self.theme, window, cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_labels() {
        assert_eq!(ServerStatus::Running.label(), "Running");
        assert_eq!(ServerStatus::Stopped.label(), "Stopped");
        assert_eq!(ServerStatus::Error.label(), "Error");
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = McpTool::new("read_file", "Read a file", vec!["path".to_string()]);
        assert_eq!(tool.name, "read_file");
        assert_eq!(tool.parameters.len(), 1);
    }

    #[test]
    fn test_mcp_view_creation() {
        let theme = MaterialTheme::dark();
        let tools = vec![
            McpTool::new("tool1", "Description 1", vec![]),
            McpTool::new("tool2", "Description 2", vec!["param1".to_string()]),
        ];
        let view = McpView::new(ServerStatus::Running, tools, "http://localhost:5000", theme);
        assert_eq!(view.tools.len(), 2);
        assert_eq!(view.status, ServerStatus::Running);
    }
}

//! rstn - A developer workbench powered by GPUI
//!
//! This is the main entry point for the rustation desktop application.

use gpui::*;
use rstn_ui::{MaterialTheme, NavItem, PageHeader, ShellLayout, Sidebar};
use anyhow::Result;
use rstn_views::{
    ChatView, DockersView, ExplorerView, McpView,
    SettingsView, TasksView, TerminalView, WorkflowsView,
};

/// Main application view
struct AppView {
    /// Current active tab
    active_tab: &'static str,
}

impl AppView {
    fn new() -> Self {
        Self {
            active_tab: "tasks",
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = MaterialTheme::dark();

        // Create navigation items based on OLD_UI_ANALYSIS.md sidebar structure
        let nav_items = vec![
            NavItem::new("explorer", "Explorer", "📁"),
            NavItem::new("workflows", "Flows", "⚡"),
            NavItem::new("claude-code", "Claude", "🤖"),
            NavItem::new("tasks", "Tasks", "📋"),
            NavItem::new("mcp", "rstn", "🔌"),
            NavItem::new("chat", "Chat", "💬"),
            NavItem::new("a2ui", "A2UI", "🎨"),
            NavItem::new("terminal", "Term", "⌨️"),
        ];

        let sidebar = Sidebar::new(nav_items, self.active_tab.to_string(), theme.clone());
        let shell = ShellLayout::new("rstn - Developer Workbench", sidebar, theme.clone());

        // Render content based on active tab
        let content = self.render_content(&theme, _window, _cx);

        shell.render(content, _window, _cx)
    }
}

impl AppView {
    /// Render content area based on active tab
    fn render_content(&self, theme: &MaterialTheme, window: &mut Window, cx: &mut App) -> Div {
        match self.active_tab {
            "tasks" => {
                // TODO: Load actual commands from rstn-core::justfile
                let commands = vec![];
                TasksView::new(commands, theme.clone()).render(window, cx)
            }
            "dockers" => {
                // TODO: Load actual services from rstn-core::docker
                let services = vec![];
                DockersView::new(services, theme.clone()).render(window, cx)
            }
            "explorer" => {
                // TODO: Load actual file tree from rstn-core::worktree
                use rstn_views::explorer::{TreeNode, GitStatus};
                let current_path = "/".to_string();
                let root_node = TreeNode {
                    name: "root".to_string(),
                    path: "/".to_string(),
                    is_dir: true,
                    is_expanded: true,
                    children: vec![],
                    git_status: GitStatus::Unmodified,
                };
                let file_entries = vec![];
                ExplorerView::new(current_path, root_node, file_entries, theme.clone()).render(window, cx)
            }
            "terminal" => {
                // TODO: Load actual sessions from rstn-core::terminal
                let sessions = vec![];
                let active_session_index = 0;
                TerminalView::new(sessions, active_session_index, theme.clone()).render(window, cx)
            }
            "chat" => {
                // TODO: Load actual messages from chat history
                let messages = vec![];
                ChatView::new(messages, theme.clone()).render(window, cx)
            }
            "workflows" => {
                WorkflowsView::new(theme.clone()).render(window, cx)
            }
            "mcp" => {
                // TODO: Load actual MCP server status and tools
                use rstn_views::mcp::ServerStatus;
                let status = ServerStatus::Stopped;
                let tools = vec![];
                McpView::new(status, tools, "http://localhost:5000", theme.clone()).render(window, cx)
            }
            "settings" => {
                SettingsView::new(theme.clone()).render(window, cx)
            }
            _ => {
                // Fallback: Welcome screen
                let page_header = PageHeader::new(
                    "Welcome to rstn",
                    Some("GPUI-powered developer workbench"),
                    theme.clone(),
                );

                div()
                    .flex()
                    .flex_col()
                    .child(page_header.render(None::<Div>))
                    .child(
                        div()
                            .mt(theme.spacing(2.0))
                            .p(theme.spacing(1.5))
                            .bg(theme.background.paper)
                            .rounded(theme.shape.border_radius_sm)
                            .child(format!("Active tab: {}", self.active_tab)),
                    )
            }
        }
    }
}


fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting rstn...");

    // Initialize GPUI application
    Application::new()
        .with_assets(Assets)
        .run(|cx: &mut gpui::App| {
            // Create window options
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("rstn".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            };

            // Open main window
            cx.open_window(options, |_window, cx| {
                cx.new(|_cx| AppView::new())
            })
            .expect("Failed to open window");
        });
}

// Empty assets for now
struct Assets;

impl AssetSource for Assets {
    fn load(&self, _path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(None)
    }

    fn list(&self, _path: &str) -> Result<Vec<gpui::SharedString>> {
        Ok(Vec::new())
    }
}

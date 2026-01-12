//! rstn - A developer workbench powered by GPUI
//!
//! This is the main entry point for the rustation desktop application.

mod state;

use gpui::*;
use rstn_ui::{MaterialTheme, NavItem, PageHeader, ShellLayout, Sidebar};
use anyhow::Result;
use rstn_views::{
    ChatView, DockersView, ExplorerView, McpView,
    SettingsView, TasksView, TerminalView, WorkflowsView,
};
use state::AppState;

/// Main application view
struct AppView {
    /// Application state (GPUI Model)
    state: Model<AppState>,
}

impl AppView {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut state = AppState::new();
        state.initialize();

        Self {
            state: cx.new_model(|_cx| state),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = MaterialTheme::dark();

        // Read active tab from state
        let active_tab = self.state.read(_cx).active_tab.clone();

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

        let sidebar = Sidebar::new(nav_items, active_tab, theme.clone());
        let shell = ShellLayout::new("rstn - Developer Workbench", sidebar, theme.clone());

        // Render content based on active tab
        let content = self.render_content(&theme, _window, _cx);

        shell.render(content, _window, _cx)
    }
}

impl AppView {
    /// Render content area based on active tab
    fn render_content(&self, theme: &MaterialTheme, window: &mut Window, cx: &mut App) -> Div {
        let state = self.state.read(cx);
        let active_tab = state.active_tab.as_str();

        match active_tab {
            "tasks" => {
                // Load commands from state
                let commands = state.get_justfile_commands();
                TasksView::new(commands, theme.clone()).render(window, cx)
            }
            "dockers" => {
                // Load Docker services from state
                let services = state.get_docker_services();
                DockersView::new(services, theme.clone()).render(window, cx)
            }
            "explorer" => {
                // Load file tree from state
                let current_path = state.get_explorer_current_path();
                let tree_root = state.get_explorer_tree_root();
                let files = state.get_explorer_files();

                ExplorerView::new(current_path, tree_root, files, theme.clone()).render(window, cx)
            }
            "terminal" => {
                // Load terminal sessions from state
                let sessions = state.get_terminal_sessions();
                let active_session_index = state.get_active_terminal_session_index();
                TerminalView::new(sessions, active_session_index, theme.clone()).render(window, cx)
            }
            "chat" => {
                // Load chat messages from state
                let messages = state.get_chat_messages();
                ChatView::new(messages, theme.clone()).render(window, cx)
            }
            "workflows" => {
                // Load workflows data from state
                let constitution_rules = state.get_constitution_rules();
                let changes = state.get_changes();
                let context_files = state.get_context_files();

                WorkflowsView::new(
                    constitution_rules,
                    changes,
                    context_files,
                    theme.clone(),
                )
                .render(window, cx)
            }
            "mcp" => {
                // Load MCP server status and tools from state
                use rstn_views::mcp::ServerStatus;

                let status = match state.get_mcp_status() {
                    rstn_core::app_state::McpStatus::Stopped => ServerStatus::Stopped,
                    rstn_core::app_state::McpStatus::Starting => ServerStatus::Starting,
                    rstn_core::app_state::McpStatus::Running => ServerStatus::Running,
                    rstn_core::app_state::McpStatus::Error => ServerStatus::Error,
                };

                let tools = state.get_mcp_tools();
                let url = state.get_mcp_url();

                McpView::new(status, tools, &url, theme.clone()).render(window, cx)
            }
            "settings" => {
                // Load settings data from state
                let theme_setting = state.get_theme();
                let default_project_path = state.get_default_project_path();
                let current_project_path = state.get_current_project_path();
                let mcp_port = state.get_mcp_port();
                let mcp_config_path = state.get_mcp_config_path();

                SettingsView::new(
                    theme_setting,
                    default_project_path,
                    current_project_path,
                    mcp_port,
                    mcp_config_path,
                    theme.clone(),
                )
                .render(window, cx)
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
                            .child(format!("Active tab: {}", active_tab)),
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
                cx.new(|cx| AppView::new(cx))
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

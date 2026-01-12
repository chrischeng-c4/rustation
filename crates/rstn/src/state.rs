//! GPUI Application State
//!
//! This module wraps rstn-core::AppState and adds GPUI-specific functionality.

use rstn_core::app_state::AppState as CoreAppState;
use rstn_core::justfile;
use rstn_core::docker::BUILTIN_SERVICES;
use rstn_core::state::DockerService;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde_json::Value;

/// GPUI Application State
///
/// Wraps rstn-core::AppState and provides methods for loading data
/// into the state from the filesystem and external sources.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Core application state (from rstn-core)
    pub core: CoreAppState,

    /// Currently active view/tab
    pub active_tab: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            core: CoreAppState::default(),
            active_tab: "tasks".to_string(),
        }
    }
}

impl AppState {
    /// Create a new AppState
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize state with data from the current environment
    pub fn initialize(&mut self) {
        // Load justfile commands
        self.load_justfile_commands();

        // Load built-in Docker services
        self.load_docker_services();
    }

    /// Load justfile commands from the current directory
    fn load_justfile_commands(&mut self) {
        let justfile_path = env::current_dir()
            .ok()
            .and_then(|path| {
                let jf = path.join("justfile");
                if jf.exists() {
                    Some(jf)
                } else {
                    None
                }
            });

        if let Some(path) = justfile_path {
            if let Ok(commands) = justfile::parse_justfile(path.to_str().unwrap()) {
                // Store commands in the core state
                // For now, we'll need to create a default project if none exists
                if self.core.projects.is_empty() {
                    let current_dir = env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("/"))
                        .to_string_lossy()
                        .to_string();
                    let project = rstn_core::app_state::ProjectState::new(current_dir);
                    self.core.projects.push(project);
                }

                // Update tasks state in the active worktree
                if let Some(project) = self.core.active_project_mut() {
                    if let Some(worktree) = project.active_worktree_mut() {
                        worktree.tasks.commands = commands.into_iter().map(|cmd| {
                            rstn_core::app_state::JustCommandInfo {
                                name: cmd.name,
                                description: cmd.description,
                                recipe: cmd.recipe,
                            }
                        }).collect();
                    }
                }
            }
        }
    }

    /// Load built-in Docker services
    fn load_docker_services(&mut self) {
        let services: Vec<rstn_core::app_state::DockerServiceInfo> = BUILTIN_SERVICES
            .iter()
            .map(|config| rstn_core::app_state::DockerServiceInfo {
                id: config.id.to_string(),
                name: config.name.to_string(),
                image: config.image.to_string(),
                status: rstn_core::app_state::ServiceStatus::Stopped,
                port: Some(config.port as u32),
                service_type: Self::convert_service_type_to_app_state(config.service_type.clone()),
                project_group: Some("rstn".to_string()),
                is_rstn_managed: true,
            })
            .collect();

        self.core.docker.services = services;
    }

    /// Get justfile commands for the active worktree
    pub fn get_justfile_commands(&self) -> Vec<rstn_core::justfile::JustCommand> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                w.tasks.commands.iter().map(|cmd| rstn_core::justfile::JustCommand {
                    name: cmd.name.clone(),
                    description: cmd.description.clone(),
                    recipe: cmd.recipe.clone(),
                }).collect()
            })
            .unwrap_or_default()
    }

    /// Get Docker services
    pub fn get_docker_services(&self) -> Vec<DockerService> {
        self.core.docker.services.iter().map(|info| DockerService {
            id: info.id.clone(),
            name: info.name.clone(),
            image: info.image.clone(),
            status: Self::convert_service_status_from_app_state(info.status),
            port: info.port,
            service_type: Self::convert_service_type_from_app_state(info.service_type),
            project_group: info.project_group.clone(),
            is_rstn_managed: info.is_rstn_managed,
        }).collect()
    }

    /// Get MCP server state for the active worktree
    pub fn get_mcp_state(&self) -> Option<&rstn_core::app_state::McpState> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| &w.mcp)
    }

    /// Get MCP server status
    pub fn get_mcp_status(&self) -> rstn_core::app_state::McpStatus {
        self.get_mcp_state()
            .map(|s| s.status)
            .unwrap_or(rstn_core::app_state::McpStatus::Stopped)
    }

    /// Get MCP server URL
    pub fn get_mcp_url(&self) -> String {
        self.get_mcp_state()
            .and_then(|s| s.port)
            .map(|port| format!("http://localhost:{}", port))
            .unwrap_or_else(|| "http://localhost:5000".to_string())
    }

    /// Get MCP available tools (converted for rstn-views)
    pub fn get_mcp_tools(&self) -> Vec<rstn_views::mcp::McpTool> {
        self.get_mcp_state()
            .map(|s| {
                s.available_tools
                    .iter()
                    .map(|tool| Self::convert_mcp_tool(tool))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert rstn-core McpTool to rstn-views McpTool
    fn convert_mcp_tool(tool: &rstn_core::app_state::McpTool) -> rstn_views::mcp::McpTool {
        // Extract parameter names from input schema
        let parameters = if let Some(properties) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
            properties.keys().cloned().collect()
        } else {
            vec![]
        };

        rstn_views::mcp::McpTool::new(
            tool.name.clone(),
            tool.description.clone(),
            parameters,
        )
    }

    // ========================================================================
    // Workflows State Accessors
    // ========================================================================

    /// Get constitution rules (converted for rstn-views)
    pub fn get_constitution_rules(&self) -> Vec<rstn_views::workflows::ConstitutionRule> {
        // For now, return mock rules based on constitution_exists flag
        // In a real implementation, this would read from .rstn/constitutions/*.md
        let exists = self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .and_then(|w| w.tasks.constitution_exists)
            .unwrap_or(false);

        if exists {
            vec![
                rstn_views::workflows::ConstitutionRule::new(
                    "State-First Architecture",
                    true,
                    "All state must be JSON-serializable"
                ),
                rstn_views::workflows::ConstitutionRule::new(
                    "YAGNI Principle",
                    true,
                    "You Aren't Gonna Need It - avoid over-engineering"
                ),
                rstn_views::workflows::ConstitutionRule::new(
                    "Automated Verification",
                    true,
                    "Everything must be checkable without human intervention"
                ),
            ]
        } else {
            vec![]
        }
    }

    /// Get changes list (converted for rstn-views)
    pub fn get_changes(&self) -> Vec<rstn_views::workflows::ChangeItem> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                w.changes.changes
                    .iter()
                    .map(|change| Self::convert_change(change))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert rstn-core Change to rstn-views ChangeItem
    fn convert_change(change: &rstn_core::app_state::Change) -> rstn_views::workflows::ChangeItem {
        use rstn_views::workflows::ChangeStatus as ViewStatus;

        let status = match change.status {
            rstn_core::app_state::ChangeStatus::Proposed => ViewStatus::Proposed,
            rstn_core::app_state::ChangeStatus::Planning => ViewStatus::Draft,
            rstn_core::app_state::ChangeStatus::Planned => ViewStatus::Approved,
            rstn_core::app_state::ChangeStatus::Implementing => ViewStatus::Implementing,
            rstn_core::app_state::ChangeStatus::Testing => ViewStatus::Implementing,
            rstn_core::app_state::ChangeStatus::Done => ViewStatus::Complete,
            rstn_core::app_state::ChangeStatus::Archived => ViewStatus::Complete,
            rstn_core::app_state::ChangeStatus::Cancelled => ViewStatus::Draft,
            rstn_core::app_state::ChangeStatus::Failed => ViewStatus::Draft,
        };

        rstn_views::workflows::ChangeItem::new(
            change.name.clone(),
            status,
            change.intent.clone(),
        )
    }

    /// Get context files list
    pub fn get_context_files(&self) -> Vec<String> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                w.context.files
                    .iter()
                    .map(|f| f.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get review gate sessions count
    pub fn get_review_sessions_count(&self) -> usize {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| w.tasks.review_gate.sessions.len())
            .unwrap_or(0)
    }

    // ========================================================================
    // Settings State Accessors
    // ========================================================================

    /// Get global settings
    pub fn get_global_settings(&self) -> &rstn_core::app_state::GlobalSettings {
        &self.core.global_settings
    }

    /// Get theme setting
    pub fn get_theme(&self) -> String {
        match self.core.global_settings.theme {
            rstn_core::app_state::Theme::System => "System".to_string(),
            rstn_core::app_state::Theme::Light => "Light".to_string(),
            rstn_core::app_state::Theme::Dark => "Dark".to_string(),
        }
    }

    /// Get default project path
    pub fn get_default_project_path(&self) -> String {
        self.core
            .global_settings
            .default_project_path
            .clone()
            .unwrap_or_else(|| "~/projects".to_string())
    }

    /// Get MCP server port
    pub fn get_mcp_port(&self) -> String {
        self.get_mcp_state()
            .and_then(|s| s.port)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "5000".to_string())
    }

    /// Get MCP config path
    pub fn get_mcp_config_path(&self) -> String {
        self.get_mcp_state()
            .and_then(|s| s.config_path.clone())
            .unwrap_or_else(|| "~/.rstn/mcp-session.json".to_string())
    }

    /// Get current project path
    pub fn get_current_project_path(&self) -> String {
        self.core
            .active_project()
            .map(|p| p.path.clone())
            .unwrap_or_else(|| env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .to_string_lossy()
                .to_string())
    }

    // ========================================================================
    // Explorer State Accessors
    // ========================================================================

    /// Get explorer current path
    pub fn get_explorer_current_path(&self) -> String {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                if w.explorer.current_path.is_empty() {
                    w.path.clone()
                } else {
                    w.explorer.current_path.clone()
                }
            })
            .unwrap_or_else(|| self.get_current_project_path())
    }

    /// Get explorer file entries (converted for rstn-views)
    pub fn get_explorer_files(&self) -> Vec<rstn_views::explorer::FileEntry> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                w.explorer.entries
                    .iter()
                    .map(|entry| Self::convert_file_entry(entry))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get explorer tree root node
    pub fn get_explorer_tree_root(&self) -> rstn_views::explorer::TreeNode {
        let current_path = self.get_explorer_current_path();
        let path_buf = PathBuf::from(&current_path);
        let name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| current_path.clone());

        // Build tree from expanded paths
        let explorer_state = self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| &w.explorer);

        let children = if let Some(explorer) = explorer_state {
            explorer.entries
                .iter()
                .filter(|e| matches!(e.kind, rstn_core::app_state::FileKind::Directory))
                .map(|entry| {
                    let is_expanded = explorer.expanded_paths.contains(&entry.path);
                    let children = if is_expanded {
                        explorer.directory_cache
                            .get(&entry.path)
                            .map(|entries| {
                                entries
                                    .iter()
                                    .filter(|e| matches!(e.kind, rstn_core::app_state::FileKind::Directory))
                                    .map(|e| Self::convert_tree_node(e, false, vec![]))
                                    .collect()
                            })
                            .unwrap_or_default()
                    } else {
                        vec![]
                    };
                    Self::convert_tree_node(entry, is_expanded, children)
                })
                .collect()
        } else {
            vec![]
        };

        rstn_views::explorer::TreeNode {
            name,
            path: current_path,
            is_dir: true,
            is_expanded: true,
            children,
            git_status: rstn_views::explorer::GitStatus::Unmodified,
        }
    }

    /// Convert rstn-core FileEntry to rstn-views FileEntry
    fn convert_file_entry(entry: &rstn_core::app_state::FileEntry) -> rstn_views::explorer::FileEntry {
        rstn_views::explorer::FileEntry {
            path: entry.path.clone(),
            name: entry.name.clone(),
            is_dir: matches!(entry.kind, rstn_core::app_state::FileKind::Directory),
            git_status: entry.git_status
                .map(Self::convert_git_status)
                .unwrap_or(rstn_views::explorer::GitStatus::Unmodified),
            size: if matches!(entry.kind, rstn_core::app_state::FileKind::File) {
                Some(entry.size)
            } else {
                None
            },
            modified: Some(entry.updated_at.clone()),
        }
    }

    /// Convert rstn-core FileEntry to rstn-views TreeNode
    fn convert_tree_node(
        entry: &rstn_core::app_state::FileEntry,
        is_expanded: bool,
        children: Vec<rstn_views::explorer::TreeNode>,
    ) -> rstn_views::explorer::TreeNode {
        rstn_views::explorer::TreeNode {
            name: entry.name.clone(),
            path: entry.path.clone(),
            is_dir: matches!(entry.kind, rstn_core::app_state::FileKind::Directory),
            is_expanded,
            children,
            git_status: entry.git_status
                .map(Self::convert_git_status)
                .unwrap_or(rstn_views::explorer::GitStatus::Unmodified),
        }
    }

    /// Convert GitFileStatus from core to views
    fn convert_git_status(status: rstn_core::app_state::GitFileStatus) -> rstn_views::explorer::GitStatus {
        match status {
            rstn_core::app_state::GitFileStatus::Modified => rstn_views::explorer::GitStatus::Modified,
            rstn_core::app_state::GitFileStatus::Added => rstn_views::explorer::GitStatus::Added,
            rstn_core::app_state::GitFileStatus::Deleted => rstn_views::explorer::GitStatus::Deleted,
            rstn_core::app_state::GitFileStatus::Untracked => rstn_views::explorer::GitStatus::Untracked,
            rstn_core::app_state::GitFileStatus::Ignored => rstn_views::explorer::GitStatus::Unmodified,
            rstn_core::app_state::GitFileStatus::Clean => rstn_views::explorer::GitStatus::Unmodified,
        }
    }

    // ========================================================================
    // Terminal State Accessors
    // ========================================================================

    /// Get terminal sessions for the active worktree
    ///
    /// TODO: Full terminal integration requires:
    /// - Integrating alacritty_terminal crate
    /// - PTY session management
    /// - Terminal rendering with GPUI
    /// - Keyboard input handling
    /// - ANSI escape sequence parsing
    ///
    /// For now, returns empty Vec until terminal backend is implemented.
    pub fn get_terminal_sessions(&self) -> Vec<rstn_views::terminal::TerminalSession> {
        // Check if active worktree has a terminal session
        if let Some(project) = self.core.active_project() {
            if let Some(worktree) = project.active_worktree() {
                if let Some(_session_id) = &worktree.terminal.session_id {
                    // TODO: Return actual session data once terminal backend is implemented
                    // For now, return empty Vec to show empty state in UI
                    return vec![];
                }
            }
        }
        vec![]
    }

    /// Get active terminal session index
    pub fn get_active_terminal_session_index(&self) -> usize {
        // Default to first session (0) when terminal backend is implemented
        0
    }

    // ========================================================================
    // Chat State Accessors
    // ========================================================================

    /// Get chat messages for the active worktree (converted for rstn-views)
    pub fn get_chat_messages(&self) -> Vec<rstn_views::chat::ChatMessage> {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| {
                w.chat.messages
                    .iter()
                    .map(|msg| Self::convert_chat_message(msg))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert rstn-core ChatMessage to rstn-views ChatMessage
    fn convert_chat_message(msg: &rstn_core::app_state::ChatMessage) -> rstn_views::chat::ChatMessage {
        use rstn_views::chat::MessageRole as ViewRole;

        let role = match msg.role {
            rstn_core::app_state::ChatRole::User => ViewRole::User,
            rstn_core::app_state::ChatRole::Assistant => ViewRole::Assistant,
            rstn_core::app_state::ChatRole::System => ViewRole::System,
        };

        rstn_views::chat::ChatMessage::new(
            role,
            msg.content.clone(),
            msg.timestamp.clone(),
        )
    }

    /// Get chat typing state
    pub fn is_chat_typing(&self) -> bool {
        self.core
            .active_project()
            .and_then(|p| p.active_worktree())
            .map(|w| w.chat.is_typing)
            .unwrap_or(false)
    }

    // ========================================================================
    // Action Dispatch
    // ========================================================================

    /// Switch to a different tab
    pub fn switch_tab(&mut self, tab: impl Into<String>) {
        self.active_tab = tab.into();
    }

    /// Dispatch an action to modify state
    pub fn dispatch(&mut self, action: AppAction) {
        match action {
            AppAction::SwitchTab(tab) => {
                self.switch_tab(tab);
            }
            AppAction::ExecuteCommand(command_name) => {
                self.execute_command(&command_name);
            }
            AppAction::RefreshDockerServices => {
                self.refresh_docker_services();
            }
        }
    }

    /// Execute a justfile command
    fn execute_command(&mut self, command_name: &str) {
        tracing::info!("Executing command: {}", command_name);

        // Get current directory for justfile execution
        let current_dir = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();

        // Update task status to Running
        if let Some(project) = self.core.active_project_mut() {
            if let Some(worktree) = project.active_worktree_mut() {
                worktree.tasks.task_statuses.insert(
                    command_name.to_string(),
                    rstn_core::app_state::TaskStatus::Running,
                );
                worktree.tasks.active_command = Some(command_name.to_string());
            }
        }

        // Spawn async task to execute command
        // Note: In GPUI, we'll need to use cx.spawn() for this
        // For now, we'll just log that execution started
        tracing::info!("Command '{}' execution started in {}", command_name, current_dir);
    }

    /// Refresh Docker service statuses
    fn refresh_docker_services(&mut self) {
        tracing::info!("Refreshing Docker services");

        // Set loading state
        self.core.docker.is_loading = true;

        // In a real implementation, we would:
        // 1. Query Docker daemon for container statuses
        // 2. Update service statuses in self.core.docker.services
        // 3. Set is_loading = false

        // For now, just log the refresh
        tracing::info!("Docker services refresh requested");
    }

    /// Execute a justfile command asynchronously
    /// Returns a tuple of (exit_code, output_lines)
    pub async fn execute_command_async(command_name: String) -> Result<(i32, Vec<String>), String> {
        tracing::info!("Executing command async: {}", command_name);

        let current_dir = env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        // Execute using `just` command
        let mut child = Command::new("just")
            .arg(&command_name)
            .current_dir(&current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let mut output_lines = Vec::new();

        // Read stdout
        let mut stdout_reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            tracing::debug!("[{}] {}", command_name, line);
            output_lines.push(line);
        }

        // Read stderr
        let mut stderr_reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            tracing::warn!("[{}] stderr: {}", command_name, line);
            output_lines.push(format!("ERROR: {}", line));
        }

        // Wait for command to finish
        let status = child.wait().await
            .map_err(|e| format!("Failed to wait for command: {}", e))?;

        let exit_code = status.code().unwrap_or(-1);
        tracing::info!("Command '{}' finished with exit code: {}", command_name, exit_code);

        Ok((exit_code, output_lines))
    }

    /// Poll Docker services asynchronously
    pub async fn poll_docker_services() -> Result<Vec<DockerServiceInfo>, String> {
        tracing::debug!("Polling Docker services");

        // Query Docker for running containers
        let output = Command::new("docker")
            .args(&["ps", "-a", "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}"])
            .output()
            .await
            .map_err(|e| format!("Failed to query Docker: {}", e))?;

        if !output.status.success() {
            return Err("Docker command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let id = parts[0].to_string();
                let name = parts[1].to_string();
                let image = parts[2].to_string();
                let status_str = parts[3];

                let status = if status_str.contains("Up") {
                    rstn_core::app_state::ServiceStatus::Running
                } else {
                    rstn_core::app_state::ServiceStatus::Stopped
                };

                // Parse port if available
                let port = if parts.len() >= 5 {
                    parts[4].split(':')
                        .nth(1)
                        .and_then(|p| p.split("->").next())
                        .and_then(|p| p.parse::<u32>().ok())
                } else {
                    None
                };

                services.push(DockerServiceInfo {
                    id,
                    name: name.clone(),
                    image,
                    status,
                    port,
                    service_type: rstn_core::app_state::ServiceType::Other,
                    project_group: None,
                    is_rstn_managed: name.starts_with("rstn-"),
                });
            }
        }

        Ok(services)
    }

    /// Check MCP server health asynchronously
    /// Returns Ok(true) if server is running and healthy
    pub async fn check_mcp_health(url: &str) -> Result<bool, String> {
        tracing::debug!("Checking MCP server health at {}", url);

        // Try to fetch tools/list endpoint as health check
        let health_url = format!("{}/health", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        match client.get(&health_url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                tracing::debug!("MCP server health check: {}", if is_healthy { "healthy" } else { "unhealthy" });
                Ok(is_healthy)
            }
            Err(e) => {
                tracing::warn!("MCP server health check failed: {}", e);
                Ok(false) // Server not reachable, but not an error
            }
        }
    }

    /// Fetch MCP tools list asynchronously
    pub async fn fetch_mcp_tools(url: &str) -> Result<Vec<rstn_core::app_state::McpTool>, String> {
        tracing::debug!("Fetching MCP tools from {}", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // MCP tools/list endpoint (JSON-RPC 2.0)
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        let response = client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server returned status: {}", response.status()));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Parse JSON-RPC response
        let tools_array = body
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .ok_or_else(|| "Invalid response format".to_string())?;

        let mut tools = Vec::new();
        for tool in tools_array {
            if let (Some(name), Some(description)) = (
                tool.get("name").and_then(|n| n.as_str()),
                tool.get("description").and_then(|d| d.as_str()),
            ) {
                tools.push(rstn_core::app_state::McpTool {
                    name: name.to_string(),
                    description: description.to_string(),
                    input_schema: tool
                        .get("inputSchema")
                        .cloned()
                        .unwrap_or(Value::Object(serde_json::Map::new())),
                });
            }
        }

        tracing::info!("Fetched {} MCP tools", tools.len());
        Ok(tools)
    }
}

/// Docker service info (re-export from core)
pub use rstn_core::app_state::DockerServiceInfo;

/// Application actions (events that modify state)
#[derive(Debug, Clone)]
pub enum AppAction {
    /// Switch to a different tab
    SwitchTab(String),
    /// Execute a justfile command
    ExecuteCommand(String),
    /// Refresh Docker service statuses
    RefreshDockerServices,
}

impl AppState {
    /// Convert ServiceType from state module to app_state module
    fn convert_service_type_to_app_state(
        service_type: rstn_core::state::ServiceType,
    ) -> rstn_core::app_state::ServiceType {
        match service_type {
            rstn_core::state::ServiceType::Database => rstn_core::app_state::ServiceType::Database,
            rstn_core::state::ServiceType::MessageBroker => rstn_core::app_state::ServiceType::MessageBroker,
            rstn_core::state::ServiceType::Cache => rstn_core::app_state::ServiceType::Cache,
            rstn_core::state::ServiceType::Other => rstn_core::app_state::ServiceType::Other,
        }
    }

    /// Convert ServiceType from app_state module to state module
    fn convert_service_type_from_app_state(
        service_type: rstn_core::app_state::ServiceType,
    ) -> rstn_core::state::ServiceType {
        match service_type {
            rstn_core::app_state::ServiceType::Database => rstn_core::state::ServiceType::Database,
            rstn_core::app_state::ServiceType::MessageBroker => rstn_core::state::ServiceType::MessageBroker,
            rstn_core::app_state::ServiceType::Cache => rstn_core::state::ServiceType::Cache,
            rstn_core::app_state::ServiceType::Other => rstn_core::state::ServiceType::Other,
        }
    }

    /// Convert ServiceStatus from app_state module to state module
    fn convert_service_status_from_app_state(
        status: rstn_core::app_state::ServiceStatus,
    ) -> rstn_core::state::ServiceStatus {
        match status {
            rstn_core::app_state::ServiceStatus::Running => rstn_core::state::ServiceStatus::Running,
            rstn_core::app_state::ServiceStatus::Stopped => rstn_core::state::ServiceStatus::Stopped,
            rstn_core::app_state::ServiceStatus::Starting => rstn_core::state::ServiceStatus::Starting,
            rstn_core::app_state::ServiceStatus::Stopping => rstn_core::state::ServiceStatus::Stopped,
            rstn_core::app_state::ServiceStatus::Error => rstn_core::state::ServiceStatus::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.active_tab, "tasks");
        assert_eq!(state.core.projects.len(), 0);
    }

    #[test]
    fn test_app_state_initialize() {
        let mut state = AppState::new();
        state.initialize();

        // Should have loaded Docker services
        assert!(state.core.docker.services.len() > 0);
    }

    #[test]
    fn test_switch_tab() {
        let mut state = AppState::new();
        assert_eq!(state.active_tab, "tasks");

        state.switch_tab("dockers");
        assert_eq!(state.active_tab, "dockers");
    }

    // ========================================================================
    // View Accessor Tests (Stage 3)
    // ========================================================================

    #[test]
    fn test_get_terminal_sessions_empty_state() {
        let state = AppState::new();
        let sessions = state.get_terminal_sessions();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_get_active_terminal_session_index() {
        let state = AppState::new();
        let index = state.get_active_terminal_session_index();
        assert_eq!(index, 0);
    }

    #[test]
    fn test_get_chat_messages_empty_state() {
        let state = AppState::new();
        let messages = state.get_chat_messages();
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_is_chat_typing_default() {
        let state = AppState::new();
        assert!(!state.is_chat_typing());
    }

    #[test]
    fn test_get_explorer_current_path_fallback() {
        let state = AppState::new();
        let path = state.get_explorer_current_path();
        // Should return current directory as fallback
        assert!(!path.is_empty());
    }

    #[test]
    fn test_get_explorer_files_empty_state() {
        let state = AppState::new();
        let files = state.get_explorer_files();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_get_mcp_status_no_project() {
        let state = AppState::new();
        let status = state.get_mcp_status();
        // Should return Stopped when no project
        assert_eq!(status, rstn_core::app_state::McpStatus::Stopped);
    }

    #[test]
    fn test_get_mcp_tools_empty() {
        let state = AppState::new();
        let tools = state.get_mcp_tools();
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_get_constitution_rules_empty() {
        let state = AppState::new();
        let rules = state.get_constitution_rules();
        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_get_changes_empty() {
        let state = AppState::new();
        let changes = state.get_changes();
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_get_context_files_empty() {
        let state = AppState::new();
        let files = state.get_context_files();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_get_theme_default() {
        let state = AppState::new();
        let theme = state.get_theme();
        // Default theme is Dark
        assert_eq!(theme, "Dark");
    }

    #[test]
    fn test_get_default_project_path_fallback() {
        let state = AppState::new();
        let path = state.get_default_project_path();
        assert_eq!(path, "~/projects");
    }

    #[test]
    fn test_get_current_project_path_fallback() {
        let state = AppState::new();
        let path = state.get_current_project_path();
        // Should return current directory as fallback
        assert!(!path.is_empty());
    }

    #[test]
    fn test_get_mcp_port_default() {
        let state = AppState::new();
        let port = state.get_mcp_port();
        assert_eq!(port, "5000");
    }

    #[test]
    fn test_get_mcp_config_path_default() {
        let state = AppState::new();
        let path = state.get_mcp_config_path();
        assert_eq!(path, "~/.rstn/mcp-session.json");
    }
}

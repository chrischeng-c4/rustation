//! rstn-core - napi-rs Rust addon for rustation desktop app.
//!
//! Provides Docker management, MCP server, and state management.

#[macro_use]
extern crate napi_derive;

pub mod actions;
pub mod agent_rules;
pub mod app_state;
pub mod claude_cli;
pub mod constitution;
pub mod context_engine;
pub mod docker;
pub mod env;
pub mod justfile;
pub mod mcp_config;
pub mod mcp_server;
pub mod migration;
pub mod persistence;
pub mod reducer;
pub mod state;
pub mod terminal;
pub mod worktree;

use actions::Action;
use app_state::AppState;
use docker::DockerManager;
use mcp_server::McpServerManager;
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use reducer::reduce;
use state::DockerService;
use std::sync::{Arc, OnceLock};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{OnceCell, RwLock};

// Global Docker manager instance
static DOCKER_MANAGER: OnceCell<Arc<DockerManager>> = OnceCell::const_new();

// Global MCP server manager instance (sync init, doesn't need tokio::OnceCell)
static MCP_SERVER_MANAGER: OnceLock<Arc<McpServerManager>> = OnceLock::new();

// Global application state
static APP_STATE: OnceCell<Arc<RwLock<AppState>>> = OnceCell::const_new();

// State update listener (callback to JavaScript)
static STATE_LISTENER: OnceCell<ThreadsafeFunction<String>> = OnceCell::const_new();

// DEFAULT_CONSTITUTION moved to constitution.rs module (modular templates)

fn get_app_state() -> &'static Arc<RwLock<AppState>> {
    APP_STATE.get().expect("AppState not initialized. Call state_init first.")
}

/// Push state update to JavaScript listener
async fn notify_state_update() {
    if let Some(listener) = STATE_LISTENER.get() {
        let state = get_app_state().read().await;
        if let Ok(json) = serde_json::to_string(&*state) {
            listener.call(Ok(json), ThreadsafeFunctionCallMode::NonBlocking);
        }
    }
}

async fn get_docker_manager() -> napi::Result<&'static Arc<DockerManager>> {
    DOCKER_MANAGER
        .get_or_try_init(|| async {
            DockerManager::new()
                .map(Arc::new)
                .map_err(|e| napi::Error::from_reason(format!("Docker not available: {}", e)))
        })
        .await
}

fn get_mcp_server_manager() -> &'static Arc<McpServerManager> {
    MCP_SERVER_MANAGER.get_or_init(|| Arc::new(McpServerManager::new()))
}


/// Check if Docker is available
#[napi]
pub async fn docker_is_available() -> bool {
    match get_docker_manager().await {
        Ok(dm) => dm.is_available().await,
        Err(_) => false,
    }
}

/// List all Docker services
#[napi]
pub async fn docker_list_services() -> napi::Result<Vec<DockerService>> {
    let dm = get_docker_manager().await?;
    Ok(dm.list_services().await)
}

/// Start a Docker service
#[napi]
pub async fn docker_start_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.start_service(&service_id)
        .await
        .map_err(napi::Error::from_reason)
}

/// Stop a Docker service
#[napi]
pub async fn docker_stop_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.stop_service(&service_id)
        .await
        .map_err(napi::Error::from_reason)
}

/// Restart a Docker service
#[napi]
pub async fn docker_restart_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.restart_service(&service_id)
        .await
        .map_err(napi::Error::from_reason)
}

/// Get container logs
#[napi]
pub async fn docker_get_logs(service_id: String, tail: Option<u32>) -> napi::Result<Vec<String>> {
    let dm = get_docker_manager().await?;
    let tail = tail.unwrap_or(100) as usize;
    dm.get_logs(&service_id, tail)
        .await
        .map_err(napi::Error::from_reason)
}

/// Remove a Docker service
#[napi]
pub async fn docker_remove_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.remove_service(&service_id)
        .await
        .map_err(napi::Error::from_reason)
}

/// Create a database in a database container
/// Returns the connection string for the new database
#[napi]
pub async fn docker_create_database(service_id: String, db_name: String) -> napi::Result<String> {
    let dm = get_docker_manager().await?;
    dm.create_database(&service_id, &db_name)
        .await
        .map_err(napi::Error::from_reason)
}

/// Create a vhost in RabbitMQ
/// Returns the connection string for the new vhost
#[napi]
pub async fn docker_create_vhost(service_id: String, vhost_name: String) -> napi::Result<String> {
    let dm = get_docker_manager().await?;
    dm.create_vhost(&service_id, &vhost_name)
        .await
        .map_err(napi::Error::from_reason)
}

/// Start a Docker service with a specific port override
#[napi]
pub async fn docker_start_service_with_port(service_id: String, port: u16) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.start_service_with_port(&service_id, port)
        .await
        .map_err(napi::Error::from_reason)
}

/// Stop any Docker container by ID or name
#[napi]
pub async fn docker_stop_container(container_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.stop_container(&container_id)
        .await
        .map_err(napi::Error::from_reason)
}

/// Check for port conflict before starting a service
#[napi]
pub async fn docker_check_port_conflict(service_id: String) -> napi::Result<Option<state::PortConflictInfo>> {
    let dm = get_docker_manager().await?;
    dm.check_port_conflict(&service_id)
        .await
        .map_err(napi::Error::from_reason)
}

// ============================================================================
// Justfile functions
// ============================================================================

/// Parse a justfile and return all commands
#[napi]
pub fn justfile_parse(path: String) -> napi::Result<Vec<justfile::JustCommand>> {
    justfile::parse_justfile(&path)
        .map_err(napi::Error::from_reason)
}

/// Run a just command in a directory
#[napi]
pub fn justfile_run(command: String, cwd: String) -> napi::Result<String> {
    justfile::run_just_command(&command, &cwd)
        .map_err(napi::Error::from_reason)
}

// ============================================================================
// Worktree functions
// ============================================================================

/// Branch info for napi export
#[napi(object)]
pub struct NapiBranchInfo {
    pub name: String,
    pub has_worktree: bool,
    pub is_current: bool,
}

/// List all branches in a repository
#[napi]
pub fn worktree_list_branches(repo_path: String) -> napi::Result<Vec<NapiBranchInfo>> {
    worktree::list_branches(&repo_path)
        .map(|branches| {
            branches
                .into_iter()
                .map(|b| NapiBranchInfo {
                    name: b.name,
                    has_worktree: b.has_worktree,
                    is_current: b.is_current,
                })
                .collect()
        })
        .map_err(napi::Error::from_reason)
}

// ============================================================================
// Env functions
// ============================================================================

/// List env files matching patterns in a directory
#[napi]
pub fn env_list_files(dir: String, patterns: Vec<String>) -> Vec<String> {
    env::list_env_files(&dir, &patterns)
}

/// Get default env patterns
#[napi]
pub fn env_default_patterns() -> Vec<String> {
    env::default_patterns()
}

// ============================================================================
// MCP functions
// ============================================================================

/// Fetch available tools from MCP server
#[napi]
pub async fn fetch_mcp_tools() -> napi::Result<String> {
    // Check if state is initialized
    let state_cell = APP_STATE.get().ok_or_else(|| {
        napi::Error::from_reason("AppState not initialized. Call state_init first.")
    })?;
    let state = state_cell.read().await;

    let port = if let Some(project) = state.active_project() {
        if let Some(worktree) = project.active_worktree() {
            worktree.mcp.port
        } else {
            None
        }
    } else {
        None
    };

    let Some(port) = port else {
        eprintln!("[fetch_mcp_tools] No port available");
        // Return JSON-RPC formatted empty response
        return Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "tools": [] }
        }).to_string());
    };

    // Call MCP server's tools/list endpoint
    let url = format!("http://localhost:{}/mcp", port);
    eprintln!("[fetch_mcp_tools] Fetching tools from: {}", url);
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }))
        .send()
        .await
        .map_err(|e| {
            eprintln!("[fetch_mcp_tools] HTTP error: {}", e);
            napi::Error::from_reason(format!("HTTP error: {}", e))
        })?;

    let body = response
        .text()
        .await
        .map_err(|e| {
            eprintln!("[fetch_mcp_tools] Read error: {}", e);
            napi::Error::from_reason(format!("Read error: {}", e))
        })?;

    eprintln!("[fetch_mcp_tools] Response body: {}", body);
    Ok(body)
}

// ============================================================================
// Context Engine functions
// ============================================================================

/// AI Context for napi export
#[napi(object)]
pub struct NapiAIContext {
    /// Open files with content
    pub open_files: Vec<NapiFileContext>,
    /// Last terminal output
    pub terminal_last_output: Option<String>,
    /// Git status summary
    pub git_status: String,
    /// Active errors
    pub active_errors: Vec<String>,
    /// Directory tree
    pub directory_tree: Option<String>,
    /// Git diff
    pub git_diff: Option<String>,
}

/// File context for napi export
#[napi(object)]
pub struct NapiFileContext {
    /// File path
    pub path: String,
    /// File content (may be truncated)
    pub content: String,
    /// Cursor line if available
    pub cursor_line: Option<u32>,
}

/// Build AI context for a project path
///
/// Gathers context from git, files, and other sources within a token budget.
#[napi]
pub fn context_build(
    project_path: String,
    active_files: Vec<String>,
    task_output: Option<String>,
    docker_errors: Vec<String>,
    token_budget: Option<u32>,
) -> NapiAIContext {
    let budget = token_budget.unwrap_or(20000) as usize;
    let path = std::path::Path::new(&project_path);

    let context = context_engine::build_context(
        path,
        active_files,
        task_output,
        docker_errors,
        budget,
    );

    NapiAIContext {
        open_files: context.open_files.into_iter().map(|f| NapiFileContext {
            path: f.path,
            content: f.content,
            cursor_line: f.cursor_line.map(|l| l as u32),
        }).collect(),
        terminal_last_output: context.terminal_last_output,
        git_status: context.git_status,
        active_errors: context.active_errors,
        directory_tree: context.directory_tree,
        git_diff: context.git_diff,
    }
}

/// Build AI context and format as a system prompt string
#[napi]
pub fn context_build_system_prompt(
    project_path: String,
    active_files: Vec<String>,
    task_output: Option<String>,
    docker_errors: Vec<String>,
    token_budget: Option<u32>,
) -> String {
    let budget = token_budget.unwrap_or(20000) as usize;
    let path = std::path::Path::new(&project_path);

    let context = context_engine::build_context(
        path,
        active_files,
        task_output,
        docker_errors,
        budget,
    );

    context.to_system_prompt()
}

// ============================================================================
// State Management (State-first architecture)
// ============================================================================

/// Initialize the application state and register a listener for state updates.
///
/// The callback will be invoked with the JSON-serialized state whenever it changes.
/// This should be called once during app startup.
#[napi]
pub fn state_init(
    #[napi(ts_arg_type = "(err: Error | null, state: string) => void")] callback: napi::JsFunction,
) -> napi::Result<()> {
    // Initialize the state with defaults
    let mut initial_state = AppState::default();

    // Load persisted global state if available
    if let Ok(Some(persisted)) = persistence::load_global() {
        persisted.apply_to(&mut initial_state);

        // Auto-open the most recent project if it exists on disk
        if let Some(recent) = initial_state.recent_projects.first() {
            let path = recent.path.clone();
            if std::path::Path::new(&path).exists() {
                reduce(&mut initial_state, Action::OpenProject { path });
            }
        }
    }

    let _ = APP_STATE.set(Arc::new(RwLock::new(initial_state)));

    // Create threadsafe function for callbacks
    let tsfn: ThreadsafeFunction<String> = callback.create_threadsafe_function(
        0,
        |ctx: ThreadSafeCallContext<String>| {
            ctx.env.create_string(&ctx.value).map(|v| vec![v])
        },
    )?;

    let _ = STATE_LISTENER.set(tsfn);

    Ok(())
}

/// Get the current state as JSON.
#[napi]
pub async fn state_get() -> napi::Result<String> {
    let state = get_app_state().read().await;
    serde_json::to_string(&*state)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize state: {}", e)))
}

/// Dispatch an action to update the state.
///
/// The action should be a JSON object with the format:
/// `{ "type": "ActionName", "payload": { ... } }`
///
/// After the action is processed, the state listener will be notified.
#[napi]
pub async fn state_dispatch(action_json: String) -> napi::Result<()> {
    // Parse the action
    let action: Action = serde_json::from_str(&action_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid action JSON: {}", e)))?;

    // Apply synchronous state changes first
    {
        let mut state = get_app_state().write().await;
        reduce(&mut state, action.clone());
    }

    // Handle async operations based on action type
    handle_async_action(action).await?;

    // Auto-save state (non-blocking)
    {
        let state = get_app_state().read().await;
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = persistence::save_global(&state_clone) {
                tracing::warn!("Failed to save global state: {}", e);
            }
        });
    }

    // Notify listener of state update
    notify_state_update().await;

    Ok(())
}

/// Refresh Docker services and update state
async fn refresh_docker_services_internal() {
    match docker_list_services().await {
        Ok(services) => {
            let service_data: Vec<actions::DockerServiceData> = services
                .into_iter()
                .map(|s| actions::DockerServiceData {
                    id: s.id,
                    name: s.name,
                    image: s.image,
                    status: s.status,
                    port: s.port,
                    service_type: s.service_type,
                    project_group: s.project_group,
                    is_rstn_managed: s.is_rstn_managed,
                })
                .collect();
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetDockerServices { services: service_data });
        }
        Err(e) => {
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetError {
                code: "DOCKER_LIST_ERROR".to_string(),
                message: e.to_string(),
                context: Some("RefreshDockerServices".to_string()),
            });
            reduce(&mut state, Action::SetDockerLoading { is_loading: false });
        }
    }
}

/// Refresh worktrees for a given project path
async fn refresh_worktrees_for_path(project_path: &str) {
    match worktree::list_worktrees(project_path) {
        Ok(worktrees) => {
            let worktree_data: Vec<actions::WorktreeData> = worktrees;
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetWorktrees { worktrees: worktree_data });
        }
        Err(e) => {
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetError {
                code: "WORKTREE_REFRESH_ERROR".to_string(),
                message: e,
                context: Some(format!("RefreshWorktrees: {}", project_path)),
            });
        }
    }
}

/// Handle async operations for actions that require backend calls.
async fn handle_async_action(action: Action) -> napi::Result<()> {
    match action {
        Action::CheckDockerAvailability => {
            let available = docker_is_available().await;
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetDockerAvailable { available });
        }

        Action::RefreshDockerServices => {
            refresh_docker_services_internal().await;
        }

        Action::StartDockerService { ref service_id } => {
            // Check for port conflict first
            match docker_check_port_conflict(service_id.clone()).await {
                Ok(Some(conflict_info)) => {
                    // Port conflict detected - set pending conflict for UI to handle
                    let conflict_data = actions::PortConflictData {
                        requested_port: conflict_info.requested_port as u16,
                        conflicting_container: actions::ConflictingContainerData {
                            id: conflict_info.container_id,
                            name: conflict_info.container_name,
                            image: conflict_info.container_image,
                            is_rstn_managed: conflict_info.is_rstn_managed,
                        },
                        suggested_port: conflict_info.suggested_port as u16,
                    };
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetPortConflict {
                        service_id: service_id.clone(),
                        conflict: conflict_data,
                    });
                }
                Ok(None) => {
                    // No conflict, proceed with start
                    match docker_start_service(service_id.clone()).await {
                        Ok(()) => {
                            refresh_docker_services_internal().await;
                        }
                        Err(e) => {
                            let mut state = get_app_state().write().await;
                            reduce(&mut state, Action::SetError {
                                code: "DOCKER_START_ERROR".to_string(),
                                message: e.to_string(),
                                context: Some(format!("StartDockerService: {}", service_id)),
                            });
                        }
                    }
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_PORT_CHECK_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("CheckPortConflict: {}", service_id)),
                    });
                }
            }
        }

        Action::StopDockerService { ref service_id } => {
            match docker_stop_service(service_id.clone()).await {
                Ok(()) => {
                    // Refresh services to get updated status
                    refresh_docker_services_internal().await;
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_STOP_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("StopDockerService: {}", service_id)),
                    });
                }
            }
        }

        Action::RestartDockerService { ref service_id } => {
            match docker_restart_service(service_id.clone()).await {
                Ok(()) => {
                    // Refresh services to get updated status
                    refresh_docker_services_internal().await;
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_RESTART_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("RestartDockerService: {}", service_id)),
                    });
                }
            }
        }

        Action::FetchDockerLogs { ref service_id, tail } => {
            match docker_get_logs(service_id.clone(), Some(tail)).await {
                Ok(logs) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetDockerLogs { logs });
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_LOGS_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("FetchDockerLogs: {}", service_id)),
                    });
                    reduce(&mut state, Action::SetDockerLogsLoading { is_loading: false });
                }
            }
        }

        Action::CreateDatabase { ref service_id, ref db_name } => {
            match docker_create_database(service_id.clone(), db_name.clone()).await {
                Ok(_connection_string) => {
                    // Database created successfully
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_CREATE_DB_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("CreateDatabase: {} in {}", db_name, service_id)),
                    });
                }
            }
        }

        Action::CreateVhost { ref service_id, ref vhost_name } => {
            match docker_create_vhost(service_id.clone(), vhost_name.clone()).await {
                Ok(_connection_string) => {
                    // Vhost created successfully
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_CREATE_VHOST_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("CreateVhost: {} in {}", vhost_name, service_id)),
                    });
                }
            }
        }

        // ====================================================================
        // MCP Server Actions
        // ====================================================================
        Action::StartMcpServer => {
            // Get worktree info from state
            let (worktree_id, worktree_path, project_name) = {
                let state = get_app_state().read().await;
                if let Some(project) = state.active_project() {
                    if let Some(worktree) = project.active_worktree() {
                        (worktree.id.clone(), worktree.path.clone(), project.name.clone())
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            };

            let manager = get_mcp_server_manager();
            match manager.start_server(
                worktree_id.clone(),
                std::path::PathBuf::from(&worktree_path),
                project_name,
                None, // Use default port
            ).await {
                Ok(port) => {
                    // Generate MCP config file for Claude CLI integration
                    let config_result = mcp_config::generate_mcp_config_file(&worktree_id, port);

                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetMcpPort { port });

                    // Store config path if generation succeeded
                    match config_result {
                        Ok(config_path) => {
                            reduce(&mut state, Action::SetMcpConfigPath { path: config_path });
                        }
                        Err(e) => {
                            // Non-fatal: Log warning but don't fail server startup
                            eprintln!("Warning: Failed to generate MCP config: {}", e);
                        }
                    }

                    // Release the write lock before fetching tools
                    drop(state);

                    // Fetch and store available tools
                    match fetch_mcp_tools().await {
                        Ok(json_str) => {
                            match serde_json::from_str::<serde_json::Value>(&json_str) {
                                Ok(data) => {
                                    if let Some(tools_array) = data.get("result")
                                        .and_then(|r| r.get("tools"))
                                        .and_then(|t| t.as_array())
                                    {
                                        let tools: Vec<actions::McpToolData> = tools_array
                                            .iter()
                                            .filter_map(|tool| {
                                                Some(actions::McpToolData {
                                                    name: tool.get("name")?.as_str()?.to_string(),
                                                    description: tool.get("description")?.as_str()?.to_string(),
                                                    input_schema: tool.get("input_schema")?.clone(),
                                                })
                                            })
                                            .collect();

                                        let mut state = get_app_state().write().await;
                                        reduce(&mut state, Action::UpdateMcpTools { tools });
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to parse MCP tools response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to fetch MCP tools: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetMcpError { error: e });
                }
            }
        }

        Action::StopMcpServer => {
            // Get worktree info and config path from state
            let (worktree_id, config_path) = {
                let state = get_app_state().read().await;
                if let Some(project) = state.active_project() {
                    if let Some(worktree) = project.active_worktree() {
                        (worktree.id.clone(), worktree.mcp.config_path.clone())
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            };

            let manager = get_mcp_server_manager();
            match manager.stop_server(&worktree_id).await {
                Ok(()) => {
                    // Status is already set to Stopped by the reducer

                    // Cleanup MCP config file
                    if let Some(path) = config_path {
                        let _ = mcp_config::cleanup_mcp_config_file(&path);
                        // Ignore errors - file may already be deleted
                    }
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetMcpError { error: e });
                }
            }
        }

        Action::LoadJustfileCommands { ref path } => {
            match justfile::parse_justfile(path) {
                Ok(commands) => {
                    let command_data: Vec<actions::JustCommandData> = commands
                        .into_iter()
                        .map(|c| actions::JustCommandData {
                            name: c.name,
                            description: c.description,
                            recipe: c.recipe,
                        })
                        .collect();
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetJustfileCommands { commands: command_data });
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetTasksError { error: Some(e) });
                }
            }
        }

        Action::RunJustCommand { ref name, ref cwd } => {
            match justfile::run_just_command(name, cwd) {
                Ok(output) => {
                    let mut state = get_app_state().write().await;
                    for line in output.lines() {
                        reduce(&mut state, Action::AppendTaskOutput { line: line.to_string() });
                    }
                    reduce(&mut state, Action::SetTaskStatus {
                        name: name.clone(),
                        status: actions::TaskStatusData::Success,
                    });
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::AppendTaskOutput { line: e.clone() });
                    reduce(&mut state, Action::SetTaskStatus {
                        name: name.clone(),
                        status: actions::TaskStatusData::Error,
                    });
                }
            }
        }

        Action::OpenProject { ref path } => {
            // After opening a project, refresh worktrees from git
            refresh_worktrees_for_path(path).await;

            // Check constitution existence for the active worktree
            // Supports both modular (.rstn/constitutions/) and legacy (.rstn/constitution.md)
            let worktree_path = {
                let state = get_app_state().read().await;
                state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone())
            };
            if let Some(wt_path) = worktree_path {
                let exists = constitution::constitution_exists(std::path::Path::new(&wt_path));

                {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetConstitutionExists { exists });
                }
                notify_state_update().await;
            }
        }

        Action::RefreshWorktrees => {
            // Get the active project path and refresh worktrees
            let project_path = {
                let state = get_app_state().read().await;
                state.active_project().map(|p| p.path.clone())
            };
            if let Some(path) = project_path {
                refresh_worktrees_for_path(&path).await;
            }
        }

        Action::AddWorktree { ref branch } => {
            // Get the active project info
            let (project_path, env_config, source_worktree) = {
                let state = get_app_state().read().await;
                if let Some(project) = state.active_project() {
                    let source = project
                        .env_config
                        .source_worktree
                        .clone()
                        .or_else(|| project.worktrees.first().map(|w| w.path.clone()));
                    (
                        Some(project.path.clone()),
                        Some(project.env_config.clone()),
                        source,
                    )
                } else {
                    (None, None, None)
                }
            };

            if let Some(path) = project_path {
                match worktree::add_worktree(&path, branch) {
                    Ok(new_worktree) => {
                        // Refresh worktrees to get the updated list
                        refresh_worktrees_for_path(&path).await;

                        // Auto-copy env files if enabled
                        if let (Some(config), Some(source)) = (env_config, source_worktree) {
                            if config.auto_copy_enabled {
                                let copy_action = Action::CopyEnvFiles {
                                    from_worktree_path: source,
                                    to_worktree_path: new_worktree.path,
                                    patterns: Some(config.tracked_patterns),
                                };
                                // Handle env copy (will add notification)
                                Box::pin(handle_async_action(copy_action)).await.ok();
                            }
                        }
                    }
                    Err(e) => {
                        let mut state = get_app_state().write().await;
                        reduce(
                            &mut state,
                            Action::SetError {
                                code: "WORKTREE_ADD_ERROR".to_string(),
                                message: e,
                                context: Some(format!("AddWorktree: {}", branch)),
                            },
                        );
                    }
                }
            }
        }

        Action::AddWorktreeNewBranch { ref branch } => {
            // Get the active project info
            let (project_path, env_config, source_worktree) = {
                let state = get_app_state().read().await;
                if let Some(project) = state.active_project() {
                    let source = project
                        .env_config
                        .source_worktree
                        .clone()
                        .or_else(|| project.worktrees.first().map(|w| w.path.clone()));
                    (
                        Some(project.path.clone()),
                        Some(project.env_config.clone()),
                        source,
                    )
                } else {
                    (None, None, None)
                }
            };

            if let Some(path) = project_path {
                match worktree::add_worktree_new_branch(&path, branch) {
                    Ok(new_worktree) => {
                        // Refresh worktrees to get the updated list
                        refresh_worktrees_for_path(&path).await;

                        // Auto-copy env files if enabled
                        if let (Some(config), Some(source)) = (env_config, source_worktree) {
                            if config.auto_copy_enabled {
                                let copy_action = Action::CopyEnvFiles {
                                    from_worktree_path: source,
                                    to_worktree_path: new_worktree.path,
                                    patterns: Some(config.tracked_patterns),
                                };
                                // Handle env copy (will add notification)
                                Box::pin(handle_async_action(copy_action)).await.ok();
                            }
                        }
                    }
                    Err(e) => {
                        let mut state = get_app_state().write().await;
                        reduce(
                            &mut state,
                            Action::SetError {
                                code: "WORKTREE_ADD_ERROR".to_string(),
                                message: e,
                                context: Some(format!("AddWorktreeNewBranch: {}", branch)),
                            },
                        );
                    }
                }
            }
        }

        Action::RemoveWorktree { ref worktree_path } => {
            // Get the active project path
            let project_path = {
                let state = get_app_state().read().await;
                state.active_project().map(|p| p.path.clone())
            };

            if let Some(path) = project_path {
                match worktree::remove_worktree(&path, worktree_path) {
                    Ok(()) => {
                        // Refresh worktrees to get the updated list
                        refresh_worktrees_for_path(&path).await;
                    }
                    Err(e) => {
                        let mut state = get_app_state().write().await;
                        reduce(&mut state, Action::SetError {
                            code: "WORKTREE_REMOVE_ERROR".to_string(),
                            message: e,
                            context: Some(format!("RemoveWorktree: {}", worktree_path)),
                        });
                    }
                }
            }
        }

        Action::StartDockerServiceWithPort { ref service_id, port } => {
            // Start service with custom port
            match docker_start_service_with_port(service_id.clone(), port).await {
                Ok(()) => {
                    refresh_docker_services_internal().await;
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_START_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("StartDockerServiceWithPort: {} on port {}", service_id, port)),
                    });
                }
            }
        }

        Action::ResolveConflictByStoppingContainer { ref conflicting_container_id, ref service_id } => {
            // Stop the conflicting container first
            match docker_stop_container(conflicting_container_id.clone()).await {
                Ok(()) => {
                    // Now start the rstn service
                    match docker_start_service(service_id.clone()).await {
                        Ok(()) => {
                            refresh_docker_services_internal().await;
                        }
                        Err(e) => {
                            let mut state = get_app_state().write().await;
                            reduce(&mut state, Action::SetError {
                                code: "DOCKER_START_ERROR".to_string(),
                                message: e.to_string(),
                                context: Some(format!("ResolveConflict: failed to start {}", service_id)),
                            });
                        }
                    }
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetError {
                        code: "DOCKER_STOP_ERROR".to_string(),
                        message: e.to_string(),
                        context: Some(format!("ResolveConflict: failed to stop {}", conflicting_container_id)),
                    });
                }
            }
        }

        Action::CopyEnvFiles {
            ref from_worktree_path,
            ref to_worktree_path,
            ref patterns,
        } => {
            let from = from_worktree_path.clone();
            let to = to_worktree_path.clone();

            // Get patterns from action or fall back to project's tracked_patterns
            let copy_patterns = if let Some(p) = patterns {
                p.clone()
            } else {
                let state = get_app_state().read().await;
                if let Some(project) = state.active_project() {
                    project.env_config.tracked_patterns.clone()
                } else {
                    env::default_patterns()
                }
            };

            match env::copy_env_files(&from, &to, &copy_patterns) {
                Ok(result) => {
                    // Convert to action data type
                    let result_data = actions::EnvCopyResultData {
                        copied_files: result.copied.clone(),
                        failed_files: result.failed.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };

                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetEnvCopyResult { result: result_data });

                    // Add notification based on result
                    let message = if result.is_success() {
                        format!("Copied {} env file(s)", result.copied.len())
                    } else if result.is_partial() {
                        format!(
                            "Copied {} file(s), {} failed",
                            result.copied.len(),
                            result.failed.len()
                        )
                    } else if result.copied.is_empty() && result.failed.is_empty() {
                        "No env files to copy".to_string()
                    } else {
                        format!("Failed to copy {} file(s)", result.failed.len())
                    };

                    let notif_type = if result.is_success() {
                        actions::NotificationTypeData::Success
                    } else if result.is_partial() {
                        actions::NotificationTypeData::Warning
                    } else {
                        actions::NotificationTypeData::Info
                    };

                    reduce(
                        &mut state,
                        Action::AddNotification {
                            message,
                            notification_type: notif_type,
                        },
                    );
                }
                Err(e) => {
                    let mut state = get_app_state().write().await;
                    reduce(
                        &mut state,
                        Action::AddNotification {
                            message: format!("Env copy failed: {}", e),
                            notification_type: actions::NotificationTypeData::Error,
                        },
                    );
                }
            }
        }

        // Synchronous actions - already handled by reduce()
        // Note: StartMcpServer and StopMcpServer are handled async above
        Action::CloseProject { .. }
        | Action::SwitchProject { .. }
        | Action::SetFeatureTab { .. }
        | Action::SwitchWorktree { .. }
        | Action::SetWorktrees { .. }
        | Action::SetMcpStatus { .. }
        | Action::SetMcpPort { .. }
        | Action::SetMcpConfigPath { .. }
        | Action::SetMcpError { .. }
        | Action::SetDockerAvailable { .. }
        | Action::SetDockerServices { .. }
        | Action::SelectDockerService { .. }
        | Action::SetDockerLogs { .. }
        | Action::SetDockerLoading { .. }
        | Action::SetDockerLogsLoading { .. }
        | Action::SetPortConflict { .. }
        | Action::ClearPortConflict
        | Action::SetJustfileCommands { .. }
        | Action::SetTaskStatus { .. }
        | Action::SetActiveCommand { .. }
        | Action::AppendTaskOutput { .. }
        | Action::ClearTaskOutput
        | Action::SetTasksLoading { .. }
        | Action::SetTasksError { .. }
        | Action::SetTheme { .. }
        | Action::SetProjectPath { .. }
        | Action::SetError { .. }
        | Action::ClearError
        // Env actions (sync)
        | Action::SetEnvCopyResult { .. }
        | Action::SetEnvTrackedPatterns { .. }
        | Action::SetEnvAutoCopy { .. }
        | Action::SetEnvSourceWorktree { .. }
        // Notification actions (sync)
        | Action::AddNotification { .. }
        | Action::DismissNotification { .. }
        | Action::MarkNotificationRead { .. }
        | Action::MarkAllNotificationsRead
        | Action::ClearNotifications
        // MCP log actions (sync)
        | Action::AddMcpLogEntry { .. }
        | Action::ClearMcpLogs
        | Action::UpdateMcpTools { .. }
        // Chat actions (sync state updates only)
        | Action::AddChatMessage { .. }
        | Action::AppendChatContent { .. }
        | Action::SetChatTyping { .. }
        | Action::SetChatError { .. }
        | Action::ClearChatError
        | Action::ClearChat
        // Terminal actions (sync - state updates only)
        | Action::SetTerminalSession { .. }
        | Action::SetTerminalSize { .. }
        // View actions (sync)
        | Action::SetActiveView { .. }
        // Dev log actions (sync)
        | Action::AddDevLog { .. }
        | Action::ClearDevLogs => {
            // Already handled synchronously
        }

        // Claude Code CLI chat (async - spawns external process)
        Action::SendChatMessage { ref text } => {
            // Get the working directory, MCP config path, and agent rules config
            let (cwd, mcp_config_path, agent_rules_config, project_id) = {
                let state = get_app_state().read().await;
                let cwd = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| std::path::PathBuf::from(&w.path));
                let config_path = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .and_then(|w| w.mcp.config_path.clone());
                let agent_rules = state
                    .active_project()
                    .map(|p| p.agent_rules_config.clone());
                let proj_id = state
                    .active_project()
                    .map(|p| p.id.clone());
                (cwd, config_path, agent_rules, proj_id)
            };

            let cwd = match cwd {
                Some(path) => path,
                None => {
                    {
                        let mut state = get_app_state().write().await;
                        reduce(
                            &mut state,
                            Action::SetChatError {
                                error: "No active project".to_string(),
                            },
                        );
                        reduce(&mut state, Action::SetChatTyping { is_typing: false });
                    } // Write lock released here
                    notify_state_update().await;
                    return Ok(());
                }
            };

            // Create assistant message placeholder (streaming)
            let msg_id = format!("assistant-{}", chrono::Utc::now().timestamp_millis());
            {
                let mut state = get_app_state().write().await;
                reduce(
                    &mut state,
                    Action::AddChatMessage {
                        message: actions::ChatMessageData {
                            id: msg_id.clone(),
                            role: actions::ChatRoleData::Assistant,
                            content: String::new(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            is_streaming: true,
                        },
                    },
                );
            } // Write lock released here
            notify_state_update().await;

            // Clone values for async task
            let prompt = text.clone();
            let cwd_for_task = cwd.clone();
            let mcp_config_for_task = mcp_config_path.clone();
            let agent_rules_for_task = agent_rules_config.clone();
            let project_id_for_task = project_id.clone();

            // Spawn async task to handle CLI interaction without blocking
            tokio::spawn(async move {
    // Validate Claude CLI exists before attempting spawn
    if let Err(e) = claude_cli::validate_claude_cli().await {
        let error = e.to_string();
        {
            let mut state = get_app_state().write().await;
            reduce(&mut state, Action::SetChatError { error });
            reduce(&mut state, Action::SetChatTyping { is_typing: false });
        }
        notify_state_update().await;
        return;
    }

    // Generate agent rules file if enabled
    let agent_rules_path = if let (Some(config), Some(proj_id)) = (&agent_rules_for_task, &project_id_for_task) {
        if config.enabled {
            // Find the active profile
            let active_profile = config.active_profile_id.as_ref()
                .and_then(|id| {
                    config.profiles.iter().find(|p| &p.id == id)
                });

            if let Some(profile) = active_profile {
                if !profile.prompt.trim().is_empty() {
                    match agent_rules::generate_agent_rules_file(&proj_id, &profile.prompt) {
                        Ok(path) => Some(path),
                        Err(e) => {
                            eprintln!("Failed to generate agent rules file: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Spawn Claude CLI process (with MCP config and/or agent rules if available)
    match claude_cli::spawn_claude(&prompt, &cwd_for_task, mcp_config_for_task.as_deref(), agent_rules_path.as_deref()) {
        Ok(mut child) => {
            // Monitor stderr for diagnostic information (errors logged to console)
            if let Some(stderr) = child.stderr.take() {
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            // Log stderr to console for debugging
                            eprintln!("[Claude CLI stderr] {}", trimmed);
                        }
                    }
                });
            }

            // Create event stream
            match claude_cli::ClaudeEventStream::new(&mut child) {
                Ok(mut stream) => {
                    use std::time::Instant;
                    let start_time = Instant::now();
                    let mut consecutive_other_events = 0;
                    const MAX_CONSECUTIVE_OTHER: u32 = 10;

                    // Event loop with timeout
                    loop {
                        // Check total timeout (5 minutes)
                        if start_time.elapsed() > claude_cli::TOTAL_TIMEOUT {
                            let error = "Request exceeded 5 minute timeout".to_string();
                            {
                                let mut state = get_app_state().write().await;
                                reduce(&mut state, Action::SetChatError { error });
                                reduce(&mut state, Action::SetChatTyping { is_typing: false });
                            }
                            notify_state_update().await;
                            break;
                        }

                        // Read next event with timeout (30s)
                        match tokio::time::timeout(
                            claude_cli::EVENT_TIMEOUT,
                            stream.next_event()
                        ).await {
                            Ok(Some(Ok(event))) => {
                                // Handle unsupported events
                                if matches!(event, claude_cli::ClaudeStreamEvent::Other) {
                                    consecutive_other_events += 1;
                                    if consecutive_other_events >= MAX_CONSECUTIVE_OTHER {
                                        let error = format!("Received {} consecutive unsupported events from Claude CLI", consecutive_other_events);
                                        {
                                            let mut state = get_app_state().write().await;
                                            reduce(&mut state, Action::SetChatError { error });
                                            reduce(&mut state, Action::SetChatTyping { is_typing: false });
                                        }
                                        notify_state_update().await;
                                        break;
                                    }
                                    continue;
                                }

                                // System events are informational, don't count as errors
                                if matches!(event, claude_cli::ClaudeStreamEvent::System { .. }) {
                                    consecutive_other_events = 0;
                                    continue;
                                }

                                // Reset counter when we get a useful event
                                consecutive_other_events = 0;

                                // Process streaming text deltas (Anthropic API format)
                                if let Some(text_chunk) = claude_cli::extract_text_delta(&event) {
                                    let content = text_chunk.to_string();
                                    {
                                        let mut state = get_app_state().write().await;
                                        reduce(&mut state, Action::AppendChatContent { content });
                                    }
                                    notify_state_update().await;
                                }

                                // Process Claude CLI assistant messages (complete message format)
                                if let Some(text_content) = claude_cli::extract_assistant_text(&event) {
                                    {
                                        let mut state = get_app_state().write().await;
                                        reduce(&mut state, Action::AppendChatContent { content: text_content });
                                    }
                                    notify_state_update().await;
                                }

                                // Check for message_stop
                                if claude_cli::is_message_stop(&event) {
                                    {
                                        let mut state = get_app_state().write().await;
                                        reduce(&mut state, Action::SetChatTyping { is_typing: false });
                                    }
                                    notify_state_update().await;
                                    break;
                                }
                            }
                            Ok(Some(Err(e))) => {
                                // Parse error
                                let error = e.to_string();
                                {
                                    let mut state = get_app_state().write().await;
                                    reduce(&mut state, Action::SetChatError { error });
                                    reduce(&mut state, Action::SetChatTyping { is_typing: false });
                                }
                                notify_state_update().await;
                                break;
                            }
                            Ok(None) => {
                                // Stream ended without message_stop - this is an error
                                let error = "Claude CLI ended unexpectedly. Check if you have valid API credentials.".to_string();
                                {
                                    let mut state = get_app_state().write().await;
                                    reduce(&mut state, Action::SetChatError { error });
                                    reduce(&mut state, Action::SetChatTyping { is_typing: false });
                                }
                                notify_state_update().await;
                                break;
                            }
                            Err(_) => {
                                // Timeout - no event received for 30s
                                let error = "No response from Claude CLI for 30 seconds".to_string();
                                {
                                    let mut state = get_app_state().write().await;
                                    reduce(&mut state, Action::SetChatError { error });
                                    reduce(&mut state, Action::SetChatTyping { is_typing: false });
                                }
                                notify_state_update().await;
                                break;
                            }
                        }
                    }

                    // Ensure typing flag is cleared after loop exits
                    {
                        let mut state = get_app_state().write().await;
                        reduce(&mut state, Action::SetChatTyping { is_typing: false });
                    }
                    notify_state_update().await;

                    // Wait for process to finish
                    let _ = child.wait().await;
                }
                Err(e) => {
                    let error = e.to_string();
                    {
                        let mut state = get_app_state().write().await;
                        reduce(&mut state, Action::SetChatError { error });
                        reduce(&mut state, Action::SetChatTyping { is_typing: false });
                    }
                    notify_state_update().await;
                }
            }
        }
        Err(e) => {
            let error = e.to_string();
            {
                let mut state = get_app_state().write().await;
                reduce(&mut state, Action::SetChatError { error });
                reduce(&mut state, Action::SetChatTyping { is_typing: false });
            }
            notify_state_update().await;
        }
    }

    // Cleanup agent rules file if it was created
    if let Some(path) = agent_rules_path {
        if let Err(e) = agent_rules::cleanup_agent_rules_file(&path) {
            eprintln!("Warning: Failed to cleanup agent rules file: {}", e);
        }
    }
});

            // Return immediately - background thread handles streaming
        }

        // Agent Rules actions (sync - handled in reducer)
        Action::SetAgentRulesEnabled { .. }
        | Action::SetAgentRulesPrompt { .. }
        | Action::SetAgentRulesTempFile { .. }
        | Action::CreateAgentProfile { .. }
        | Action::UpdateAgentProfile { .. }
        | Action::DeleteAgentProfile { .. }
        | Action::SelectAgentProfile { .. } => {
            // These are pure state mutations, handled synchronously in reducer
            // No async operations needed
        }

        // Constitution Workflow actions (CESDD Phase 1)
        Action::StartConstitutionWorkflow
        | Action::ClearConstitutionWorkflow
        | Action::AnswerConstitutionQuestion { .. }
        | Action::AppendConstitutionOutput { .. } => {
            // Sync actions - handled in reducer
        }

        Action::GenerateConstitution => {
            // Get workflow state and build prompt
            let (cwd, answers) = {
                let state = get_app_state().read().await;
                let cwd = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| std::path::PathBuf::from(&w.path));
                let answers = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .and_then(|w| w.tasks.constitution_workflow.as_ref())
                    .map(|wf| wf.answers.clone());
                (cwd, answers)
            };

            let cwd = match cwd {
                Some(path) => path,
                None => return Ok(()),
            };

            let answers = match answers {
                Some(ans) => ans,
                None => return Ok(()),
            };

            // Build constitution generation prompt
            let tech_stack = answers.get("tech_stack").cloned().unwrap_or_default();
            let security = answers.get("security").cloned().unwrap_or_default();
            let code_quality = answers.get("code_quality").cloned().unwrap_or_default();
            let architecture = answers.get("architecture").cloned().unwrap_or_default();

            let prompt = format!(
                r#"You are helping create a project Constitution - governance rules for AI development.

User provided the following information:
- Technology Stack: {}
- Security Requirements: {}
- Code Quality Standards: {}
- Architectural Constraints: {}

Generate a comprehensive constitution.md file in Markdown format with these sections:

# Project Constitution

## Technology Stack
{{detailed rules based on tech_stack}}

## Security Requirements
{{detailed rules based on security}}

## Code Quality Standards
{{detailed rules based on code_quality}}

## Architectural Constraints
{{detailed rules based on architecture}}

Be specific, actionable, and authoritative. Use "MUST", "MUST NOT" language."#,
                tech_stack, security, code_quality, architecture
            );

            // Spawn Claude CLI to generate constitution
            let cwd_for_task = cwd.clone();
            tokio::spawn(async move {
                // Validate Claude CLI
                if let Err(e) = claude_cli::validate_claude_cli().await {
                    eprintln!("Claude CLI validation failed: {}", e);
                    return;
                }

                // Build Claude CLI command
                let mut cmd = tokio::process::Command::new("claude");
                cmd.arg("-p")
                    .arg("--verbose")
                    .arg("--output-format")
                    .arg("stream-json")
                    .arg("--include-partial-messages")
                    .arg(&prompt)
                    .current_dir(&cwd_for_task)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());

                match cmd.spawn() {
                    Ok(mut child) => {
                        let stdout = child.stdout.take().expect("Failed to get stdout");
                        let mut reader = tokio::io::BufReader::new(stdout).lines();

                        // Stream output
                        while let Ok(Some(line)) = reader.next_line().await {
                            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                                // Extract content from Claude streaming events
                                if let Some(content_block) = event["content_block"].as_object() {
                                    if let Some(text) = content_block["text"].as_str() {
                                        let mut state = get_app_state().write().await;
                                        reduce(
                                            &mut state,
                                            Action::AppendConstitutionOutput {
                                                content: text.to_string(),
                                            },
                                        );
                                        drop(state);
                                        notify_state_update().await;
                                    }
                                } else if let Some(delta) = event["delta"].as_object() {
                                    if let Some(text) = delta["text"].as_str() {
                                        let mut state = get_app_state().write().await;
                                        reduce(
                                            &mut state,
                                            Action::AppendConstitutionOutput {
                                                content: text.to_string(),
                                            },
                                        );
                                        drop(state);
                                        notify_state_update().await;
                                    }
                                }
                            }
                        }

                        // Wait for process to complete
                        let _ = child.wait().await;

                        // After completion, save the constitution file
                        let (output, worktree_path) = {
                            let state = get_app_state().read().await;
                            let output = state
                                .active_project()
                                .and_then(|p| p.active_worktree())
                                .and_then(|w| w.tasks.constitution_workflow.as_ref())
                                .map(|wf| wf.output.clone());
                            let path = state
                                .active_project()
                                .and_then(|p| p.active_worktree())
                                .map(|w| w.path.clone());
                            (output, path)
                        };

                        if let (Some(content), Some(wt_path)) = (output, worktree_path) {
                            let rstn_dir = std::path::PathBuf::from(&wt_path).join(".rstn");
                            let constitution_file = rstn_dir.join("constitution.md");

                            // Create .rstn directory if it doesn't exist
                            if let Err(e) = tokio::fs::create_dir_all(&rstn_dir).await {
                                eprintln!("Failed to create .rstn directory: {}", e);
                                return;
                            }

                            // Write constitution file
                            if let Err(e) = tokio::fs::write(&constitution_file, content).await {
                                eprintln!("Failed to write constitution.md: {}", e);
                                return;
                            }

                            // Mark workflow as complete
                            {
                                let mut state = get_app_state().write().await;
                                if let Some(project) = state.active_project_mut() {
                                    if let Some(worktree) = project.active_worktree_mut() {
                                        if let Some(workflow) = &mut worktree.tasks.constitution_workflow {
                                            workflow.status = crate::app_state::WorkflowStatus::Complete;
                                        }
                                    }
                                }
                            }
                            notify_state_update().await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to spawn Claude CLI: {}", e);
                    }
                }
            });
        }

        Action::SaveConstitution => {
            // Get output and worktree path
            let (output, worktree_path) = {
                let state = get_app_state().read().await;
                let output = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .and_then(|w| w.tasks.constitution_workflow.as_ref())
                    .map(|wf| wf.output.clone());
                let path = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone());
                (output, path)
            };

            if let (Some(content), Some(wt_path)) = (output, worktree_path) {
                let rstn_dir = std::path::PathBuf::from(&wt_path).join(".rstn");
                let constitution_file = rstn_dir.join("constitution.md");

                // Create .rstn directory if it doesn't exist
                if let Err(e) = tokio::fs::create_dir_all(&rstn_dir).await {
                    eprintln!("Failed to create .rstn directory: {}", e);
                    return Ok(());
                }

                // Write constitution file
                if let Err(e) = tokio::fs::write(&constitution_file, content).await {
                    eprintln!("Failed to write constitution.md: {}", e);
                    return Ok(());
                }

                // Update state to Complete (already done in reducer)
                notify_state_update().await;
            }
        }

        Action::CheckConstitutionExists => {
            // Get the active worktree path
            let worktree_path = {
                let state = get_app_state().read().await;
                state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone())
            };

            if let Some(wt_path) = worktree_path {
                // Check for modular (.rstn/constitutions/) or legacy (.rstn/constitution.md)
                let exists = constitution::constitution_exists(std::path::Path::new(&wt_path));

                {
                    let mut state = get_app_state().write().await;
                    reduce(&mut state, Action::SetConstitutionExists { exists });
                }
                notify_state_update().await;
            }
        }

        Action::SetConstitutionExists { .. } => {
            // Sync action - handled in reducer
        }

        Action::ApplyDefaultConstitution => {
            // Get the active worktree path
            let worktree_path = {
                let state = get_app_state().read().await;
                state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone())
            };

            if let Some(wt_path) = worktree_path {
                let project_path = std::path::Path::new(&wt_path);

                // Create modular constitution with language detection
                match constitution::create_modular_constitution(project_path).await {
                    Ok(()) => {
                        // Update state
                        {
                            let mut state = get_app_state().write().await;
                            reduce(&mut state, Action::SetConstitutionExists { exists: true });
                        }
                        notify_state_update().await;
                    }
                    Err(e) => {
                        eprintln!("Failed to create modular constitution: {}", e);
                    }
                }
            }
        }

        // ====================================================================
        // Change Management Actions (CESDD Phase 2 - async operations)
        // ====================================================================
        Action::CreateChange { intent } => {
            // Get the active worktree path
            let worktree_path = {
                let state = get_app_state().read().await;
                state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone())
            };

            if let Some(wt_path) = worktree_path {
                // Generate change ID and name from intent
                let change_id = format!("change-{}", chrono::Utc::now().timestamp_millis());
                let change_name = slugify(&intent);
                let now = chrono::Utc::now().to_rfc3339();

                // Create change directory: .rstn/changes/<change-name>/
                let changes_dir = std::path::Path::new(&wt_path)
                    .join(".rstn")
                    .join("changes")
                    .join(&change_name);
                if let Err(e) = std::fs::create_dir_all(&changes_dir) {
                    eprintln!("Failed to create changes directory: {}", e);
                    return Ok(());
                }

                // Write intent.md
                let intent_path = changes_dir.join("intent.md");
                if let Err(e) = std::fs::write(&intent_path, &intent) {
                    eprintln!("Failed to write intent.md: {}", e);
                    return Ok(());
                }

                // Create the change in state
                let change = app_state::Change {
                    id: change_id,
                    name: change_name,
                    status: app_state::ChangeStatus::Proposed,
                    intent: intent.clone(),
                    proposal: None,
                    plan: None,
                    streaming_output: String::new(),
                    created_at: now.clone(),
                    updated_at: now,
                };

                {
                    let mut state = get_app_state().write().await;
                    if let Some(project) = state.active_project_mut() {
                        if let Some(worktree) = project.active_worktree_mut() {
                            worktree.changes.changes.push(change);
                            worktree.changes.is_loading = false;
                        }
                    }
                }
                notify_state_update().await;
            }
        }

        Action::GenerateProposal { change_id } => {
            // Reduce first to set status to Planning
            {
                let mut state = get_app_state().write().await;
                reduce(
                    &mut state,
                    Action::GenerateProposal {
                        change_id: change_id.clone(),
                    },
                );
            }
            notify_state_update().await;

            // TODO: Implement Claude CLI streaming for proposal generation
            // For now, just mark as complete with placeholder
            eprintln!("GenerateProposal: Claude CLI integration pending for change {}", change_id);
        }

        Action::AppendProposalOutput { .. } | Action::CompleteProposal { .. } => {
            // Sync actions - handled in reducer
        }

        Action::GeneratePlan { change_id } => {
            // Reduce first to set status to Planning
            {
                let mut state = get_app_state().write().await;
                reduce(
                    &mut state,
                    Action::GeneratePlan {
                        change_id: change_id.clone(),
                    },
                );
            }
            notify_state_update().await;

            // TODO: Implement Claude CLI streaming for plan generation
            eprintln!("GeneratePlan: Claude CLI integration pending for change {}", change_id);
        }

        Action::AppendPlanOutput { .. }
        | Action::CompletePlan { .. }
        | Action::ApprovePlan { .. }
        | Action::CancelChange { .. }
        | Action::SelectChange { .. }
        | Action::SetChangesLoading { .. } => {
            // Sync actions - handled in reducer
        }

        Action::RefreshChanges => {
            // Get the active worktree path
            let worktree_path = {
                let state = get_app_state().read().await;
                state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .map(|w| w.path.clone())
            };

            if let Some(wt_path) = worktree_path {
                let changes_dir = std::path::Path::new(&wt_path)
                    .join(".rstn")
                    .join("changes");

                let mut changes = Vec::new();

                if changes_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&changes_dir) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                let change_name = entry.file_name().to_string_lossy().to_string();
                                let intent_path = entry.path().join("intent.md");
                                let proposal_path = entry.path().join("proposal.md");
                                let plan_path = entry.path().join("plan.md");

                                let intent = std::fs::read_to_string(&intent_path)
                                    .unwrap_or_default();
                                let proposal = std::fs::read_to_string(&proposal_path).ok();
                                let plan = std::fs::read_to_string(&plan_path).ok();

                                // Determine status from files
                                let status = if plan.is_some() {
                                    app_state::ChangeStatus::Planned
                                } else {
                                    // Default to Proposed if no plan yet
                                    app_state::ChangeStatus::Proposed
                                };

                                let now = chrono::Utc::now().to_rfc3339();
                                changes.push(app_state::Change {
                                    id: format!("change-{}", change_name),
                                    name: change_name,
                                    status,
                                    intent,
                                    proposal,
                                    plan,
                                    streaming_output: String::new(),
                                    created_at: now.clone(),
                                    updated_at: now,
                                });
                            }
                        }
                    }
                }

                {
                    let mut state = get_app_state().write().await;
                    if let Some(project) = state.active_project_mut() {
                        if let Some(worktree) = project.active_worktree_mut() {
                            worktree.changes.changes = changes;
                            worktree.changes.is_loading = false;
                        }
                    }
                }
                notify_state_update().await;
            }
        }

        Action::SetChanges { .. } => {
            // Sync action - handled in reducer
        }

        // Terminal actions (async - PTY operations)
        Action::SpawnTerminal { .. }
        | Action::ResizeTerminal { .. }
        | Action::WriteTerminal { .. }
        | Action::KillTerminal { .. } => {
            // TODO: Add terminal manager handling
            // These will be handled by a global terminal manager
        }
    }

    Ok(())
}

/// Convert intent to a URL-friendly slug
fn slugify(intent: &str) -> String {
    intent
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)
        .collect()
}

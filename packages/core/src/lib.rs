//! rstn-core - napi-rs Rust addon for rustation desktop app.
//!
//! Provides Docker management, MCP server, and state management.

#[macro_use]
extern crate napi_derive;

pub mod actions;
pub mod app_state;
pub mod docker;
pub mod justfile;
pub mod persistence;
pub mod reducer;
pub mod state;

use actions::Action;
use app_state::AppState;
use docker::DockerManager;
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use reducer::reduce;
use state::DockerService;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

// Global Docker manager instance
static DOCKER_MANAGER: OnceCell<Arc<DockerManager>> = OnceCell::const_new();

// Global application state
static APP_STATE: OnceCell<Arc<RwLock<AppState>>> = OnceCell::const_new();

// State update listener (callback to JavaScript)
static STATE_LISTENER: OnceCell<ThreadsafeFunction<String>> = OnceCell::const_new();

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
        .map_err(|e| napi::Error::from_reason(e))
}

/// Stop a Docker service
#[napi]
pub async fn docker_stop_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.stop_service(&service_id)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

/// Restart a Docker service
#[napi]
pub async fn docker_restart_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.restart_service(&service_id)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

/// Get container logs
#[napi]
pub async fn docker_get_logs(service_id: String, tail: Option<u32>) -> napi::Result<Vec<String>> {
    let dm = get_docker_manager().await?;
    let tail = tail.unwrap_or(100) as usize;
    dm.get_logs(&service_id, tail)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

/// Remove a Docker service
#[napi]
pub async fn docker_remove_service(service_id: String) -> napi::Result<()> {
    let dm = get_docker_manager().await?;
    dm.remove_service(&service_id)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

/// Create a database in a database container
/// Returns the connection string for the new database
#[napi]
pub async fn docker_create_database(service_id: String, db_name: String) -> napi::Result<String> {
    let dm = get_docker_manager().await?;
    dm.create_database(&service_id, &db_name)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

/// Create a vhost in RabbitMQ
/// Returns the connection string for the new vhost
#[napi]
pub async fn docker_create_vhost(service_id: String, vhost_name: String) -> napi::Result<String> {
    let dm = get_docker_manager().await?;
    dm.create_vhost(&service_id, &vhost_name)
        .await
        .map_err(|e| napi::Error::from_reason(e))
}

// ============================================================================
// Justfile functions
// ============================================================================

/// Parse a justfile and return all commands
#[napi]
pub fn justfile_parse(path: String) -> napi::Result<Vec<justfile::JustCommand>> {
    justfile::parse_justfile(&path)
        .map_err(|e| napi::Error::from_reason(e))
}

/// Run a just command in a directory
#[napi]
pub fn justfile_run(command: String, cwd: String) -> napi::Result<String> {
    justfile::run_just_command(&command, &cwd)
        .map_err(|e| napi::Error::from_reason(e))
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
        tracing::info!("Loaded persisted state with {} recent projects", initial_state.recent_projects.len());
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
            match docker_start_service(service_id.clone()).await {
                Ok(()) => {
                    // Refresh services to get updated status
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

        // Synchronous actions - already handled by reduce()
        Action::OpenProject { .. }
        | Action::CloseProject { .. }
        | Action::SwitchProject { .. }
        | Action::SetFeatureTab { .. }
        | Action::SetDockerAvailable { .. }
        | Action::SetDockerServices { .. }
        | Action::SelectDockerService { .. }
        | Action::SetDockerLogs { .. }
        | Action::SetDockerLoading { .. }
        | Action::SetDockerLogsLoading { .. }
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
        | Action::ClearError => {
            // Already handled synchronously
        }
    }

    Ok(())
}

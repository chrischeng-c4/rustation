//! Tauri commands for rstn.
//!
//! These commands are the interface between the Frontend (React) and
//! the Backend (Rust). The Frontend invokes commands via `invoke()`.

use crate::state::{AppState, Tab};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

/// Type alias for shared app state
pub type SharedState = Arc<RwLock<AppState>>;

// ============================================================================
// State Commands
// ============================================================================

/// Get the current application state
#[tauri::command]
pub async fn get_app_state(state: State<'_, SharedState>) -> Result<AppState, String> {
    let app_state = state.read().await;
    Ok(app_state.clone())
}

/// Set the active tab
#[tauri::command]
pub async fn set_active_tab(tab: Tab, state: State<'_, SharedState>) -> Result<(), String> {
    let mut app_state = state.write().await;
    app_state.active_tab = tab;
    Ok(())
}

// ============================================================================
// Workflow Commands
// ============================================================================

/// Send a prompt to Claude
#[tauri::command]
pub async fn send_prompt(
    prompt: String,
    state: State<'_, SharedState>,
    window: tauri::Window,
) -> Result<(), String> {
    use crate::state::{ChatMessage, MessageRole};

    // Add user message to state
    {
        let mut app_state = state.write().await;
        app_state.workflows.messages.push(ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: prompt.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        app_state.workflows.is_streaming = true;
    }

    // Emit state update to frontend
    let _ = window.emit("state:update", ());

    // TODO: Actually call Claude CLI and stream response
    // For now, just simulate a response
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    {
        let mut app_state = state.write().await;
        app_state.workflows.messages.push(ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: format!("Echo: {}", prompt),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        app_state.workflows.is_streaming = false;
    }

    let _ = window.emit("state:update", ());

    Ok(())
}

// ============================================================================
// Docker Commands
// ============================================================================

/// Toggle a Docker service (start/stop)
#[tauri::command]
pub async fn toggle_docker_service(
    service_id: String,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    use crate::state::ServiceStatus;

    let mut app_state = state.write().await;

    if let Some(service) = app_state
        .dockers
        .services
        .iter_mut()
        .find(|s| s.id == service_id)
    {
        service.status = match service.status {
            ServiceStatus::Running => ServiceStatus::Stopped,
            ServiceStatus::Stopped => ServiceStatus::Starting,
            _ => service.status.clone(),
        };
    }

    // TODO: Actually call docker compose or bollard

    Ok(())
}

/// Get Docker container logs
#[tauri::command]
pub async fn get_container_logs(
    _service_id: String,
    _tail: Option<usize>,
) -> Result<Vec<String>, String> {
    // TODO: Implement actual log fetching
    Ok(vec![
        "[INFO] Container started".to_string(),
        "[INFO] Ready to accept connections".to_string(),
    ])
}

// ============================================================================
// Settings Commands
// ============================================================================

/// Update a setting
#[tauri::command]
pub async fn set_setting(
    key: String,
    value: serde_json::Value,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    use crate::state::Theme;

    let mut app_state = state.write().await;

    match key.as_str() {
        "theme" => {
            if let Some(theme_str) = value.as_str() {
                app_state.settings.theme = match theme_str {
                    "light" => Theme::Light,
                    "dark" => Theme::Dark,
                    _ => Theme::System,
                };
            }
        }
        "default_project_path" => {
            app_state.settings.default_project_path = value.as_str().map(String::from);
        }
        _ => return Err(format!("Unknown setting: {}", key)),
    }

    Ok(())
}

// ============================================================================
// MCP Commands
// ============================================================================

/// Get MCP server info
#[tauri::command]
pub async fn get_mcp_server_info(
    mcp_handle: State<'_, Option<crate::mcp::McpServerHandle>>,
) -> Result<serde_json::Value, String> {
    match mcp_handle.inner() {
        Some(handle) => Ok(serde_json::json!({
            "running": true,
            "port": handle.port(),
            "url": handle.mcp_url(),
        })),
        None => Ok(serde_json::json!({
            "running": false,
        })),
    }
}

//! rstn - rustation v3 Tauri backend library.
//!
//! This library provides the core functionality for the Tauri application,
//! including state management, MCP server, and Tauri commands.

pub mod commands;
pub mod mcp;
pub mod state;

use commands::SharedState;
use mcp::{McpServerConfig, McpServerHandle};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rstn=debug".parse().unwrap())
                .add_directive("tower_http=info".parse().unwrap()),
        )
        .init();

    info!("Starting rstn v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Create shared app state
            let app_state: SharedState = Arc::new(RwLock::new(AppState::default()));

            // Store state in Tauri
            app.manage(app_state.clone());

            // Start MCP server in background
            let mcp_state = app_state.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                match start_mcp_server(mcp_state).await {
                    Ok(handle) => {
                        info!("MCP server started on port {}", handle.port());

                        // Write config for Claude Code discovery
                        if let Err(e) = mcp::write_mcp_config(handle.port()) {
                            tracing::warn!("Failed to write MCP config: {}", e);
                        }

                        // Store handle (we'll manage shutdown later)
                        app_handle.manage(Some(handle));
                    }
                    Err(e) => {
                        tracing::error!("Failed to start MCP server: {}", e);
                        app_handle.manage(None::<McpServerHandle>);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::set_active_tab,
            commands::send_prompt,
            commands::toggle_docker_service,
            commands::get_container_logs,
            commands::set_setting,
            commands::get_mcp_server_info,
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Cleanup MCP config on exit
                if let Err(e) = mcp::cleanup_mcp_config() {
                    tracing::warn!("Failed to cleanup MCP config: {}", e);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Start the MCP server
async fn start_mcp_server(app_state: SharedState) -> anyhow::Result<McpServerHandle> {
    let config = McpServerConfig::default();
    mcp::start_server(config, app_state).await
}

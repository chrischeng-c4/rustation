//! rstn-core - napi-rs Rust addon for rustation desktop app.
//!
//! Provides Docker management, MCP server, and state management.

#[macro_use]
extern crate napi_derive;

pub mod docker;
pub mod state;

use docker::DockerManager;
use state::DockerService;
use std::sync::Arc;
use tokio::sync::OnceCell;

// Global Docker manager instance
static DOCKER_MANAGER: OnceCell<Arc<DockerManager>> = OnceCell::const_new();

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

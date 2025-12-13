//! Service management module (simplified version)

use crate::errors::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

/// Service state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Stopped,
    Running,
    Unknown,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub state: ServiceState,
    pub pid: Option<u32>,
}

/// Check if a service/command is running
pub async fn check_service_running(command_name: &str) -> Result<bool> {
    // Use `pgrep` to check if process is running
    let output = Command::new("pgrep")
        .arg("-x")  // Exact match
        .arg(command_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(false),
    }
}

/// Get process ID for a running service
pub async fn get_service_pid(command_name: &str) -> Result<Option<u32>> {
    let output = Command::new("pgrep")
        .arg("-x")
        .arg(command_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::CommandFailed(format!("pgrep failed: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(pid_str) = stdout.lines().next() {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                return Ok(Some(pid));
            }
        }
    }

    Ok(None)
}

/// List common development services
pub async fn list_services() -> Result<Vec<ServiceInfo>> {
    let common_services = vec![
        ("rust-analyzer", "rust-analyzer"),
        ("cargo-watch", "cargo-watch"),
        ("docker", "docker"),
        ("postgres", "postgres"),
        ("redis-server", "redis-server"),
    ];

    let mut services = Vec::new();

    for (name, command) in common_services {
        let is_running = check_service_running(command).await.unwrap_or(false);
        let pid = if is_running {
            get_service_pid(command).await.unwrap_or(None)
        } else {
            None
        };

        services.push(ServiceInfo {
            name: name.to_string(),
            command: command.to_string(),
            args: vec![],
            state: if is_running {
                ServiceState::Running
            } else {
                ServiceState::Stopped
            },
            pid,
        });
    }

    Ok(services)
}

/// Get status of a specific service
pub async fn get_service_status(name: &str) -> Result<ServiceInfo> {
    let services = list_services().await?;
    services
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| CoreError::ServiceNotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_services() {
        let services = list_services().await.unwrap();
        assert!(!services.is_empty());
    }
}

//! Docker manager using bollard.

use crate::state::{DockerService, ServiceStatus};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::services::{ServiceConfig, BUILTIN_SERVICES};

/// Docker manager for container operations
pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    /// Create a new DockerManager
    pub fn new() -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    /// Check if Docker is available
    pub async fn is_available(&self) -> bool {
        self.docker.ping().await.is_ok()
    }

    /// Get status of all built-in services
    pub async fn list_services(&self) -> Vec<DockerService> {
        let mut services = Vec::new();

        // Get all containers (including stopped)
        let filters: HashMap<String, Vec<String>> = HashMap::new();
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = match self.docker.list_containers(Some(options)).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to list containers: {}", e);
                // Return default stopped services
                return BUILTIN_SERVICES
                    .iter()
                    .map(|config| DockerService {
                        id: config.service.container_name().to_string(),
                        name: config.service.display_name().to_string(),
                        image: config.image.to_string(),
                        status: ServiceStatus::Stopped,
                        port: Some(config.port),
                    })
                    .collect();
            }
        };

        // Build container status map
        let mut container_map: HashMap<String, (ServiceStatus, String)> = HashMap::new();
        for container in containers {
            if let Some(names) = container.names {
                for name in names {
                    // Container names start with /
                    let name = name.trim_start_matches('/');
                    if name.starts_with("rstn-") {
                        let status = match container.state.as_deref() {
                            Some("running") => ServiceStatus::Running,
                            Some("created") | Some("restarting") => ServiceStatus::Starting,
                            Some("exited") | Some("dead") => ServiceStatus::Stopped,
                            _ => ServiceStatus::Stopped,
                        };
                        let image = container.image.clone().unwrap_or_default();
                        container_map.insert(name.to_string(), (status, image));
                    }
                }
            }
        }

        // Build service list from built-in configs
        for config in BUILTIN_SERVICES {
            let container_name = config.service.container_name();
            let (status, image) = container_map
                .get(container_name)
                .cloned()
                .unwrap_or((ServiceStatus::Stopped, config.image.to_string()));

            services.push(DockerService {
                id: container_name.to_string(),
                name: config.service.display_name().to_string(),
                image,
                status,
                port: Some(config.port),
            });
        }

        services
    }

    /// Start a service
    pub async fn start_service(&self, service_id: &str) -> Result<(), String> {
        let config = ServiceConfig::find_by_name(service_id)
            .ok_or_else(|| format!("Unknown service: {}", service_id))?;

        info!("Starting service: {}", service_id);

        // Check if container exists
        let exists = self.container_exists(service_id).await;

        if exists {
            // Start existing container
            self.docker
                .start_container(service_id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| format!("Failed to start container: {}", e))?;
        } else {
            // Pull image and create container
            self.ensure_image(config.image).await?;
            self.create_container(config).await?;

            // Start the new container
            self.docker
                .start_container(service_id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| format!("Failed to start container: {}", e))?;
        }

        info!("Service started: {}", service_id);
        Ok(())
    }

    /// Stop a service
    pub async fn stop_service(&self, service_id: &str) -> Result<(), String> {
        info!("Stopping service: {}", service_id);

        self.docker
            .stop_container(service_id, Some(StopContainerOptions { t: 10 }))
            .await
            .map_err(|e| format!("Failed to stop container: {}", e))?;

        info!("Service stopped: {}", service_id);
        Ok(())
    }

    /// Restart a service
    pub async fn restart_service(&self, service_id: &str) -> Result<(), String> {
        info!("Restarting service: {}", service_id);

        self.docker
            .restart_container(service_id, Some(RestartContainerOptions { t: 10 }))
            .await
            .map_err(|e| format!("Failed to restart container: {}", e))?;

        info!("Service restarted: {}", service_id);
        Ok(())
    }

    /// Get container logs
    pub async fn get_logs(&self, service_id: &str, tail: usize) -> Result<Vec<String>, String> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut logs = Vec::new();
        let mut stream = self.docker.logs(service_id, Some(options));

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let line = match output {
                        LogOutput::StdOut { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        LogOutput::StdErr { message } => {
                            format!("[stderr] {}", String::from_utf8_lossy(&message))
                        }
                        _ => continue,
                    };
                    logs.push(line.trim_end().to_string());
                }
                Err(e) => {
                    error!("Error reading logs: {}", e);
                    break;
                }
            }
        }

        Ok(logs)
    }

    /// Remove a service container
    pub async fn remove_service(&self, service_id: &str) -> Result<(), String> {
        info!("Removing service: {}", service_id);

        // Stop first if running
        let _ = self.stop_service(service_id).await;

        self.docker
            .remove_container(
                service_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| format!("Failed to remove container: {}", e))?;

        info!("Service removed: {}", service_id);
        Ok(())
    }

    // Private helpers

    async fn container_exists(&self, name: &str) -> bool {
        self.docker.inspect_container(name, None).await.is_ok()
    }

    async fn ensure_image(&self, image: &str) -> Result<(), String> {
        debug!("Ensuring image: {}", image);

        // Check if image exists locally
        if self.docker.inspect_image(image).await.is_ok() {
            debug!("Image already exists: {}", image);
            return Ok(());
        }

        info!("Pulling image: {}", image);

        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Pull status: {}", status);
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to pull image: {}", e));
                }
            }
        }

        info!("Image pulled: {}", image);
        Ok(())
    }

    async fn create_container(&self, config: &ServiceConfig) -> Result<(), String> {
        let name = config.service.container_name();
        debug!("Creating container: {}", name);

        // Build environment variables
        let env: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Build port bindings
        let port_key = format!("{}/tcp", config.internal_port);
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            port_key.clone(),
            Some(vec![bollard::service::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(config.port.to_string()),
            }]),
        );

        // Build exposed ports
        let mut exposed_ports = HashMap::new();
        exposed_ports.insert(port_key, HashMap::new());

        let container_config = Config {
            image: Some(config.image.to_string()),
            env: Some(env),
            exposed_ports: Some(exposed_ports),
            host_config: Some(bollard::service::HostConfig {
                port_bindings: Some(port_bindings),
                restart_policy: Some(bollard::service::RestartPolicy {
                    name: Some(bollard::service::RestartPolicyNameEnum::UNLESS_STOPPED),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name,
            platform: None,
        };

        self.docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| format!("Failed to create container: {}", e))?;

        debug!("Container created: {}", name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_manager_creation() {
        // This test will only pass if Docker is installed
        if let Ok(manager) = DockerManager::new() {
            let available = manager.is_available().await;
            println!("Docker available: {}", available);
        }
    }

    #[tokio::test]
    async fn test_docker_integration() {
        // Skip if Docker not available
        let manager = match DockerManager::new() {
            Ok(m) => m,
            Err(e) => {
                println!("Docker not available: {}", e);
                return;
            }
        };

        // 1. Check connection
        let available = manager.is_available().await;
        println!("1. Docker available: {}", available);
        if !available {
            println!("Docker daemon not running, skipping test");
            return;
        }

        // 2. List services
        let services = manager.list_services().await;
        println!("2. Services ({}):", services.len());
        for s in &services {
            println!("   - {} ({:?}) - {}", s.name, s.status, s.image);
        }

        // 3. Start Redis (simplest, no auth needed)
        println!("3. Starting Redis...");
        match manager.start_service("rstn-redis").await {
            Ok(()) => println!("   Redis started successfully"),
            Err(e) => {
                println!("   Failed to start Redis: {}", e);
                return;
            }
        }

        // 4. Check status after start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let services = manager.list_services().await;
        let redis = services.iter().find(|s| s.id == "rstn-redis");
        println!("4. Redis status after start: {:?}", redis.map(|s| &s.status));

        // 5. Get logs
        match manager.get_logs("rstn-redis", 10).await {
            Ok(logs) => {
                println!("5. Redis logs ({} lines):", logs.len());
                for log in &logs {
                    println!("   {}", log);
                }
            }
            Err(e) => println!("5. Failed to get logs: {}", e),
        }

        // 6. Stop Redis
        println!("6. Stopping Redis...");
        match manager.stop_service("rstn-redis").await {
            Ok(()) => println!("   Redis stopped successfully"),
            Err(e) => println!("   Failed to stop Redis: {}", e),
        }

        // 7. Final status
        let services = manager.list_services().await;
        let redis = services.iter().find(|s| s.id == "rstn-redis");
        println!("7. Redis status after stop: {:?}", redis.map(|s| &s.status));
    }
}

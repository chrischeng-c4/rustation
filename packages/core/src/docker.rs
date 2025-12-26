//! Docker container management using bollard.

use crate::state::{DockerService, PortConflictInfo, ServiceType};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
    RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use tracing::{debug, info};

/// Built-in service definitions
pub struct ServiceConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub image: &'static str,
    pub port: u16,
    pub internal_port: u16,
    pub env: &'static [(&'static str, &'static str)],
    pub service_type: ServiceType,
}

pub const BUILTIN_SERVICES: &[ServiceConfig] = &[
    ServiceConfig {
        id: "rstn-postgres",
        name: "PostgreSQL",
        image: "postgres:16-alpine",
        port: 5432,
        internal_port: 5432,
        env: &[("POSTGRES_PASSWORD", "postgres")],
        service_type: ServiceType::Database,
    },
    ServiceConfig {
        id: "rstn-mysql",
        name: "MySQL",
        image: "mysql:8",
        port: 3306,
        internal_port: 3306,
        env: &[("MYSQL_ROOT_PASSWORD", "mysql")],
        service_type: ServiceType::Database,
    },
    ServiceConfig {
        id: "rstn-mongodb",
        name: "MongoDB",
        image: "mongo:7",
        port: 27017,
        internal_port: 27017,
        env: &[],
        service_type: ServiceType::Database,
    },
    ServiceConfig {
        id: "rstn-redis",
        name: "Redis",
        image: "redis:7-alpine",
        port: 6379,
        internal_port: 6379,
        env: &[],
        service_type: ServiceType::Cache,
    },
    ServiceConfig {
        id: "rstn-rabbitmq",
        name: "RabbitMQ",
        image: "rabbitmq:3-management",
        port: 5672,
        internal_port: 5672,
        env: &[],
        service_type: ServiceType::MessageBroker,
    },
    ServiceConfig {
        id: "rstn-nats",
        name: "NATS",
        image: "nats:latest",
        port: 4222,
        internal_port: 4222,
        env: &[],
        service_type: ServiceType::Other,
    },
];

/// Docker manager
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

    /// List all services with their current status
    /// Returns ALL containers on the system, grouped by project prefix
    pub async fn list_services(&self) -> Vec<DockerService> {
        let mut services = Vec::new();

        // Get ALL containers (no filter)
        let all_containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .unwrap_or_default();

        // Track which rstn services are already running
        let mut running_rstn_ids: Vec<String> = Vec::new();

        // Build service list from ALL running containers
        for container in &all_containers {
            let container_name = container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            if container_name.is_empty() {
                continue;
            }

            let is_rstn_managed = container_name.starts_with("rstn-");
            let project_group = Self::detect_project_group(&container_name);

            // Track running rstn services
            if is_rstn_managed {
                running_rstn_ids.push(container_name.clone());
            }

            let status = match container.state.as_deref() {
                Some("running") => "running".to_string(),
                Some("created") | Some("restarting") => "starting".to_string(),
                Some("exited") | Some("dead") => "stopped".to_string(),
                _ => "stopped".to_string(),
            };

            // Get the port from container ports
            let port = container.ports.as_ref().and_then(|ports| {
                ports.first().and_then(|p| p.public_port.map(|pp| pp as u32))
            });

            // Determine service type (best effort for non-rstn containers)
            let service_type = Self::detect_service_type(&container.image.clone().unwrap_or_default());

            services.push(DockerService {
                id: container_name.clone(),
                name: Self::extract_service_name(&container_name),
                image: container.image.clone().unwrap_or_default(),
                status,
                port,
                service_type: format!("{:?}", service_type),
                project_group: Some(project_group),
                is_rstn_managed,
            });
        }

        // Add built-in rstn services that aren't running (for Quick Start)
        for config in BUILTIN_SERVICES {
            if !running_rstn_ids.contains(&config.id.to_string()) {
                services.push(DockerService {
                    id: config.id.to_string(),
                    name: config.name.to_string(),
                    image: config.image.to_string(),
                    status: "stopped".to_string(),
                    port: Some(config.port as u32),
                    service_type: format!("{:?}", config.service_type),
                    project_group: Some("rstn".to_string()),
                    is_rstn_managed: true,
                });
            }
        }

        services
    }

    /// Detect project group from container name
    /// e.g., "tech-platform-postgres" -> "tech-platform"
    /// e.g., "rstn-postgres" -> "rstn"
    /// e.g., "pg-bench" -> "pg-bench" (single segment)
    fn detect_project_group(container_name: &str) -> String {
        let parts: Vec<&str> = container_name.split('-').collect();
        if parts.len() >= 2 {
            // Check if it looks like "prefix-service" pattern
            // For names like "tech-platform-postgres", join first two parts
            // For names like "rstn-postgres", use first part
            if parts.len() >= 3 && !["rstn", "pg"].contains(&parts[0]) {
                // Likely "project-subproject-service" -> "project-subproject"
                parts[..parts.len() - 1].join("-")
            } else {
                // "prefix-service" -> "prefix"
                parts[0].to_string()
            }
        } else {
            // Single word or no hyphens - use as-is
            container_name.to_string()
        }
    }

    /// Extract display name from container name
    /// e.g., "tech-platform-postgres" -> "postgres"
    /// e.g., "rstn-postgres" -> "postgres"
    fn extract_service_name(container_name: &str) -> String {
        // For rstn containers, use the friendly name from config
        if let Some(config) = BUILTIN_SERVICES.iter().find(|c| c.id == container_name) {
            return config.name.to_string();
        }
        // Otherwise extract last part after hyphen
        container_name
            .rsplit('-')
            .next()
            .unwrap_or(container_name)
            .to_string()
    }

    /// Detect service type from image name
    fn detect_service_type(image: &str) -> ServiceType {
        let image_lower = image.to_lowercase();
        if image_lower.contains("postgres") || image_lower.contains("mysql") || image_lower.contains("mongo") {
            ServiceType::Database
        } else if image_lower.contains("redis") || image_lower.contains("dragonfly") {
            ServiceType::Cache
        } else if image_lower.contains("rabbit") || image_lower.contains("nats") {
            ServiceType::MessageBroker
        } else {
            ServiceType::Other
        }
    }

    /// Start a service
    pub async fn start_service(&self, service_id: &str) -> Result<(), String> {
        info!("Starting service: {}", service_id);

        let config = BUILTIN_SERVICES
            .iter()
            .find(|s| s.id == service_id)
            .ok_or_else(|| format!("Unknown service: {}", service_id))?;

        // Ensure image exists
        self.ensure_image(config.image).await?;

        // Check if container already exists
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("name".to_string(), vec![service_id.to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| e.to_string())?;

        if let Some(container) = containers.first() {
            // Container exists, just start it
            if container.state.as_deref() != Some("running") {
                self.docker
                    .start_container(service_id, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        } else {
            // Create and start new container
            debug!("Creating container: {}", service_id);

            let env: Vec<String> = config
                .env
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

            let port_binding = format!("{}/tcp", config.internal_port);
            let host_port = format!("{}", config.port);

            let host_config = HostConfig {
                port_bindings: Some({
                    let mut bindings = HashMap::new();
                    bindings.insert(
                        port_binding.clone(),
                        Some(vec![bollard::models::PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some(host_port),
                        }]),
                    );
                    bindings
                }),
                ..Default::default()
            };

            let container_config = Config {
                image: Some(config.image.to_string()),
                env: Some(env),
                host_config: Some(host_config),
                ..Default::default()
            };

            self.docker
                .create_container(
                    Some(CreateContainerOptions {
                        name: service_id,
                        platform: None,
                    }),
                    container_config,
                )
                .await
                .map_err(|e| e.to_string())?;

            debug!("Container created: {}", service_id);

            self.docker
                .start_container(service_id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;

        info!("Service stopped: {}", service_id);
        Ok(())
    }

    /// Restart a service
    pub async fn restart_service(&self, service_id: &str) -> Result<(), String> {
        info!("Restarting service: {}", service_id);

        self.docker
            .restart_container(service_id, Some(RestartContainerOptions { t: 10 }))
            .await
            .map_err(|e| e.to_string())?;

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

        let mut logs_stream = self.docker.logs(service_id, Some(options));
        let mut logs = Vec::new();

        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(log) => logs.push(log.to_string()),
                Err(e) => return Err(e.to_string()),
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
            .map_err(|e| e.to_string())?;

        info!("Service removed: {}", service_id);
        Ok(())
    }

    /// Create a database in a database container
    pub async fn create_database(&self, service_id: &str, db_name: &str) -> Result<String, String> {
        info!("Creating database '{}' in service: {}", db_name, service_id);

        // Validate db_name (alphanumeric and underscores only)
        if !db_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Database name must contain only alphanumeric characters and underscores".to_string());
        }

        // Create SQL command string (must live longer than cmd vec)
        let sql_cmd = format!("CREATE DATABASE {}", db_name);

        let cmd: Vec<&str> = match service_id {
            "rstn-postgres" => {
                vec!["psql", "-U", "postgres", "-c", &sql_cmd]
            }
            "rstn-mysql" => {
                vec!["mysql", "-u", "root", "-pmysql", "-e", &sql_cmd]
            }
            "rstn-mongodb" => {
                // MongoDB creates databases on first use, just return success
                info!("MongoDB databases are created on first use");
                return Ok(format!("mongodb://localhost:27017/{}", db_name));
            }
            _ => return Err(format!("Service {} does not support database creation", service_id)),
        };

        self.exec_in_container(service_id, &cmd).await?;

        // Return connection string
        let connection_string = match service_id {
            "rstn-postgres" => format!("postgresql://postgres:postgres@localhost:5432/{}", db_name),
            "rstn-mysql" => format!("mysql://root:mysql@localhost:3306/{}", db_name),
            _ => String::new(),
        };

        info!("Database '{}' created successfully", db_name);
        Ok(connection_string)
    }

    /// Create a vhost in RabbitMQ
    pub async fn create_vhost(&self, service_id: &str, vhost_name: &str) -> Result<String, String> {
        info!("Creating vhost '{}' in service: {}", vhost_name, service_id);

        if service_id != "rstn-rabbitmq" {
            return Err(format!("Service {} does not support vhost creation", service_id));
        }

        // Validate vhost_name (alphanumeric, underscores, hyphens)
        if !vhost_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Vhost name must contain only alphanumeric characters, underscores, and hyphens".to_string());
        }

        let cmd = vec!["rabbitmqctl", "add_vhost", vhost_name];
        self.exec_in_container(service_id, &cmd).await?;

        // Return connection string with vhost
        let connection_string = format!("amqp://guest:guest@localhost:5672/{}", vhost_name);

        info!("Vhost '{}' created successfully", vhost_name);
        Ok(connection_string)
    }

    /// Execute a command in a container
    async fn exec_in_container(&self, container_id: &str, cmd: &[&str]) -> Result<String, String> {
        debug!("Executing in container {}: {:?}", container_id, cmd);

        let exec = self.docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| format!("Failed to create exec: {}", e))?;

        let output = self.docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| format!("Failed to start exec: {}", e))?;

        let mut result = String::new();
        if let StartExecResults::Attached { mut output, .. } = output {
            while let Some(msg) = output.next().await {
                match msg {
                    Ok(log) => result.push_str(&log.to_string()),
                    Err(e) => return Err(format!("Exec error: {}", e)),
                }
            }
        }

        // Check exec exit code
        let inspect = self.docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| format!("Failed to inspect exec: {}", e))?;

        if let Some(exit_code) = inspect.exit_code {
            if exit_code != 0 {
                return Err(format!("Command failed with exit code {}: {}", exit_code, result));
            }
        }

        Ok(result)
    }

    /// Ensure an image is available locally
    async fn ensure_image(&self, image: &str) -> Result<(), String> {
        debug!("Ensuring image: {}", image);

        // Check if image exists
        if self.docker.inspect_image(image).await.is_ok() {
            debug!("Image already exists: {}", image);
            return Ok(());
        }

        // Pull image
        info!("Pulling image: {}", image);

        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed to pull image: {}", e)),
            }
        }

        info!("Image pulled: {}", image);
        Ok(())
    }

    /// Start a service with a specific port override
    pub async fn start_service_with_port(&self, service_id: &str, port: u16) -> Result<(), String> {
        info!("Starting service {} with port override: {}", service_id, port);

        let config = BUILTIN_SERVICES
            .iter()
            .find(|s| s.id == service_id)
            .ok_or_else(|| format!("Unknown service: {}", service_id))?;

        // Remove existing container if any (to apply new port)
        let _ = self.remove_service(service_id).await;

        // Ensure image exists
        self.ensure_image(config.image).await?;

        // Create container with custom port
        let env: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let port_binding = format!("{}/tcp", config.internal_port);
        let host_port = format!("{}", port);

        let host_config = HostConfig {
            port_bindings: Some({
                let mut bindings = HashMap::new();
                bindings.insert(
                    port_binding.clone(),
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(host_port),
                    }]),
                );
                bindings
            }),
            ..Default::default()
        };

        let container_config = Config {
            image: Some(config.image.to_string()),
            env: Some(env),
            host_config: Some(host_config),
            ..Default::default()
        };

        self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: service_id,
                    platform: None,
                }),
                container_config,
            )
            .await
            .map_err(|e| e.to_string())?;

        self.docker
            .start_container(service_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| e.to_string())?;

        info!("Service started with custom port: {} on port {}", service_id, port);
        Ok(())
    }

    /// Stop any container by ID or name (not just rstn-* containers)
    pub async fn stop_container(&self, container_id: &str) -> Result<(), String> {
        info!("Stopping container: {}", container_id);

        self.docker
            .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
            .await
            .map_err(|e| e.to_string())?;

        info!("Container stopped: {}", container_id);
        Ok(())
    }

    /// Check for port conflict before starting a service
    /// Returns None if no conflict, Some(PortConflictInfo) if port is in use
    pub async fn check_port_conflict(&self, service_id: &str) -> Result<Option<PortConflictInfo>, String> {
        let config = BUILTIN_SERVICES
            .iter()
            .find(|s| s.id == service_id)
            .ok_or_else(|| format!("Unknown service: {}", service_id))?;

        let target_port = config.port;

        // List all running containers
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: false, // Only running containers
                ..Default::default()
            }))
            .await
            .map_err(|e| e.to_string())?;

        // Check if any container is using this port
        for container in containers {
            if let Some(ports) = &container.ports {
                for port_info in ports {
                    if port_info.public_port == Some(target_port) {
                        // Found a conflict!
                        let container_id = container.id.clone().unwrap_or_default();
                        let container_name = container
                            .names
                            .as_ref()
                            .and_then(|n| n.first())
                            .map(|n| n.trim_start_matches('/').to_string())
                            .unwrap_or_default();
                        let is_rstn_managed = container_name.starts_with("rstn-");

                        // Find next available port
                        let suggested_port = self.find_next_available_port(target_port).await;

                        return Ok(Some(PortConflictInfo {
                            requested_port: target_port as u32,
                            container_id,
                            container_name,
                            container_image: container.image.clone().unwrap_or_default(),
                            is_rstn_managed,
                            suggested_port: suggested_port as u32,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find the next available port starting from a base port
    async fn find_next_available_port(&self, base_port: u16) -> u16 {
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: false,
                ..Default::default()
            }))
            .await
            .unwrap_or_default();

        // Collect all used ports
        let mut used_ports: Vec<u16> = Vec::new();
        for container in &containers {
            if let Some(ports) = &container.ports {
                for port_info in ports {
                    if let Some(public_port) = port_info.public_port {
                        used_ports.push(public_port);
                    }
                }
            }
        }

        // Find next available port
        let mut port = base_port + 1;
        while used_ports.contains(&port) && port < 65535 {
            port += 1;
        }

        port
    }
}

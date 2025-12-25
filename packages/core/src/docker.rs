//! Docker container management using bollard.

use crate::state::{DockerService, ServiceType};
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
    pub async fn list_services(&self) -> Vec<DockerService> {
        let mut services = Vec::new();

        // Get running containers
        let running_containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("name".to_string(), vec!["rstn-".to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .unwrap_or_default();

        // Build service list from built-in definitions
        for config in BUILTIN_SERVICES {
            let container = running_containers
                .iter()
                .find(|c| c.names.as_ref().map_or(false, |n| n.iter().any(|name| name.contains(config.id))));

            let status = match container {
                Some(c) => match c.state.as_deref() {
                    Some("running") => "running".to_string(),
                    Some("created") | Some("restarting") => "starting".to_string(),
                    Some("exited") | Some("dead") => "stopped".to_string(),
                    _ => "stopped".to_string(),
                },
                None => "stopped".to_string(),
            };

            services.push(DockerService {
                id: config.id.to_string(),
                name: config.name.to_string(),
                image: config.image.to_string(),
                status,
                port: Some(config.port as u32),
                service_type: format!("{:?}", config.service_type),
            });
        }

        services
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
}

//! Docker container management using bollard.
//!
//! Provides management for built-in development services:
//! - PostgreSQL, MySQL, MongoDB (databases)
//! - Redis (cache)
//! - RabbitMQ, NATS (messaging)

mod manager;
mod services;

pub use manager::DockerManager;
pub use services::{BuiltinService, ServiceConfig, BUILTIN_SERVICES};

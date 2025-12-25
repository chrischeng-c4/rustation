//! State types for rstn.
//!
//! All types are serializable for FFI with napi-rs.

use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Service status
#[napi(string_enum)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ServiceStatus {
    Running,
    #[default]
    Stopped,
    Starting,
    Error,
}

/// Service type - determines what features are available
#[napi(string_enum)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ServiceType {
    /// Database services (PostgreSQL, MySQL, MongoDB) - can create databases
    Database,
    /// Message brokers (RabbitMQ) - can create vhosts
    MessageBroker,
    /// Cache services (Redis) - no extra features
    Cache,
    /// Other services (NATS, etc.) - no extra features
    #[default]
    Other,
}

/// Docker service
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerService {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,  // napi string_enum exports as string
    pub port: Option<u32>,
    pub service_type: String, // ServiceType as string for napi
}

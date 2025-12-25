//! Built-in Docker service definitions.

use serde::{Deserialize, Serialize};

/// Built-in service type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinService {
    Postgres,
    Mysql,
    Mongodb,
    Redis,
    Rabbitmq,
    Nats,
}

impl BuiltinService {
    /// Get the container name (prefixed with rstn-)
    pub fn container_name(&self) -> &'static str {
        match self {
            Self::Postgres => "rstn-postgres",
            Self::Mysql => "rstn-mysql",
            Self::Mongodb => "rstn-mongodb",
            Self::Redis => "rstn-redis",
            Self::Rabbitmq => "rstn-rabbitmq",
            Self::Nats => "rstn-nats",
        }
    }

    /// Get the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Postgres => "PostgreSQL",
            Self::Mysql => "MySQL",
            Self::Mongodb => "MongoDB",
            Self::Redis => "Redis",
            Self::Rabbitmq => "RabbitMQ",
            Self::Nats => "NATS",
        }
    }

    /// Get all built-in services
    pub fn all() -> &'static [BuiltinService] {
        &[
            Self::Postgres,
            Self::Mysql,
            Self::Mongodb,
            Self::Redis,
            Self::Rabbitmq,
            Self::Nats,
        ]
    }
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service: BuiltinService,
    pub image: &'static str,
    pub port: u16,
    pub internal_port: u16,
    pub env: &'static [(&'static str, &'static str)],
}

/// Built-in service configurations
pub const BUILTIN_SERVICES: &[ServiceConfig] = &[
    ServiceConfig {
        service: BuiltinService::Postgres,
        image: "postgres:16-alpine",
        port: 5432,
        internal_port: 5432,
        env: &[
            ("POSTGRES_USER", "rstn"),
            ("POSTGRES_PASSWORD", "rstn"),
            ("POSTGRES_DB", "rstn"),
        ],
    },
    ServiceConfig {
        service: BuiltinService::Mysql,
        image: "mysql:8",
        port: 3306,
        internal_port: 3306,
        env: &[
            ("MYSQL_ROOT_PASSWORD", "rstn"),
            ("MYSQL_DATABASE", "rstn"),
            ("MYSQL_USER", "rstn"),
            ("MYSQL_PASSWORD", "rstn"),
        ],
    },
    ServiceConfig {
        service: BuiltinService::Mongodb,
        image: "mongo:7",
        port: 27017,
        internal_port: 27017,
        env: &[
            ("MONGO_INITDB_ROOT_USERNAME", "rstn"),
            ("MONGO_INITDB_ROOT_PASSWORD", "rstn"),
        ],
    },
    ServiceConfig {
        service: BuiltinService::Redis,
        image: "redis:7-alpine",
        port: 6379,
        internal_port: 6379,
        env: &[],
    },
    ServiceConfig {
        service: BuiltinService::Rabbitmq,
        image: "rabbitmq:3-management",
        port: 5672,
        internal_port: 5672,
        env: &[
            ("RABBITMQ_DEFAULT_USER", "rstn"),
            ("RABBITMQ_DEFAULT_PASS", "rstn"),
        ],
    },
    ServiceConfig {
        service: BuiltinService::Nats,
        image: "nats:latest",
        port: 4222,
        internal_port: 4222,
        env: &[],
    },
];

impl ServiceConfig {
    /// Find config by service type
    pub fn find(service: BuiltinService) -> Option<&'static ServiceConfig> {
        BUILTIN_SERVICES.iter().find(|c| c.service == service)
    }

    /// Find config by container name
    pub fn find_by_name(name: &str) -> Option<&'static ServiceConfig> {
        BUILTIN_SERVICES
            .iter()
            .find(|c| c.service.container_name() == name)
    }
}

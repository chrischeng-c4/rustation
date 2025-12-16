//! MCP (Model Context Protocol) configuration management

use crate::errors::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// MCP server definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// MCP registry structure (from .claude/mcp-registry.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRegistry {
    pub version: String,
    pub servers: Vec<McpServer>,
    #[serde(default)]
    pub component_mappings: HashMap<String, Vec<String>>,
}

/// MCP configuration for .mcp.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// MCP server configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// Load MCP registry from file
pub async fn load_registry<P: AsRef<Path>>(path: P) -> Result<McpRegistry> {
    let content = tokio::fs::read_to_string(path.as_ref())
        .await
        .map_err(|e| CoreError::Config(format!("Failed to read MCP registry: {}", e)))?;

    serde_json::from_str(&content).map_err(CoreError::Json)
}

/// Generate .mcp.json config from registry
pub fn generate_mcp_config(registry: &McpRegistry, component: Option<&str>) -> Result<McpConfig> {
    let mut mcp_servers = HashMap::new();

    // Determine which servers to include
    let server_names: Vec<String> = if let Some(comp) = component {
        // Get servers for specific component
        registry
            .component_mappings
            .get(comp)
            .cloned()
            .unwrap_or_else(|| {
                // If component not found, include all servers
                registry.servers.iter().map(|s| s.name.clone()).collect()
            })
    } else {
        // Include all servers
        registry.servers.iter().map(|s| s.name.clone()).collect()
    };

    // Build config for selected servers
    for server in &registry.servers {
        if server_names.contains(&server.name) {
            mcp_servers.insert(
                server.name.clone(),
                McpServerConfig {
                    command: server.command.clone(),
                    args: server.args.clone(),
                    env: server.env.clone(),
                },
            );
        }
    }

    Ok(McpConfig { mcp_servers })
}

/// Write MCP config to file
pub async fn write_mcp_config<P: AsRef<Path>>(config: &McpConfig, path: P) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    tokio::fs::write(path.as_ref(), json)
        .await
        .map_err(CoreError::Io)?;
    Ok(())
}

/// Find default MCP registry location
pub async fn find_registry_path() -> Result<PathBuf> {
    // Try .claude/mcp-registry.json in current directory
    let local_path = PathBuf::from(".claude/mcp-registry.json");
    if tokio::fs::metadata(&local_path).await.is_ok() {
        return Ok(local_path);
    }

    // Try in git root
    if let Ok(output) = tokio::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .await
    {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout);
            let root_path = PathBuf::from(root.trim()).join(".claude/mcp-registry.json");
            if tokio::fs::metadata(&root_path).await.is_ok() {
                return Ok(root_path);
            }
        }
    }

    Err(CoreError::Config(
        "MCP registry not found. Expected at .claude/mcp-registry.json".into(),
    ))
}

/// Validate MCP configuration
pub fn validate_mcp_config(config: &McpConfig) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    if config.mcp_servers.is_empty() {
        warnings.push("No MCP servers configured".to_string());
    }

    for (name, server) in &config.mcp_servers {
        if server.command.is_empty() {
            warnings.push(format!("Server '{}' has empty command", name));
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mcp_config() {
        let registry = McpRegistry {
            version: "1.0".to_string(),
            servers: vec![
                McpServer {
                    name: "filesystem".to_string(),
                    command: "npx".to_string(),
                    args: vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                    ],
                    description: Some("File system access".to_string()),
                    env: None,
                },
                McpServer {
                    name: "git".to_string(),
                    command: "npx".to_string(),
                    args: vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-git".to_string(),
                    ],
                    description: Some("Git operations".to_string()),
                    env: None,
                },
            ],
            component_mappings: HashMap::new(),
        };

        let config = generate_mcp_config(&registry, None).unwrap();
        assert_eq!(config.mcp_servers.len(), 2);
        assert!(config.mcp_servers.contains_key("filesystem"));
        assert!(config.mcp_servers.contains_key("git"));
    }

    #[test]
    fn test_validate_mcp_config() {
        let config = McpConfig {
            mcp_servers: HashMap::new(),
        };

        let warnings = validate_mcp_config(&config).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("No MCP servers"));
    }
}

//! MCP (Model Context Protocol) module for rstn.
//!
//! Provides an embedded HTTP server for Claude Code integration.

pub mod server;
pub mod types;

pub use server::{cleanup_mcp_config, start_server, write_mcp_config, McpServerHandle};
pub use types::{McpServerConfig, McpStatus};

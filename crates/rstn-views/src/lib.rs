//! rstn-views - Feature views for rustation
//!
//! This crate contains all feature page implementations using GPUI.
//! Each view corresponds to a tab in the main application.

#![recursion_limit = "512"]

pub mod chat;
pub mod dockers;
pub mod explorer;
pub mod mcp;
pub mod settings;
pub mod tasks;
pub mod terminal;
pub mod workflows;

// Re-export key types
pub use chat::ChatView;
pub use dockers::DockersView;
pub use explorer::ExplorerView;
pub use mcp::McpView;
pub use settings::SettingsView;
pub use tasks::TasksView;
pub use terminal::TerminalView;
pub use workflows::WorkflowsView;

//! TUI module for rscli
//!
//! Provides a full-featured terminal user interface with:
//! - Dashboard view showing project status
//! - Menu-driven command navigation
//! - Interactive command execution with live output

pub mod app;
pub mod claude_stream;
pub mod event;
pub mod logging;
pub mod mcp_server;
pub mod mini_dialog; // NEW: Mini TUI dialog for CLI mode MCP interaction
pub mod paste;
pub mod protocol;
pub mod state; // NEW: State management (Feature 079)
pub mod views;
pub mod widgets;

pub use app::{App, AppResult};

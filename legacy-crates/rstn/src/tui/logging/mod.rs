//! Logging infrastructure for TUI event tracking
//!
//! This module provides types and data structures for comprehensive activity
//! logging in the TUI, including timestamped log entries, circular buffer
//! management, and file change tracking.

mod buffer;
mod entry;
mod file_tracker;

pub use buffer::LogBuffer;
pub use entry::{LogCategory, LogEntry};
pub use file_tracker::FileChangeTracker;

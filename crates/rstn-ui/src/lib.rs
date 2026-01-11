//! rstn-ui - UI component library for rustation
//!
//! This crate provides reusable GPUI components following Material Design 3 principles.
//! All components are designed for the rustation developer workbench.

#![recursion_limit = "512"]

pub mod components;
pub mod theme;

// Re-export key types
pub use components::{EmptyState, NavItem, PageHeader, ShellLayout, Sidebar};
pub use theme::{MaterialTheme, Themed};

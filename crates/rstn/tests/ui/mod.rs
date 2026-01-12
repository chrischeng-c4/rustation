//! UI Integration Test Suite
//!
//! This module contains UI integration tests for all views in the GPUI application.
//!
//! ## Test Structure
//!
//! - `tasks_view_test.rs` - TasksView rendering and state integration
//! - `dockers_view_test.rs` - DockersView rendering and service display
//!
//! ## Running Tests
//!
//! **Requirements**:
//! - Full Xcode installation (15.4+)
//! - Metal toolchain configured
//!
//! ```bash
//! # Run all UI tests
//! cargo test --test '*' --features gpui/test-support
//!
//! # Run specific test file
//! cargo test --test tasks_view_test
//! ```
//!
//! ## Known Limitations
//!
//! These tests **cannot run without Xcode** due to Metal shader compilation.
//! See `openspec/UI_TESTING_PLAN.md` for:
//! - Workarounds (GitHub Actions CI)
//! - Alternative testing strategies
//! - What can be tested without Xcode
//!
//! ## Test Status
//!
//! - ✅ Test structure created
//! - ✅ Example tests written
//! - ❌ Cannot execute (blocked by Metal/Xcode)
//!
//! Last Updated: 2026-01-12

mod tasks_view_test;
mod dockers_view_test;

// TODO: Add remaining view tests
// mod explorer_view_test;
// mod terminal_view_test;
// mod chat_view_test;
// mod workflows_view_test;
// mod mcp_view_test;
// mod settings_view_test;

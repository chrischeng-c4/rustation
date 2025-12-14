//! E2E tests entry point
//!
//! This file serves as the entry point for E2E tests using ratatui's TestBackend.

mod e2e_tests;

// Re-export tests from submodules
pub use e2e_tests::sdd_workflow_e2e::*;

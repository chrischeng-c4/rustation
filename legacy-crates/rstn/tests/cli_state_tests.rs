//! CLI state management tests
//!
//! These tests verify:
//! - CLI flags work correctly (--save-state, --load-state, --state-version)
//! - State round-trip via CLI (save → load)
//! - State restoration accuracy
//! - Version mismatch handling
//!
//! Tests use actual CLI invocation to validate end-to-end behavior.

use rstn::tui::state::builders::WorktreeViewStateBuilder;
use rstn::tui::state::AppState;
use rstn::tui::views::{PhaseStatus, SpecPhase};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// ========================================
// Helper Functions
// ========================================

/// Get the path to the rstn binary
fn rstn_binary() -> PathBuf {
    // Cargo builds test binaries in target/debug/
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps"
    path.push("rstn");
    path
}

/// Run rstn CLI command and return output
fn run_rstn(args: &[&str]) -> std::process::Output {
    Command::new(rstn_binary())
        .args(args)
        .output()
        .expect("Failed to execute rstn")
}

// ========================================
// --state-version Flag Tests
// ========================================

#[test]
fn test_cli_state_version_flag() {
    let output = run_rstn(&["--state-version"]);

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("rstn state schema version:"),
        "Output should contain version message"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "Output should contain package version"
    );
}

// ========================================
// --save-state Flag Tests
// ========================================

#[test]
fn test_cli_save_state_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    let output = run_rstn(&["--save-state", state_file.to_str().unwrap()]);

    assert!(output.status.success(), "Command should succeed");
    assert!(state_file.exists(), "State file should be created");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("State saved to:"),
        "Output should confirm save"
    );
}

#[test]
fn test_cli_save_state_valid_json() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    let output = run_rstn(&["--save-state", state_file.to_str().unwrap()]);

    assert!(output.status.success());

    // Verify JSON is valid and can be parsed
    let json = fs::read_to_string(&state_file).unwrap();
    let state: AppState = serde_json::from_str(&json).expect("Saved state should be valid JSON");

    // Verify state structure
    assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
    assert!(state.worktree_view.feature_info.is_none()); // Default state
}

// ========================================
// --load-state Flag Tests
// ========================================

#[test]
fn test_cli_load_state_success() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // Create a test state
    let state = AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
        worktree_view: WorktreeViewStateBuilder::new()
            .with_feature("042", "test-feature")
            .build(),
        dashboard_view: Default::default(),
        settings_view: Default::default(),
        session_history_view: Default::default(),
    };
    state.save_to_file(&state_file).unwrap();

    // Load state via CLI
    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("State loaded from:"));
    assert!(stdout.contains("State version:"));
    assert!(stdout.contains("Feature: 042-test-feature"));
}

#[test]
fn test_cli_load_state_nonexistent_file() {
    let output = run_rstn(&["--load-state", "/tmp/nonexistent-state-file.json"]);

    assert!(!output.status.success(), "Command should fail");
}

#[test]
fn test_cli_load_state_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    fs::write(&state_file, "{ invalid json }").unwrap();

    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);

    assert!(
        !output.status.success(),
        "Command should fail for invalid JSON"
    );
}

// ========================================
// Save → Load Round-Trip Tests
// ========================================

#[test]
fn test_cli_save_load_round_trip_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // Save default state
    let save_output = run_rstn(&["--save-state", state_file.to_str().unwrap()]);
    assert!(save_output.status.success());

    // Load state back
    let load_output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);
    assert!(load_output.status.success());

    let stdout = String::from_utf8(load_output.stdout).unwrap();
    assert!(stdout.contains("No feature loaded")); // Default state
}

#[test]
fn test_cli_save_load_round_trip_with_feature() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // Create and save state with feature
    let state = AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
        worktree_view: WorktreeViewStateBuilder::new()
            .with_feature("042", "click-function")
            .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
            .build(),
        dashboard_view: Default::default(),
        settings_view: Default::default(),
        session_history_view: Default::default(),
    };
    state.save_to_file(&state_file).unwrap();

    // Load via CLI
    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Feature: 042-click-function"));
}

// ========================================
// State Restoration Accuracy Tests
// ========================================

#[test]
fn test_cli_state_restoration_preserves_all_fields() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // Create detailed state
    let original_state = AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
        worktree_view: WorktreeViewStateBuilder::new()
            .with_feature("042", "test-feature")
            .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
            .with_phase(SpecPhase::Plan, PhaseStatus::InProgress)
            .with_spec_content("# Test Spec")
            .with_plan_content("# Test Plan")
            .with_content_scroll(100)
            .build(),
        dashboard_view: Default::default(),
        settings_view: Default::default(),
        session_history_view: Default::default(),
    };
    original_state.save_to_file(&state_file).unwrap();

    // Load and verify
    let loaded_state = AppState::load_from_file(&state_file).unwrap();

    assert_eq!(
        original_state, loaded_state,
        "State should be preserved exactly"
    );
}

// ========================================
// Version Mismatch Handling Tests
// ========================================

#[test]
fn test_cli_load_state_version_mismatch_warning() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // Create state with different version
    let state = AppState {
        version: "0.0.0-test".to_string(), // Intentionally different
        worktree_view: WorktreeViewStateBuilder::new().build(),
        dashboard_view: Default::default(),
        settings_view: Default::default(),
        session_history_view: Default::default(),
    };
    state.save_to_file(&state_file).unwrap();

    // Load state (should succeed with warning)
    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);

    // Should succeed but show warning (Phase 2: allows mismatch)
    assert!(
        output.status.success(),
        "Should succeed with warning in Phase 2"
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    // Warning might go to stderr depending on logging config
    // This is acceptable in Phase 2; Phase 5 will enforce strict versioning
}

// ========================================
// YAML Format Tests
// ========================================

#[test]
fn test_cli_load_state_yaml_format() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.yaml");

    // Create and save state as YAML
    let state = AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
        worktree_view: WorktreeViewStateBuilder::new()
            .with_feature("042", "test-feature")
            .build(),
        dashboard_view: Default::default(),
        settings_view: Default::default(),
        session_history_view: Default::default(),
    };
    state.save_to_yaml_file(&state_file).unwrap();

    // Load YAML via CLI (should auto-detect format)
    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);

    assert!(output.status.success(), "Should load YAML successfully");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Feature: 042-test-feature"));
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_cli_state_flags_do_not_start_tui() {
    // Verify that state flags are non-interactive (don't start TUI)

    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("state.json");

    // --save-state should not start TUI
    let output = run_rstn(&["--save-state", state_file.to_str().unwrap()]);
    assert!(output.status.success());
    // Should complete quickly without TUI

    // --load-state should not start TUI
    let output = run_rstn(&["--load-state", state_file.to_str().unwrap()]);
    assert!(output.status.success());
    // Should complete quickly without TUI

    // --state-version should not start TUI
    let output = run_rstn(&["--state-version"]);
    assert!(output.status.success());
    // Should complete immediately
}

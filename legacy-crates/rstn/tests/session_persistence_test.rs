//! Session persistence integration test
//!
//! Tests the complete save/load cycle for session persistence

use rstn::tui::state::dashboard::DashboardState;
use rstn::tui::state::settings::SettingsState;
use rstn::tui::state::worktree::WorktreeViewState;
use rstn::tui::state::AppState;
use std::path::PathBuf;

/// Test complete session persistence cycle
#[test]
fn test_session_save_load_cycle() {
    let temp_dir = std::env::temp_dir();
    let session_file = temp_dir.join("test_session_persistence.yaml");

    // Clean up any existing test file
    let _ = std::fs::remove_file(&session_file);

    // Create a state with modified values
    let mut original_state = AppState::default();

    // Modify worktree view state
    original_state.worktree_view.content_scroll = 42;
    original_state.worktree_view.output_scroll = 13;
    original_state.worktree_view.is_running = true;
    original_state.worktree_view.running_phase = Some("Specify".to_string());

    // Modify dashboard state
    original_state.dashboard_view.git_branch = "test-branch".to_string();
    original_state.dashboard_view.worktree_count = 5;
    original_state.dashboard_view.project_name = "test-project".to_string();

    // Modify settings state
    original_state.settings_view.selected_index = 2;
    original_state.settings_view.current_feature = Some("079".to_string());
    original_state.settings_view.settings.auto_run = false;
    original_state.settings_view.settings.max_turns = 100;

    // Save to YAML file
    original_state
        .save_to_yaml_file(&session_file)
        .expect("Failed to save session");

    // Verify file exists
    assert!(session_file.exists(), "Session file was not created");

    // Verify file is readable YAML
    let file_content = std::fs::read_to_string(&session_file).expect("Failed to read session file");
    assert!(file_content.contains("version:"), "Missing version field");
    assert!(
        file_content.contains("worktree_view:"),
        "Missing worktree_view"
    );
    assert!(
        file_content.contains("dashboard_view:"),
        "Missing dashboard_view"
    );
    assert!(
        file_content.contains("settings_view:"),
        "Missing settings_view"
    );

    // Load from file
    let loaded_state = AppState::load_from_file(&session_file).expect("Failed to load session");

    // Verify all fields are preserved
    assert_eq!(
        loaded_state.version, original_state.version,
        "Version mismatch"
    );

    // Verify worktree view fields
    assert_eq!(
        loaded_state.worktree_view.content_scroll, 42,
        "content_scroll not preserved"
    );
    assert_eq!(
        loaded_state.worktree_view.output_scroll, 13,
        "output_scroll not preserved"
    );
    assert_eq!(
        loaded_state.worktree_view.is_running, true,
        "is_running not preserved"
    );
    assert_eq!(
        loaded_state.worktree_view.running_phase,
        Some("Specify".to_string()),
        "running_phase not preserved"
    );

    // Verify dashboard fields
    assert_eq!(
        loaded_state.dashboard_view.git_branch, "test-branch",
        "git_branch not preserved"
    );
    assert_eq!(
        loaded_state.dashboard_view.worktree_count, 5,
        "worktree_count not preserved"
    );
    assert_eq!(
        loaded_state.dashboard_view.project_name, "test-project",
        "project_name not preserved"
    );

    // Verify settings fields
    assert_eq!(
        loaded_state.settings_view.selected_index, 2,
        "selected_index not preserved"
    );
    assert_eq!(
        loaded_state.settings_view.current_feature,
        Some("079".to_string()),
        "current_feature not preserved"
    );
    assert_eq!(
        loaded_state.settings_view.settings.auto_run, false,
        "auto_run not preserved"
    );
    assert_eq!(
        loaded_state.settings_view.settings.max_turns, 100,
        "max_turns not preserved"
    );

    // Clean up
    std::fs::remove_file(&session_file).expect("Failed to clean up test file");
}

/// Test that missing session file doesn't cause errors
#[test]
fn test_load_missing_session() {
    let temp_dir = std::env::temp_dir();
    let missing_file = temp_dir.join("nonexistent_session.yaml");

    // Ensure file doesn't exist
    let _ = std::fs::remove_file(&missing_file);

    // Load should fail gracefully
    let result = AppState::load_from_file(&missing_file);
    assert!(result.is_err(), "Should fail for missing file");
}

/// Test that corrupted session file fails gracefully
#[test]
fn test_load_corrupted_session() {
    let temp_dir = std::env::temp_dir();
    let corrupted_file = temp_dir.join("corrupted_session.yaml");

    // Create corrupted YAML
    std::fs::write(&corrupted_file, "invalid: yaml: syntax: [[[").expect("Failed to write file");

    // Load should fail
    let result = AppState::load_from_file(&corrupted_file);
    assert!(result.is_err(), "Should fail for corrupted file");

    // Clean up
    std::fs::remove_file(&corrupted_file).expect("Failed to clean up");
}

/// Test session file format and structure
#[test]
fn test_session_file_format() {
    let temp_dir = std::env::temp_dir();
    let session_file = temp_dir.join("test_format.yaml");

    let state = AppState::default();
    state
        .save_to_yaml_file(&session_file)
        .expect("Failed to save");

    let content = std::fs::read_to_string(&session_file).expect("Failed to read");

    // Verify YAML structure
    assert!(content.starts_with("version:"), "Should start with version");
    assert!(
        content.contains("worktree_view:"),
        "Should have worktree_view"
    );
    assert!(
        content.contains("dashboard_view:"),
        "Should have dashboard_view"
    );
    assert!(
        content.contains("settings_view:"),
        "Should have settings_view"
    );

    // Verify some nested fields
    assert!(
        content.contains("content_scroll:"),
        "Should have scroll fields"
    );
    assert!(content.contains("git_branch:"), "Should have git fields");
    assert!(
        content.contains("max_turns:"),
        "Should have settings fields"
    );

    // Clean up
    std::fs::remove_file(&session_file).expect("Failed to clean up");
}

/// Test that session preserves complex nested structures
#[test]
fn test_session_preserves_collections() {
    let temp_dir = std::env::temp_dir();
    let session_file = temp_dir.join("test_collections.yaml");

    let mut state = AppState::default();

    // Set complex collections
    state.dashboard_view.git_status = vec![
        "M src/main.rs".to_string(),
        "A new_file.rs".to_string(),
        "D old_file.rs".to_string(),
    ];

    state.worktree_view.log_entries = vec![]; // Empty but should serialize

    // Save and load
    state.save_to_yaml_file(&session_file).unwrap();
    let loaded = AppState::load_from_file(&session_file).unwrap();

    // Verify collections preserved
    assert_eq!(loaded.dashboard_view.git_status.len(), 3);
    assert_eq!(loaded.dashboard_view.git_status[0], "M src/main.rs");
    assert_eq!(loaded.dashboard_view.git_status[1], "A new_file.rs");
    assert_eq!(loaded.dashboard_view.git_status[2], "D old_file.rs");

    // Clean up
    std::fs::remove_file(&session_file).unwrap();
}

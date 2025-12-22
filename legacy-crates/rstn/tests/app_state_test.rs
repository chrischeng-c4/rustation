//! AppState serialization tests (Phase 4)
//!
//! Tests complete application state serialization including all view states:
//! - WorktreeViewState (36 fields)
//! - DashboardState (12 fields)
//! - SettingsState (4 fields)

use rstn::tui::state::dashboard::DashboardState;
use rstn::tui::state::settings::SettingsState;
use rstn::tui::state::worktree::WorktreeViewState;
use rstn::tui::state::{AppState, StateInvariants};

/// Test AppState JSON serialization round-trip
#[test]
fn test_app_state_json_round_trip() {
    let original = AppState::default();

    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Failed to serialize to JSON");

    // Deserialize back
    let deserialized: AppState =
        serde_json::from_str(&json).expect("Failed to deserialize from JSON");

    // Should be identical
    assert_eq!(original, deserialized);
}

/// Test AppState YAML serialization round-trip
#[test]
fn test_app_state_yaml_round_trip() {
    let original = AppState::default();

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&original).expect("Failed to serialize to YAML");

    // Deserialize back
    let deserialized: AppState =
        serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

    // Should be identical
    assert_eq!(original, deserialized);
}

/// Test that all view states preserve their fields
#[test]
fn test_all_view_states_preserved() {
    let mut state = AppState::default();

    // Modify worktree view state
    state.worktree_view.content_scroll = 42;
    state.worktree_view.output_scroll = 13;

    // Modify dashboard state
    state.dashboard_view.git_branch = "feature-079".to_string();
    state.dashboard_view.worktree_count = 5;

    // Modify settings state
    state.settings_view.selected_index = 2;
    state.settings_view.current_feature = Some("079".to_string());

    // Serialize and deserialize
    let json = serde_json::to_string(&state).unwrap();
    let loaded: AppState = serde_json::from_str(&json).unwrap();

    // Verify worktree state
    assert_eq!(loaded.worktree_view.content_scroll, 42);
    assert_eq!(loaded.worktree_view.output_scroll, 13);

    // Verify dashboard state
    assert_eq!(loaded.dashboard_view.git_branch, "feature-079");
    assert_eq!(loaded.dashboard_view.worktree_count, 5);

    // Verify settings state
    assert_eq!(loaded.settings_view.selected_index, 2);
    assert_eq!(
        loaded.settings_view.current_feature,
        Some("079".to_string())
    );
}

/// Test AppState invariants validation
#[test]
fn test_app_state_invariants() {
    let state = AppState::default();

    // Should not panic
    state.assert_invariants();
}

/// Test AppState version field
#[test]
fn test_app_state_version() {
    let state = AppState::default();

    // Version should be set to CARGO_PKG_VERSION
    assert!(!state.version.is_empty());
    assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
}

/// Test save_to_file and load_from_file (JSON)
#[test]
fn test_save_load_json_file() {
    let original = AppState::default();
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_app_state.json");

    // Save to file
    original
        .save_to_file(&file_path)
        .expect("Failed to save state");

    // Load from file
    let loaded = AppState::load_from_file(&file_path).expect("Failed to load state");

    // Should be identical
    assert_eq!(original, loaded);

    // Cleanup
    std::fs::remove_file(&file_path).ok();
}

/// Test save_to_yaml_file and load_from_file (YAML)
#[test]
fn test_save_load_yaml_file() {
    let original = AppState::default();
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_app_state.yaml");

    // Save to YAML file
    original
        .save_to_yaml_file(&file_path)
        .expect("Failed to save YAML state");

    // Load from file (auto-detects YAML)
    let loaded = AppState::load_from_file(&file_path).expect("Failed to load YAML state");

    // Should be identical
    assert_eq!(original, loaded);

    // Cleanup
    std::fs::remove_file(&file_path).ok();
}

/// Test pretty-printed JSON output
#[test]
fn test_pretty_json_format() {
    let state = AppState::default();

    let json = serde_json::to_string_pretty(&state).expect("Failed to serialize");

    // Should be formatted with indentation
    assert!(json.contains("  \"version\":"));
    assert!(json.contains("  \"worktree_view\":"));
    assert!(json.contains("  \"dashboard_view\":"));
    assert!(json.contains("  \"settings_view\":"));
}

/// Test that invalid version logs warning but doesn't fail
#[test]
fn test_version_mismatch_warning() {
    let mut state = AppState::default();
    state.version = "0.0.1".to_string(); // Old version

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_version_mismatch.json");

    // Save old version
    state.save_to_file(&file_path).unwrap();

    // Load should succeed with warning (Phase 2 behavior)
    let loaded = AppState::load_from_file(&file_path);
    assert!(loaded.is_ok());

    // Cleanup
    std::fs::remove_file(&file_path).ok();
}

/// Test schema_version static method
#[test]
fn test_schema_version() {
    let version = AppState::schema_version();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

/// Test WorktreeViewState independently
#[test]
fn test_worktree_view_state_serialization() {
    let state = WorktreeViewState::default();

    let json = serde_json::to_string(&state).expect("Failed to serialize WorktreeViewState");
    let loaded: WorktreeViewState =
        serde_json::from_str(&json).expect("Failed to deserialize WorktreeViewState");

    assert_eq!(state, loaded);
}

/// Test DashboardState independently
#[test]
fn test_dashboard_state_serialization() {
    let state = DashboardState::default();

    let json = serde_json::to_string(&state).expect("Failed to serialize DashboardState");
    let loaded: DashboardState =
        serde_json::from_str(&json).expect("Failed to deserialize DashboardState");

    assert_eq!(state, loaded);
}

/// Test SettingsState independently
#[test]
fn test_settings_state_serialization() {
    let state = SettingsState::default();

    let json = serde_json::to_string(&state).expect("Failed to serialize SettingsState");
    let loaded: SettingsState =
        serde_json::from_str(&json).expect("Failed to deserialize SettingsState");

    assert_eq!(state, loaded);
}

/// Test complex AppState with all fields populated
#[test]
fn test_complex_app_state_serialization() {
    let mut state = AppState::default();

    // Populate worktree view
    state.worktree_view.content_scroll = 100;
    state.worktree_view.output_scroll = 50;
    state.worktree_view.is_running = true;
    state.worktree_view.running_phase = Some("Specify".to_string());

    // Populate dashboard
    state.dashboard_view.git_branch = "main".to_string();
    state.dashboard_view.git_status = vec!["M src/main.rs".to_string()];
    state.dashboard_view.worktree_count = 3;
    state.dashboard_view.project_name = "rstn".to_string();

    // Populate settings
    state.settings_view.selected_index = 1;
    state.settings_view.current_feature = Some("079".to_string());
    state.settings_view.status_message = Some("Settings updated".to_string());
    state.settings_view.settings.auto_run = false;
    state.settings_view.settings.max_turns = 100;

    // Serialize to JSON
    let json = serde_json::to_string(&state).expect("Failed to serialize complex state");

    // Deserialize back
    let loaded: AppState =
        serde_json::from_str(&json).expect("Failed to deserialize complex state");

    // Verify all fields preserved
    assert_eq!(loaded.worktree_view.content_scroll, 100);
    assert_eq!(loaded.worktree_view.output_scroll, 50);
    assert_eq!(loaded.worktree_view.is_running, true);
    assert_eq!(
        loaded.worktree_view.running_phase,
        Some("Specify".to_string())
    );

    assert_eq!(loaded.dashboard_view.git_branch, "main");
    assert_eq!(loaded.dashboard_view.git_status.len(), 1);
    assert_eq!(loaded.dashboard_view.worktree_count, 3);

    assert_eq!(loaded.settings_view.selected_index, 1);
    assert_eq!(
        loaded.settings_view.current_feature,
        Some("079".to_string())
    );
    assert_eq!(loaded.settings_view.settings.auto_run, false);
    assert_eq!(loaded.settings_view.settings.max_turns, 100);
}

/// Test empty optional fields
#[test]
fn test_empty_optional_fields() {
    let mut state = AppState::default();

    // Set all Option fields to None
    state.worktree_view.feature_info = None;
    state.worktree_view.spec_content = None;
    state.worktree_view.plan_content = None;
    state.worktree_view.current_phase = None;
    state.dashboard_view.worktree_path = None;
    state.dashboard_view.git_error = None;
    state.settings_view.current_feature = None;
    state.settings_view.status_message = None;

    // Should serialize and deserialize correctly
    let json = serde_json::to_string(&state).unwrap();
    let loaded: AppState = serde_json::from_str(&json).unwrap();

    assert_eq!(state, loaded);
}

/// Test that invariants are checked on deserialization
#[test]
#[should_panic(expected = "Worktree count must be at least 1")]
fn test_invalid_state_panics() {
    // Create state with invalid data
    let json = r#"{
        "version": "0.1.0",
        "worktree_view": {
            "feature_info": null,
            "worktree_type": "NotGit",
            "spec_content": null,
            "plan_content": null,
            "tasks_content": null,
            "phases": [],
            "current_phase": null,
            "focus": "Commands",
            "content_type": "Spec",
            "content_scroll": 0,
            "commands": [],
            "command_state_index": null,
            "log_entries": [],
            "output_scroll": 0,
            "is_running": false,
            "running_phase": null,
            "pending_git_command": null,
            "active_session_id": null,
            "pending_follow_up": false,
            "pending_input_phase": null,
            "prompt_input": null,
            "inline_input": null,
            "progress_step": null,
            "progress_total": null,
            "progress_message": null,
            "pending_commit_message": null,
            "commit_warnings": [],
            "commit_groups": null,
            "current_commit_index": 0,
            "commit_message_input": "",
            "commit_message_cursor": 0,
            "commit_sensitive_files": [],
            "commit_validation_error": null,
            "specify_state": {
                "current_phase": "Specify",
                "input_buffer": "",
                "input_cursor": 0,
                "is_generating": false,
                "generation_error": null,
                "generated_spec": null,
                "feature_number": null,
                "feature_name": null,
                "edit_mode": false,
                "edit_text_input": null,
                "task_list_state": null,
                "executing_task_index": null,
                "execution_output": "",
                "auto_advance": false,
                "validation_error": null
            },
            "prompt_edit_mode": false,
            "prompt_output": ""
        },
        "dashboard_view": {
            "focused_panel": "QuickActions",
            "git_branch": "main",
            "git_status": [],
            "worktree_count": 0,
            "worktree_path": null,
            "is_git_repo": true,
            "worktree_type": "MainRepository",
            "git_error": null,
            "test_results": null,
            "project_name": "test",
            "rust_version": "1.75",
            "quick_action_index": 0
        },
        "settings_view": {
            "settings": {
                "auto_run": true,
                "max_turns": 50,
                "skip_permissions": true,
                "logging_enabled": true,
                "log_level": "info",
                "log_to_console": false,
                "claude_path": null
            },
            "selected_index": 0,
            "current_feature": null,
            "status_message": null
        },
        "session_history_view": {
            "selected_index": null,
            "focus": "List",
            "max_sessions": 50,
            "show_log_preview": true,
            "log_scroll": 0,
            "filter_type": null,
            "filter_status": null
        }
    }"#;

    let state: AppState = serde_json::from_str(json).unwrap();

    // This should panic due to worktree_count = 0 violating invariant
    state.assert_invariants();
}

//! State-only tests for WorktreeViewState
//!
//! These tests verify:
//! - Serialization round-trips (JSON, YAML)
//! - State construction via builders
//! - State equality
//! - Field preservation across serialization
//!
//! NO UI, NO RENDERING, NO EVENT HANDLING - just pure state tests.

use rstn::tui::state::builders::WorktreeViewStateBuilder;
use rstn::tui::state::worktree::WorktreeViewState;
use rstn::tui::state::StateInvariants;
use rstn::tui::views::{ContentType, PhaseStatus, SpecPhase, WorktreeFocus};

// ========================================
// Serialization Round-Trip Tests
// ========================================

#[test]
fn test_worktree_state_serialization_json_round_trip_minimal() {
    let state = WorktreeViewStateBuilder::new().build();

    // Serialize to JSON
    let json = serde_json::to_string(&state).expect("Should serialize to JSON");

    // Deserialize back
    let loaded: WorktreeViewState =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    // Must be identical
    assert_eq!(state, loaded, "State should round-trip through JSON");
}

#[test]
fn test_worktree_state_serialization_json_round_trip_with_feature() {
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    let json = serde_json::to_string(&state).expect("Should serialize to JSON");
    let loaded: WorktreeViewState =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(state, loaded);
}

#[test]
fn test_worktree_state_serialization_json_round_trip_full() {
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .with_phase(SpecPhase::Plan, PhaseStatus::InProgress)
        .with_spec_content("# Feature 042\n\nClick support for tabs")
        .with_plan_content("# Plan\n\n1. Task 1\n2. Task 2")
        .with_focus(WorktreeFocus::Content)
        .with_content_type(ContentType::Plan)
        .with_content_scroll(42)
        .build();

    let json = serde_json::to_string(&state).expect("Should serialize to JSON");
    let loaded: WorktreeViewState =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(state, loaded);
}

#[test]
fn test_worktree_state_serialization_yaml_round_trip_minimal() {
    let state = WorktreeViewStateBuilder::new().build();

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&state).expect("Should serialize to YAML");

    // Deserialize back
    let loaded: WorktreeViewState =
        serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

    assert_eq!(state, loaded, "State should round-trip through YAML");
}

#[test]
fn test_worktree_state_serialization_yaml_round_trip_full() {
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .with_spec_content("# Feature 042")
        .build();

    let yaml = serde_yaml::to_string(&state).expect("Should serialize to YAML");
    let loaded: WorktreeViewState =
        serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

    assert_eq!(state, loaded);
}

// ========================================
// Field Preservation Tests
// ========================================

#[test]
fn test_worktree_state_preserves_all_fields() {
    let original = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .with_spec_content("Content")
        .with_plan_content("Plan")
        .with_tasks_content("Tasks")
        .with_current_phase(SpecPhase::Plan)
        .with_focus(WorktreeFocus::Content)
        .with_content_type(ContentType::Plan)
        .with_content_scroll(100)
        .build();

    let json = serde_json::to_string(&original).unwrap();
    let loaded: WorktreeViewState = serde_json::from_str(&json).unwrap();

    // Verify every important field
    assert_eq!(original.feature_info, loaded.feature_info);
    assert_eq!(original.worktree_type, loaded.worktree_type);
    assert_eq!(original.spec_content, loaded.spec_content);
    assert_eq!(original.plan_content, loaded.plan_content);
    assert_eq!(original.tasks_content, loaded.tasks_content);
    assert_eq!(original.phases, loaded.phases);
    assert_eq!(original.current_phase, loaded.current_phase);
    assert_eq!(original.focus, loaded.focus);
    assert_eq!(original.content_type, loaded.content_type);
    assert_eq!(original.content_scroll, loaded.content_scroll);
}

// ========================================
// Builder Tests
// ========================================

#[test]
fn test_builder_empty_state() {
    let state = WorktreeViewStateBuilder::new().build();

    assert_eq!(state.feature_info, None);
    assert_eq!(state.spec_content, None);
    assert_eq!(state.focus, WorktreeFocus::Commands);
    assert_eq!(state.content_type, ContentType::Spec);
}

#[test]
fn test_builder_with_feature() {
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    let feature = state.feature_info.expect("Feature info should be set");
    assert_eq!(feature.number, "042");
    assert_eq!(feature.name, "click-function");
    assert_eq!(feature.branch, "042-click-function");
}

#[test]
fn test_builder_preset_specify_in_progress() {
    let state = WorktreeViewStateBuilder::specify_in_progress("042");

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::InProgress
    );
    assert_eq!(state.current_phase, Some(SpecPhase::Specify));
    assert_eq!(state.focus, WorktreeFocus::Output);
    assert_eq!(state.content_type, ContentType::Spec);
}

#[test]
fn test_builder_preset_specify_completed() {
    let state = WorktreeViewStateBuilder::specify_completed("042");

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert!(state.spec_content.is_some());
    assert_eq!(state.content_type, ContentType::Spec);
}

#[test]
fn test_builder_preset_plan_completed() {
    let state = WorktreeViewStateBuilder::plan_completed("042");

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::Completed
    );
    assert!(state.spec_content.is_some());
    assert!(state.plan_content.is_some());
}

// ========================================
// State Invariant Tests
// ========================================

#[test]
fn test_invariants_hold_for_empty_state() {
    let state = WorktreeViewStateBuilder::new().build();
    state.assert_invariants(); // Should not panic
}

#[test]
fn test_invariants_hold_for_feature_state() {
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();
    state.assert_invariants();
}

#[test]
fn test_invariants_hold_for_all_presets() {
    let presets = vec![
        WorktreeViewStateBuilder::specify_in_progress("042"),
        WorktreeViewStateBuilder::specify_completed("042"),
        WorktreeViewStateBuilder::plan_completed("042"),
    ];

    for state in presets {
        state.assert_invariants();
    }
}

// ========================================
// State Getter/Setter Tests
// ========================================

#[test]
fn test_get_phase_status() {
    let state = WorktreeViewStateBuilder::new().build();

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::NotStarted
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::NotStarted
    );
}

#[test]
fn test_set_phase_status() {
    let mut state = WorktreeViewStateBuilder::new().build();

    state.set_phase_status(SpecPhase::Specify, PhaseStatus::Completed);

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    // Other phases unaffected
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::NotStarted
    );
}

// ========================================
// Phase 3A: P2 Fields Serialization Tests
// ========================================

#[test]
fn test_p2_commands_serialization() {
    use rstn::tui::views::{Command, GitCommand};

    // Create state with command state
    let state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Verify default command list is populated
    assert!(
        !state.commands.is_empty(),
        "Commands list should not be empty"
    );
    assert!(state.commands.contains(&Command::PromptClaude));
    assert_eq!(state.command_state_index, Some(1)); // Prompt Claude is at index 1

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.commands, loaded.commands);
    assert_eq!(state.command_state_index, loaded.command_state_index);
}

#[test]
fn test_p2_logging_output_serialization() {
    use rstn::tui::logging::{LogCategory, LogEntry};

    // Create state with log entries
    let mut state = WorktreeViewStateBuilder::new().build();

    // Add log entries manually
    state.log_entries = vec![
        LogEntry::new(LogCategory::Command, "Running /speckit.specify".to_string()),
        LogEntry::new(
            LogCategory::ClaudeStream,
            "Analyzing feature...".to_string(),
        ),
        LogEntry::new(LogCategory::System, "Command completed".to_string()),
    ];
    state.output_scroll = 10;

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.log_entries.len(), loaded.log_entries.len());
    assert_eq!(state.output_scroll, loaded.output_scroll);

    // Verify log entry content
    for (original, loaded_entry) in state.log_entries.iter().zip(loaded.log_entries.iter()) {
        assert_eq!(original.category, loaded_entry.category);
        assert_eq!(original.content, loaded_entry.content);
    }
}

#[test]
fn test_p2_running_state_serialization() {
    let mut state = WorktreeViewStateBuilder::new().build();

    // Set running state
    state.is_running = true;
    state.running_phase = Some("Specify".to_string());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.is_running, loaded.is_running);
    assert_eq!(state.running_phase, loaded.running_phase);
}

#[test]
fn test_p2_session_state_serialization() {
    let mut state = WorktreeViewStateBuilder::new().build();

    // Set session state
    state.active_session_id = Some("session-12345".to_string());
    state.pending_follow_up = true;

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.active_session_id, loaded.active_session_id);
    assert_eq!(state.pending_follow_up, loaded.pending_follow_up);
}

#[test]
fn test_p2_pending_git_command_serialization() {
    use rstn::tui::views::GitCommand;

    let mut state = WorktreeViewStateBuilder::new().build();

    // Set pending git command
    state.pending_git_command = Some(GitCommand::Commit);

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.pending_git_command, loaded.pending_git_command);
}

#[test]
fn test_p2_invariant_command_state_index_in_bounds() {
    let mut state = WorktreeViewStateBuilder::new().build();

    // Valid index
    state.command_state_index = Some(0);
    state.assert_invariants(); // Should not panic

    // Invalid index (out of bounds)
    state.command_state_index = Some(state.commands.len() + 10);

    let result = std::panic::catch_unwind(|| {
        state.assert_invariants();
    });

    assert!(
        result.is_err(),
        "Should panic for out-of-bounds command index"
    );
}

#[test]
fn test_p2_invariant_running_phase_implies_is_running() {
    let mut state = WorktreeViewStateBuilder::new().build();

    // Valid: running_phase set AND is_running = true
    state.running_phase = Some("Specify".to_string());
    state.is_running = true;
    state.assert_invariants(); // Should not panic

    // Invalid: running_phase set BUT is_running = false
    state.is_running = false;

    let result = std::panic::catch_unwind(|| {
        state.assert_invariants();
    });

    assert!(
        result.is_err(),
        "Should panic when running_phase is set but is_running is false"
    );
}

#[test]
fn test_p2_full_state_with_all_fields() {
    use rstn::tui::logging::{LogCategory, LogEntry};
    use rstn::tui::views::GitCommand;

    // Create comprehensive P2 state
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .with_phase(SpecPhase::Specify, PhaseStatus::InProgress)
        .build();

    // Set all P2 fields
    state.command_state_index = Some(2);
    state.log_entries = vec![
        LogEntry::new(LogCategory::Command, "Test log 1".to_string()),
        LogEntry::new(LogCategory::System, "Test log 2".to_string()),
    ];
    state.output_scroll = 5;
    state.is_running = true;
    state.running_phase = Some("Specify".to_string());
    state.pending_git_command = Some(GitCommand::Push);
    state.active_session_id = Some("session-xyz".to_string());
    state.pending_follow_up = true;

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify P2 fields preserved (compare individually to handle timestamp precision)
    assert_eq!(state.commands, loaded.commands);
    assert_eq!(state.command_state_index, loaded.command_state_index);
    assert_eq!(state.log_entries.len(), loaded.log_entries.len());
    for (orig, load) in state.log_entries.iter().zip(loaded.log_entries.iter()) {
        assert_eq!(orig.category, load.category);
        assert_eq!(orig.content, load.content);
        // Timestamp precision to seconds is acceptable
    }
    assert_eq!(state.output_scroll, loaded.output_scroll);
    assert_eq!(state.is_running, loaded.is_running);
    assert_eq!(state.running_phase, loaded.running_phase);
    assert_eq!(state.pending_git_command, loaded.pending_git_command);
    assert_eq!(state.active_session_id, loaded.active_session_id);
    assert_eq!(state.pending_follow_up, loaded.pending_follow_up);
}
// ========================================
// Phase 3B: P3 Fields Serialization Tests
// ========================================

#[test]
fn test_p3_input_subsystem_serialization() {
    use rstn::tui::views::InlineInput;
    use rstn::tui::views::SpecPhase;
    use rstn::tui::widgets::TextInput;

    // Create state with input subsystem fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Set P3 input fields
    state.pending_input_phase = Some(SpecPhase::Specify);
    state.prompt_input = Some(TextInput::new("Enter prompt:".to_string()));
    state.inline_input = Some(InlineInput::new("Claude asks: What next?".to_string()));

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.pending_input_phase, loaded.pending_input_phase);
    assert_eq!(state.prompt_input, loaded.prompt_input);
    assert_eq!(state.inline_input, loaded.inline_input);
}

#[test]
fn test_p3_progress_subsystem_serialization() {
    // Create state with progress subsystem fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Set P3 progress fields
    state.progress_step = Some(3);
    state.progress_total = Some(10);
    state.progress_message = Some("Processing task 3 of 10".to_string());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.progress_step, loaded.progress_step);
    assert_eq!(state.progress_total, loaded.progress_total);
    assert_eq!(state.progress_message, loaded.progress_message);
}

#[test]
fn test_p3_full_subsystems_round_trip() {
    use rstn::tui::views::SpecPhase;
    use rstn::tui::widgets::TextInput;

    // Create comprehensive P3 state
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    // Set all P3 fields
    state.pending_input_phase = Some(SpecPhase::Plan);
    state.prompt_input = Some(TextInput::new("Prompt:".to_string()));
    state.progress_step = Some(5);
    state.progress_total = Some(12);
    state.progress_message = Some("Generating plan...".to_string());

    // Test JSON round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state, loaded);
}

// ========================================
// Phase 3B: P4 Fields Serialization Tests
// ========================================

#[test]
fn test_p4_commit_workflow_serialization() {
    use rstn::domain::git::{CommitGroup, SecurityWarning, Severity};

    // Create state with commit workflow fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Set P4 commit fields
    state.pending_commit_message = Some("Initial commit message".to_string());
    state.commit_warnings = vec![SecurityWarning {
        file_path: "src/main.rs".to_string(),
        line_number: 42,
        pattern_matched: "api_key".to_string(),
        severity: Severity::High,
        message: "Possible API key detected".to_string(),
    }];
    state.commit_groups = Some(vec![CommitGroup {
        files: vec!["src/file1.rs".to_string(), "src/file2.rs".to_string()],
        message: "feat: add new feature".to_string(),
        description: "Implements feature X".to_string(),
        category: Some("feature".to_string()),
    }]);
    state.current_commit_index = 1;
    state.commit_message_input = "Work in progress".to_string();
    state.commit_message_cursor = 15;
    state.commit_sensitive_files = vec![".env".to_string(), "credentials.json".to_string()];
    state.commit_validation_error = Some("Message too short".to_string());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.pending_commit_message, loaded.pending_commit_message);
    assert_eq!(state.commit_warnings.len(), loaded.commit_warnings.len());
    assert_eq!(state.commit_groups, loaded.commit_groups);
    assert_eq!(state.current_commit_index, loaded.current_commit_index);
    assert_eq!(state.commit_message_input, loaded.commit_message_input);
    assert_eq!(state.commit_message_cursor, loaded.commit_message_cursor);
    assert_eq!(state.commit_sensitive_files, loaded.commit_sensitive_files);
    assert_eq!(
        state.commit_validation_error,
        loaded.commit_validation_error
    );
}

#[test]
fn test_p4_security_warnings_serialization() {
    use rstn::domain::git::{SecurityWarning, Severity};

    // Create state with multiple security warnings
    let mut state = WorktreeViewStateBuilder::new().build();

    state.commit_warnings = vec![
        SecurityWarning {
            file_path: "src/auth.rs".to_string(),
            line_number: 10,
            pattern_matched: "password".to_string(),
            severity: Severity::Critical,
            message: "Private key detected".to_string(),
        },
        SecurityWarning {
            file_path: "src/config.rs".to_string(),
            line_number: 25,
            pattern_matched: "token".to_string(),
            severity: Severity::High,
            message: "API token pattern".to_string(),
        },
        SecurityWarning {
            file_path: "src/utils.rs".to_string(),
            line_number: 100,
            pattern_matched: "base64".to_string(),
            severity: Severity::Medium,
            message: "Base64 string detected".to_string(),
        },
    ];

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.commit_warnings.len(), 3);
    assert_eq!(loaded.commit_warnings.len(), 3);

    for (orig, load) in state
        .commit_warnings
        .iter()
        .zip(loaded.commit_warnings.iter())
    {
        assert_eq!(orig.file_path, load.file_path);
        assert_eq!(orig.line_number, load.line_number);
        assert_eq!(orig.pattern_matched, load.pattern_matched);
        assert_eq!(orig.severity, load.severity);
        assert_eq!(orig.message, load.message);
    }
}

#[test]
fn test_p4_commit_groups_serialization() {
    use rstn::domain::git::CommitGroup;

    // Create state with multiple commit groups
    let mut state = WorktreeViewStateBuilder::new().build();

    state.commit_groups = Some(vec![
        CommitGroup {
            files: vec!["src/models/user.rs".to_string()],
            message: "feat(models): add User model".to_string(),
            description: "Implements user data structure".to_string(),
            category: Some("models".to_string()),
        },
        CommitGroup {
            files: vec!["tests/user_test.rs".to_string()],
            message: "test(models): add User tests".to_string(),
            description: "Test user model functionality".to_string(),
            category: Some("tests".to_string()),
        },
        CommitGroup {
            files: vec!["README.md".to_string(), "docs/api.md".to_string()],
            message: "docs: update documentation".to_string(),
            description: "Update project documentation".to_string(),
            category: Some("docs".to_string()),
        },
    ]);

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.commit_groups, loaded.commit_groups);

    let groups = loaded.commit_groups.unwrap();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].category, Some("models".to_string()));
    assert_eq!(groups[1].category, Some("tests".to_string()));
    assert_eq!(groups[2].category, Some("docs".to_string()));
}

// ========================================
// Phase 3B: P5 Fields Serialization Tests
// ========================================

#[test]
fn test_p5_specify_state_serialization() {
    use rstn::tui::views::{SpecPhase, SpecifyState};

    // Create state with specify workflow
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Set P5 specify fields
    state.specify_state = SpecifyState::for_phase(SpecPhase::Plan);
    state.specify_state.input_buffer = "Test feature description".to_string();
    state.specify_state.input_cursor = 10;
    state.specify_state.is_generating = false;
    state.specify_state.generated_spec = Some("# Feature Spec\n\nGenerated content".to_string());
    state.specify_state.feature_number = Some("042".to_string());
    state.specify_state.feature_name = Some("test-feature".to_string());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(
        state.specify_state.current_phase,
        loaded.specify_state.current_phase
    );
    assert_eq!(
        state.specify_state.input_buffer,
        loaded.specify_state.input_buffer
    );
    assert_eq!(
        state.specify_state.input_cursor,
        loaded.specify_state.input_cursor
    );
    assert_eq!(
        state.specify_state.is_generating,
        loaded.specify_state.is_generating
    );
    assert_eq!(
        state.specify_state.generated_spec,
        loaded.specify_state.generated_spec
    );
    assert_eq!(
        state.specify_state.feature_number,
        loaded.specify_state.feature_number
    );
    assert_eq!(
        state.specify_state.feature_name,
        loaded.specify_state.feature_name
    );
}

#[test]
fn test_p5_prompt_workflow_serialization() {
    // Create state with prompt workflow fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Set P5 prompt fields
    state.prompt_edit_mode = true;
    state.prompt_output = "Claude response:\n\nThis is a streaming output...".to_string();

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.prompt_edit_mode, loaded.prompt_edit_mode);
    assert_eq!(state.prompt_output, loaded.prompt_output);
}

#[test]
fn test_p5_full_subsystems_round_trip() {
    use rstn::tui::views::{SpecPhase, SpecifyState};

    // Create comprehensive P5 state
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    // Set all P5 fields
    state.specify_state = SpecifyState::for_phase(SpecPhase::Specify);
    state.specify_state.input_buffer = "Click support for tabs".to_string();
    state.specify_state.generated_spec = Some("# Spec\n\nClick handling".to_string());
    state.prompt_edit_mode = true;
    state.prompt_output = "Processing your request...".to_string();

    // Test JSON round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state, loaded);
}

// ========================================
// Phase 3B: Comprehensive Integration Tests
// ========================================

#[test]
fn test_full_p1_to_p5_state_serialization() {
    use rstn::domain::git::{CommitGroup, SecurityWarning, Severity};
    use rstn::tui::logging::{LogCategory, LogEntry};
    use rstn::tui::views::{GitCommand, SpecPhase, SpecifyState};
    use rstn::tui::widgets::TextInput;

    // Create comprehensive state with ALL P1-P5 fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .with_phase(SpecPhase::Plan, PhaseStatus::InProgress)
        .with_spec_content("# Feature 042\n\nClick support")
        .with_plan_content("# Plan\n\n1. Task 1")
        .build();

    // P2 fields
    state.log_entries = vec![
        LogEntry::new(LogCategory::Command, "Running command".to_string()),
        LogEntry::new(LogCategory::System, "System message".to_string()),
    ];
    state.output_scroll = 5;
    state.is_running = true;
    state.running_phase = Some("Plan".to_string());
    state.pending_git_command = Some(GitCommand::Commit);

    // P3 fields
    state.pending_input_phase = Some(SpecPhase::Plan);
    state.prompt_input = Some(TextInput::new("Enter details:".to_string()));
    state.progress_step = Some(2);
    state.progress_total = Some(5);
    state.progress_message = Some("Step 2 of 5".to_string());

    // P4 fields
    state.pending_commit_message = Some("WIP commit".to_string());
    state.commit_warnings = vec![SecurityWarning {
        file_path: "src/test.rs".to_string(),
        line_number: 10,
        pattern_matched: "key".to_string(),
        severity: Severity::High,
        message: "Possible secret".to_string(),
    }];
    state.commit_groups = Some(vec![CommitGroup {
        files: vec!["src/main.rs".to_string()],
        message: "feat: add feature".to_string(),
        description: "Feature implementation".to_string(),
        category: Some("feat".to_string()),
    }]);
    state.current_commit_index = 0;
    state.commit_message_input = "Initial message".to_string();
    state.commit_message_cursor = 10;
    state.commit_sensitive_files = vec![".env".to_string()];

    // P5 fields
    state.specify_state = SpecifyState::for_phase(SpecPhase::Specify);
    state.specify_state.input_buffer = "Feature description".to_string();
    state.specify_state.generated_spec = Some("# Generated".to_string());
    state.prompt_edit_mode = true;
    state.prompt_output = "Output text".to_string();

    // Test JSON round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify all subsystems preserved
    assert_eq!(state.feature_info, loaded.feature_info); // P1
    assert_eq!(state.commands.len(), loaded.commands.len()); // P2
    assert_eq!(state.pending_input_phase, loaded.pending_input_phase); // P3
    assert_eq!(state.progress_step, loaded.progress_step); // P3
    assert_eq!(state.commit_warnings.len(), loaded.commit_warnings.len()); // P4
    assert_eq!(
        state.specify_state.input_buffer,
        loaded.specify_state.input_buffer
    ); // P5
    assert_eq!(state.prompt_edit_mode, loaded.prompt_edit_mode); // P5
}

#[test]
fn test_yaml_round_trip_with_p3_p5_fields() {
    use rstn::domain::git::CommitGroup;
    use rstn::tui::views::SpecPhase;

    // Create state with P3-P5 fields
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    state.progress_message = Some("Processing...".to_string());
    state.commit_groups = Some(vec![CommitGroup {
        files: vec!["file.rs".to_string()],
        message: "commit msg".to_string(),
        description: "desc".to_string(),
        category: None,
    }]);
    state.prompt_output = "Claude output".to_string();

    // Test YAML round-trip
    let yaml = serde_yaml::to_string(&state).expect("Failed to serialize to YAML");
    let loaded: WorktreeViewState =
        serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

    assert_eq!(state, loaded);
}

#[test]
fn test_empty_collections_serialization() {
    // Verify empty collections serialize/deserialize correctly
    let state = WorktreeViewStateBuilder::new().build();

    // All collections should be empty
    assert!(state.commit_warnings.is_empty());
    assert!(state.commit_groups.is_none());
    assert!(state.commit_sensitive_files.is_empty());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state, loaded);
}

#[test]
fn test_option_fields_none_serialization() {
    // Verify None option fields serialize correctly
    let state = WorktreeViewStateBuilder::new().build();

    // All P3-P5 option fields should be None
    assert!(state.pending_input_phase.is_none());
    assert!(state.prompt_input.is_none());
    assert!(state.inline_input.is_none());
    assert!(state.progress_step.is_none());
    assert!(state.progress_total.is_none());
    assert!(state.progress_message.is_none());
    assert!(state.pending_commit_message.is_none());
    assert!(state.commit_groups.is_none());
    assert!(state.commit_validation_error.is_none());

    // Test round-trip
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    let loaded: WorktreeViewState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state, loaded);
}

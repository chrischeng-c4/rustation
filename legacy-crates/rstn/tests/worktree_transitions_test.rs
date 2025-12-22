//! State transition tests for WorktreeView
//!
//! These tests verify state transitions across workflows:
//! - Feature detection (NotGit → FeatureWorktree)
//! - Phase progression (NotStarted → InProgress → Completed)
//! - Command execution (idle → running → completed)
//! - Content loading (empty → loaded)
//!
//! Tests use state-only operations (NO UI, NO RENDERING).

use rstn::tui::event::WorktreeType;
use rstn::tui::state::builders::WorktreeViewStateBuilder;
use rstn::tui::state::worktree::WorktreeViewState;
use rstn::tui::state::StateInvariants;
use rstn::tui::views::{ContentType, GitCommand, PhaseStatus, SpecPhase, WorktreeFocus};

// ========================================
// Feature Detection Workflow Tests
// ========================================

#[test]
fn test_transition_no_git_to_feature_worktree() {
    // Initial state: Not a git repository
    let mut state = WorktreeViewStateBuilder::new()
        .with_worktree_type(WorktreeType::NotGit)
        .build();

    assert!(state.feature_info.is_none());
    assert_eq!(state.worktree_type, WorktreeType::NotGit);

    // Transition: Detect feature worktree
    state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    // Verify transition
    assert!(state.feature_info.is_some());
    assert!(matches!(
        state.worktree_type,
        WorktreeType::FeatureWorktree { .. }
    ));
    assert_eq!(state.feature_info.as_ref().unwrap().number, "042");
    assert_eq!(state.feature_info.as_ref().unwrap().name, "click-function");
}

#[test]
fn test_transition_main_repository_to_feature_worktree() {
    // Initial state: Main repository
    let mut state = WorktreeViewStateBuilder::new()
        .with_worktree_type(WorktreeType::MainRepository)
        .build();

    assert_eq!(state.worktree_type, WorktreeType::MainRepository);

    // Transition: Switch to feature worktree (e.g., via git worktree add)
    state = WorktreeViewStateBuilder::new()
        .with_feature("053", "new-feature")
        .build();

    // Verify transition
    assert!(matches!(
        state.worktree_type,
        WorktreeType::FeatureWorktree { .. }
    ));
    assert_eq!(state.feature_info.as_ref().unwrap().number, "053");
}

// ========================================
// Phase Progression Tests
// ========================================

#[test]
fn test_transition_phase_not_started_to_in_progress() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: All phases NotStarted
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::NotStarted
    );

    // Transition: Start Specify phase
    state.set_phase_status(SpecPhase::Specify, PhaseStatus::InProgress);
    state.current_phase = Some(SpecPhase::Specify);

    // Verify transition
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::InProgress
    );
    assert_eq!(state.current_phase, Some(SpecPhase::Specify));
}

#[test]
fn test_transition_phase_in_progress_to_completed() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .with_phase(SpecPhase::Specify, PhaseStatus::InProgress)
        .build();

    state.current_phase = Some(SpecPhase::Specify);

    // Transition: Complete Specify phase
    state.set_phase_status(SpecPhase::Specify, PhaseStatus::Completed);
    state.current_phase = None;

    // Verify transition
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert_eq!(state.current_phase, None);
}

#[test]
fn test_transition_sequential_phase_progression() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Workflow: Specify → Plan → Tasks
    let workflow = vec![SpecPhase::Specify, SpecPhase::Plan, SpecPhase::Tasks];

    for phase in workflow {
        // Start phase
        state.set_phase_status(phase, PhaseStatus::InProgress);
        state.current_phase = Some(phase);
        assert_eq!(state.get_phase_status(phase), PhaseStatus::InProgress);
        assert_eq!(state.current_phase, Some(phase));

        // Complete phase
        state.set_phase_status(phase, PhaseStatus::Completed);
        state.current_phase = None;
        assert_eq!(state.get_phase_status(phase), PhaseStatus::Completed);
        assert_eq!(state.current_phase, None);
    }

    // Verify all three phases completed
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::Completed
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Tasks),
        PhaseStatus::Completed
    );
}

#[test]
fn test_transition_phase_needs_update() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .build();

    // Transition: Spec needs update (e.g., requirements changed)
    state.set_phase_status(SpecPhase::Specify, PhaseStatus::NeedsUpdate);

    // Verify transition
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::NeedsUpdate
    );
}

// ========================================
// Command Execution Tests
// ========================================

#[test]
fn test_transition_command_execution_start() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: Not running
    assert!(!state.is_running);
    assert!(state.running_phase.is_none());

    // Transition: Start command execution
    state.is_running = true;
    state.running_phase = Some("Specify".to_string());

    // Verify transition
    assert!(state.is_running);
    assert_eq!(state.running_phase, Some("Specify".to_string()));

    // State invariants should hold
    state.assert_invariants();
}

#[test]
fn test_transition_command_execution_complete() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Start execution
    state.is_running = true;
    state.running_phase = Some("Plan".to_string());

    // Transition: Complete execution
    state.is_running = false;
    state.running_phase = None;

    // Verify transition
    assert!(!state.is_running);
    assert!(state.running_phase.is_none());
}

#[test]
fn test_transition_command_execution_with_session() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Transition: Start command with session tracking
    state.is_running = true;
    state.running_phase = Some("Specify".to_string());
    state.active_session_id = Some("session-abc123".to_string());

    // Verify transition
    assert!(state.is_running);
    assert_eq!(state.active_session_id, Some("session-abc123".to_string()));

    state.assert_invariants();

    // Transition: Command completes but session remains (for follow-up)
    state.is_running = false;
    state.running_phase = None;
    // active_session_id persists

    assert!(!state.is_running);
    assert_eq!(state.active_session_id, Some("session-abc123".to_string()));
}

#[test]
fn test_transition_pending_follow_up() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Transition: Command pauses for user input
    state.is_running = false;
    state.pending_follow_up = true;
    state.active_session_id = Some("session-xyz".to_string());

    // Verify transition
    assert!(state.pending_follow_up);
    assert!(!state.is_running);

    // Transition: User provides input, command resumes
    state.pending_follow_up = false;
    state.is_running = true;

    assert!(!state.pending_follow_up);
    assert!(state.is_running);
}

// ========================================
// Content Loading Tests
// ========================================

#[test]
fn test_transition_load_spec_content() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: No content loaded
    assert!(state.spec_content.is_none());
    assert_eq!(state.content_type, ContentType::Spec);

    // Transition: Load spec content
    state.spec_content =
        Some("# Feature 042: Click Function\n\nImplement click support.".to_string());
    state.content_type = ContentType::Spec;

    // Verify transition
    assert!(state.spec_content.is_some());
    assert!(state.spec_content.as_ref().unwrap().contains("Feature 042"));
}

#[test]
fn test_transition_load_plan_content() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
        .build();

    // Initial: Spec loaded, plan not loaded
    state.spec_content = Some("# Spec".to_string());
    assert!(state.plan_content.is_none());

    // Transition: Load plan content
    state.plan_content = Some("# Implementation Plan\n\n## Architecture".to_string());
    state.content_type = ContentType::Plan;
    state.set_phase_status(SpecPhase::Plan, PhaseStatus::InProgress);

    // Verify transition
    assert!(state.plan_content.is_some());
    assert_eq!(state.content_type, ContentType::Plan);
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::InProgress
    );
}

#[test]
fn test_transition_load_tasks_content() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .with_phase(SpecPhase::Plan, PhaseStatus::Completed)
        .build();

    // Transition: Load tasks content
    state.tasks_content = Some("# Tasks\n\n- [ ] Task 1\n- [ ] Task 2".to_string());
    state.content_type = ContentType::Tasks;
    state.set_phase_status(SpecPhase::Tasks, PhaseStatus::InProgress);

    // Verify transition
    assert!(state.tasks_content.is_some());
    assert_eq!(state.content_type, ContentType::Tasks);
}

#[test]
fn test_transition_switch_content_view() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Load all content
    state.spec_content = Some("# Spec".to_string());
    state.plan_content = Some("# Plan".to_string());
    state.tasks_content = Some("# Tasks".to_string());

    // Transition: Switch between content types
    state.content_type = ContentType::Spec;
    assert_eq!(state.content_type, ContentType::Spec);

    state.content_type = ContentType::Plan;
    assert_eq!(state.content_type, ContentType::Plan);

    state.content_type = ContentType::Tasks;
    assert_eq!(state.content_type, ContentType::Tasks);

    // All content remains loaded
    assert!(state.spec_content.is_some());
    assert!(state.plan_content.is_some());
    assert!(state.tasks_content.is_some());
}

// ========================================
// Focus Transition Tests
// ========================================

#[test]
fn test_transition_focus_between_panes() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: Focus on Commands
    assert_eq!(state.focus, WorktreeFocus::Commands);

    // Transition: Focus to Content
    state.focus = WorktreeFocus::Content;
    assert_eq!(state.focus, WorktreeFocus::Content);

    // Transition: Focus to Output
    state.focus = WorktreeFocus::Output;
    assert_eq!(state.focus, WorktreeFocus::Output);

    // Transition: Focus back to Commands
    state.focus = WorktreeFocus::Commands;
    assert_eq!(state.focus, WorktreeFocus::Commands);
}

// ========================================
// Git Command Workflow Tests
// ========================================

#[test]
fn test_transition_queue_git_command() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: No pending git command
    assert!(state.pending_git_command.is_none());

    // Transition: Queue git commit
    state.pending_git_command = Some(GitCommand::Commit);

    // Verify transition
    assert_eq!(state.pending_git_command, Some(GitCommand::Commit));
}

#[test]
fn test_transition_execute_git_command() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Queue command
    state.pending_git_command = Some(GitCommand::Push);

    // Transition: Execute command (clear pending)
    state.pending_git_command = None;
    state.is_running = true;
    state.running_phase = Some("Git Push".to_string());

    // Verify transition
    assert!(state.pending_git_command.is_none());
    assert!(state.is_running);

    state.assert_invariants();
}

// ========================================
// Scroll Position Tests
// ========================================

#[test]
fn test_transition_content_scroll() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: No scroll
    assert_eq!(state.content_scroll, 0);

    // Transition: Scroll down
    state.content_scroll = 10;
    assert_eq!(state.content_scroll, 10);

    state.content_scroll = 50;
    assert_eq!(state.content_scroll, 50);

    // Verify invariants hold
    state.assert_invariants();
}

#[test]
fn test_transition_output_scroll() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: No scroll
    assert_eq!(state.output_scroll, 0);

    // Transition: Scroll through output
    state.output_scroll = 20;
    assert_eq!(state.output_scroll, 20);

    // Verify invariants hold
    state.assert_invariants();
}

// ========================================
// Command Selection Tests
// ========================================

#[test]
fn test_transition_command_selection() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Initial: Default selection (Prompt Claude at index 1)
    assert_eq!(state.command_state_index, Some(1));

    // Transition: Navigate to different commands
    state.command_state_index = Some(2); // SDD phase
    assert_eq!(state.command_state_index, Some(2));

    state.command_state_index = Some(10); // Git action
    assert_eq!(state.command_state_index, Some(10));

    // Verify within bounds
    assert!(state.command_state_index.unwrap() < state.commands.len());
    state.assert_invariants();
}

// ========================================
// Integration Tests (Multiple Transitions)
// ========================================

#[test]
fn test_transition_complete_specify_workflow() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "click-function")
        .build();

    // Step 1: Start Specify phase
    state.set_phase_status(SpecPhase::Specify, PhaseStatus::InProgress);
    state.current_phase = Some(SpecPhase::Specify);
    state.is_running = true;
    state.running_phase = Some("Specify".to_string());
    state.active_session_id = Some("session-001".to_string());

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::InProgress
    );
    assert!(state.is_running);

    // Step 2: Generate spec content
    state.spec_content = Some("# Feature 042: Click Function".to_string());
    state.content_type = ContentType::Spec;

    assert!(state.spec_content.is_some());

    // Step 3: Complete Specify phase
    state.set_phase_status(SpecPhase::Specify, PhaseStatus::Completed);
    state.current_phase = None;
    state.is_running = false;
    state.running_phase = None;

    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert!(!state.is_running);

    // Verify final state
    state.assert_invariants();
}

#[test]
fn test_transition_full_sdd_workflow() {
    let mut state = WorktreeViewStateBuilder::new()
        .with_feature("042", "test-feature")
        .build();

    // Execute Specify → Plan → Tasks workflow
    let phases = vec![
        (SpecPhase::Specify, "spec.md content"),
        (SpecPhase::Plan, "plan.md content"),
        (SpecPhase::Tasks, "tasks.md content"),
    ];

    for (phase, content) in phases {
        // Start phase
        state.set_phase_status(phase, PhaseStatus::InProgress);
        state.current_phase = Some(phase);
        state.is_running = true;
        state.running_phase = Some(format!("{:?}", phase));

        // Generate content
        match phase {
            SpecPhase::Specify => state.spec_content = Some(content.to_string()),
            SpecPhase::Plan => state.plan_content = Some(content.to_string()),
            SpecPhase::Tasks => state.tasks_content = Some(content.to_string()),
            _ => {}
        }

        // Complete phase
        state.set_phase_status(phase, PhaseStatus::Completed);
        state.current_phase = None;
        state.is_running = false;
        state.running_phase = None;
    }

    // Verify all phases completed and content loaded
    assert_eq!(
        state.get_phase_status(SpecPhase::Specify),
        PhaseStatus::Completed
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Plan),
        PhaseStatus::Completed
    );
    assert_eq!(
        state.get_phase_status(SpecPhase::Tasks),
        PhaseStatus::Completed
    );
    assert!(state.spec_content.is_some());
    assert!(state.plan_content.is_some());
    assert!(state.tasks_content.is_some());

    state.assert_invariants();
}

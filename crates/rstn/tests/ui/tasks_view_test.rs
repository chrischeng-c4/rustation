//! UI Integration Tests for TasksView
//!
//! These tests verify that TasksView correctly renders with different states.
//! They use GPUI's TestAppContext for headless testing.
//!
//! **Note**: These tests require Metal/Xcode to run. See openspec/UI_TESTING_PLAN.md

use gpui::*;
use rstn::state::AppState;
use rstn_views::TasksView;
use rstn_ui::MaterialTheme;

#[gpui::test]
async fn test_tasks_view_renders_with_empty_state(cx: &mut TestAppContext) {
    // Setup: Create empty state (no project initialized)
    let state = AppState::new();

    cx.update(|cx| {
        let tasks = state.get_justfile_tasks();

        // Verify empty state
        assert!(tasks.is_empty(), "New state should have no tasks");

        // Attempt to render view
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks, theme)
        });

        // Assertion: View should render without panic
        assert!(window.is_ok(), "TasksView should render with empty tasks");
    });
}

#[gpui::test]
async fn test_tasks_view_renders_with_real_tasks(cx: &mut TestAppContext) {
    // Setup: Create state and initialize (loads real justfile)
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let tasks = state.get_justfile_tasks();

        // Verify tasks were loaded from justfile
        assert!(
            !tasks.is_empty(),
            "Initialized state should load tasks from justfile"
        );

        // Verify first task has expected structure
        let first_task = &tasks[0];
        assert!(!first_task.name.is_empty(), "Task should have a name");
        assert!(!first_task.command.is_empty(), "Task should have a command");

        // Render view with real data
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks.clone(), theme)
        });

        assert!(window.is_ok(), "TasksView should render with real tasks");
    });
}

#[gpui::test]
async fn test_tasks_view_displays_correct_task_count(cx: &mut TestAppContext) {
    // Setup: State with known task count
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let tasks = state.get_justfile_tasks();
        let expected_count = tasks.len();

        // Verify we have tasks
        assert!(
            expected_count > 0,
            "Should have at least 1 task from justfile"
        );

        // Render view
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks, theme)
        });

        assert!(window.is_ok());

        // TODO: Once we can query rendered elements, verify:
        // - Number of TaskCard components matches task count
        // - Each task displays name and description
        // - Status indicators are correct
    });
}

#[gpui::test]
async fn test_tasks_view_handles_state_updates(cx: &mut TestAppContext) {
    // Setup: Create state model
    let state_model = cx.new_model(|_cx| {
        let mut s = AppState::new();
        s.initialize();
        s
    });

    cx.update(|cx| {
        // Initial render
        let initial_tasks = state_model.read(cx).get_justfile_tasks();
        let initial_count = initial_tasks.len();

        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(initial_tasks, theme)
        });

        assert!(window.is_ok());

        // TODO: When state update mechanism is implemented:
        // - Update state (e.g., task execution completes)
        // - Re-render view with updated state
        // - Verify UI reflects state changes

        assert!(initial_count > 0, "Should have tasks");
    });
}

#[gpui::test]
async fn test_tasks_view_theme_compatibility(cx: &mut TestAppContext) {
    // Test both dark and light themes
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let tasks = state.get_justfile_tasks();

        // Test dark theme
        let dark_window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks.clone(), theme)
        });
        assert!(dark_window.is_ok(), "Should render with dark theme");

        // Test light theme
        let light_window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::light();
            TasksView::new(tasks.clone(), theme)
        });
        assert!(light_window.is_ok(), "Should render with light theme");
    });
}

// TODO: Add these tests when event handlers are implemented
//
// #[gpui::test]
// async fn test_execute_task_button_click(cx: &mut TestAppContext) {
//     // Test button click triggers command execution
// }
//
// #[gpui::test]
// async fn test_task_status_updates_during_execution(cx: &mut TestAppContext) {
//     // Test status changes: Idle → Running → Success/Failed
// }
//
// #[gpui::test]
// async fn test_log_panel_displays_command_output(cx: &mut TestAppContext) {
//     // Test stdout/stderr streaming to LogPanel
// }

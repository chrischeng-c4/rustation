//! UI Integration Tests for DockersView
//!
//! These tests verify that DockersView correctly renders Docker services.
//! They use GPUI's TestAppContext for headless testing.
//!
//! **Note**: These tests require Metal/Xcode to run. See openspec/UI_TESTING_PLAN.md

use gpui::*;
use rstn::state::AppState;
use rstn_views::DockersView;
use rstn_ui::MaterialTheme;

#[gpui::test]
async fn test_dockers_view_renders_with_empty_services(cx: &mut TestAppContext) {
    // Setup: Empty state (no initialization)
    let state = AppState::new();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Verify no services in uninitialized state
        assert!(services.is_empty(), "New state should have no Docker services");

        // Render view with empty services
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        });

        // Should render empty state without panic
        assert!(window.is_ok(), "DockersView should render with empty services");
    });
}

#[gpui::test]
async fn test_dockers_view_renders_built_in_services(cx: &mut TestAppContext) {
    // Setup: Initialize state to load built-in services
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Verify built-in services were loaded
        assert!(
            !services.is_empty(),
            "Initialized state should have built-in Docker services"
        );

        // Verify service structure
        let first_service = &services[0];
        assert!(!first_service.name.is_empty(), "Service should have a name");
        assert!(!first_service.image.is_empty(), "Service should have an image");

        // Render view
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services.clone(), theme)
        });

        assert!(window.is_ok(), "DockersView should render with built-in services");
    });
}

#[gpui::test]
async fn test_dockers_view_displays_service_metadata(cx: &mut TestAppContext) {
    // Setup: State with services
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();
        let service_count = services.len();

        assert!(service_count > 0, "Should have at least 1 service");

        // Verify each service has required fields
        for service in &services {
            assert!(!service.name.is_empty(), "Service name should not be empty");
            assert!(!service.image.is_empty(), "Service image should not be empty");
            // Status can be any valid enum value
            // Ports may be empty (no ports exposed)
        }

        // Render view
        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        });

        assert!(window.is_ok());

        // TODO: When element querying is available:
        // - Verify ServiceCard components rendered
        // - Check status badge colors
        // - Verify port display
        // - Check service grouping
    });
}

#[gpui::test]
async fn test_dockers_view_handles_different_service_states(cx: &mut TestAppContext) {
    // This test will be more useful when we can set service status
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Built-in services should have default "Stopped" status
        // TODO: Once Docker polling is implemented, test:
        // - Running services (green badge)
        // - Stopped services (gray badge)
        // - Error services (red badge)
        // - Starting services (yellow badge)

        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        });

        assert!(window.is_ok());
    });
}

#[gpui::test]
async fn test_dockers_view_reactive_updates(cx: &mut TestAppContext) {
    // Setup: Create state model for reactive updates
    let state_model = cx.new_model(|_cx| {
        let mut s = AppState::new();
        s.initialize();
        s
    });

    cx.update(|cx| {
        // Initial render
        let initial_services = state_model.read(cx).get_docker_services();
        let initial_count = initial_services.len();

        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(initial_services, theme)
        });

        assert!(window.is_ok());

        // TODO: When background polling is implemented:
        // 1. Trigger Docker status poll
        // 2. Update state model with new service statuses
        // 3. Re-render view
        // 4. Verify UI reflects updated statuses

        assert!(initial_count > 0);
    });
}

#[gpui::test]
async fn test_dockers_view_service_grouping(cx: &mut TestAppContext) {
    // Test that services are properly grouped by project
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Verify services exist
        assert!(!services.is_empty());

        // TODO: Once we can query rendered structure:
        // - Verify services grouped by project
        // - Check group headers
        // - Verify sorting within groups

        let window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        });

        assert!(window.is_ok());
    });
}

#[gpui::test]
async fn test_dockers_view_theme_compatibility(cx: &mut TestAppContext) {
    // Test rendering with different themes
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Test dark theme
        let dark_window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services.clone(), theme)
        });
        assert!(dark_window.is_ok(), "Should render with dark theme");

        // Test light theme
        let light_window = cx.open_window(WindowOptions::default(), |_cx| {
            let theme = MaterialTheme::light();
            DockersView::new(services.clone(), theme)
        });
        assert!(light_window.is_ok(), "Should render with light theme");
    });
}

// TODO: Add these tests when interactive features are implemented
//
// #[gpui::test]
// async fn test_start_service_button_click(cx: &mut TestAppContext) {
//     // Test clicking "Start" button triggers Docker command
// }
//
// #[gpui::test]
// async fn test_stop_service_button_click(cx: &mut TestAppContext) {
//     // Test clicking "Stop" button triggers Docker command
// }
//
// #[gpui::test]
// async fn test_service_status_updates_from_polling(cx: &mut TestAppContext) {
//     // Test background polling updates service status
// }
//
// #[gpui::test]
// async fn test_view_logs_button_click(cx: &mut TestAppContext) {
//     // Test clicking "View Logs" opens log panel
// }

//! Test GPUI without using #[gpui::test] macro
//!
//! This test bypasses the gpui::test proc macro to determine if the macro
//! itself is causing the SIGBUS error.

#![recursion_limit = "1024"]

use gpui::*;

#[tokio::test]
async fn test_gpui_manual_context() {
    // Manually create TestAppContext without using gpui::test macro
    let app = Application::production().unwrap();

    // Create a minimal test
    app.run(|cx: &mut gpui::App| {
        // Just verify we can create the context
        assert!(true, "Successfully created GPUI app context");

        // Exit immediately
        cx.quit();
    });
}

#[test]
fn test_basic_rust_test() {
    // Simplest possible test - just verify Rust testing works
    assert_eq!(2 + 2, 4);
}

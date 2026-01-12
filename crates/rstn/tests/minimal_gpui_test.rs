//! Minimal GPUI test to isolate SIGBUS issue
//!
//! This test file contains the simplest possible GPUI test
//! to determine if the issue is with:
//! 1. GPUI test macro itself
//! 2. Our test setup
//! 3. Project configuration

#![recursion_limit = "2048"]

use gpui::*;

/// Minimal test - no imports, no rendering, just GPUI context
#[gpui::test]
async fn test_minimal_gpui_context(cx: &mut TestAppContext) {
    cx.update(|_cx| {
        // Do nothing - just test if macro works
        assert!(true);
    });
}

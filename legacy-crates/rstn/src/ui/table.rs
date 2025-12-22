//! Table formatting for test results

use crate::TestResults;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

/// Display test summary as a table
pub fn display_test_summary(results: &TestResults) {
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header
    table.set_header(vec![
        Cell::new("Status").fg(Color::Cyan),
        Cell::new("Count").fg(Color::Cyan),
    ]);

    // Passed tests
    table.add_row(vec![
        Cell::new("Passed").fg(Color::Green),
        Cell::new(results.passed.to_string()),
    ]);

    // Failed tests (if any)
    if results.failed > 0 {
        table.add_row(vec![
            Cell::new("Failed").fg(Color::Red),
            Cell::new(results.failed.to_string()),
        ]);
    }

    // Ignored tests (if any)
    if results.ignored > 0 {
        table.add_row(vec![
            Cell::new("Ignored").fg(Color::Yellow),
            Cell::new(results.ignored.to_string()),
        ]);
    }

    // Filtered out (if any)
    if results.filtered_out > 0 {
        table.add_row(vec![
            Cell::new("Filtered Out").fg(Color::Cyan),
            Cell::new(results.filtered_out.to_string()),
        ]);
    }

    // Total
    table.add_row(vec![
        Cell::new("Total").fg(Color::Cyan),
        Cell::new(results.total().to_string()),
    ]);

    println!("{table}");
    println!();

    // Overall status
    if results.failed == 0 {
        println!(
            "{}",
            format!("✓ All tests passed ({} tests)", results.total())
                .green()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "✗ {} test{} failed",
                results.failed,
                if results.failed == 1 { "" } else { "s" }
            )
            .red()
            .bold()
        );
    }
    println!();
}

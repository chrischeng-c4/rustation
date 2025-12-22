//! Test command implementation

use crate::domain::test;
use crate::ui::table;
use crate::{Result, RscliError};
use colored::Colorize;

pub async fn run(
    filter: Option<&str>,
    lib_only: bool,
    integration_only: bool,
    _watch: bool, // TODO: Implement watch mode in Phase 3
    test_verbose: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("{}", "Running tests...".bright_blue());
    }

    // Call core test module
    let results = test::run_tests(filter, lib_only, integration_only, test_verbose)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    // Display results
    if test_verbose {
        // In verbose mode, cargo already printed everything
        println!();
    }

    // Display summary
    table::display_test_summary(&results);

    // Exit with error if tests failed
    if results.failed > 0 {
        return Err(RscliError::TestFailed(format!(
            "{}/{} tests failed",
            results.failed,
            results.total()
        )));
    }

    Ok(())
}

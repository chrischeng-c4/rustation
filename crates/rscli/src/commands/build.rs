//! Build command implementations

use crate::runners::cargo;
use crate::{Result, RscliError};
use colored::Colorize;

pub async fn run(release: bool, verbose: bool) -> Result<()> {
    println!(
        "{}",
        format!(
            "Building rush {}...",
            if release { "(release)" } else { "(debug)" }
        )
        .bright_blue()
    );

    let output = cargo::build(release, verbose).await?;

    if !output.status.success() {
        return Err(RscliError::BuildFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    println!("{}", "✓ Build successful".green());
    Ok(())
}

pub async fn check(verbose: bool) -> Result<()> {
    println!("{}", "Checking rush...".bright_blue());

    let output = cargo::check(verbose).await?;

    if !output.status.success() {
        return Err(RscliError::BuildFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    println!("{}", "✓ Check complete".green());
    Ok(())
}

pub async fn lint(fix: bool, verbose: bool) -> Result<()> {
    println!("{}", "Running clippy...".bright_blue());

    let output = cargo::clippy(fix, verbose).await?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(RscliError::BuildFailed("Clippy found issues".to_string()));
    }

    println!("{}", "✓ Clippy passed".green());
    Ok(())
}

pub async fn fmt(check: bool, verbose: bool) -> Result<()> {
    println!(
        "{}",
        if check {
            "Checking formatting...".bright_blue()
        } else {
            "Formatting code...".bright_blue()
        }
    );

    let output = cargo::fmt(check, verbose).await?;

    if !output.status.success() {
        return Err(RscliError::BuildFailed(
            "Formatting check failed".to_string(),
        ));
    }

    println!(
        "{}",
        if check {
            "✓ Formatting OK".green()
        } else {
            "✓ Code formatted".green()
        }
    );
    Ok(())
}

pub async fn ci(fix: bool, verbose: bool) -> Result<()> {
    println!("{}", "Running CI checks...".bright_blue().bold());
    println!();

    // 1. Tests
    println!("{}", "[1/4] Running tests...".bright_cyan());
    let test_results = cargo::run_tests(None, false, false, false).await?;
    if test_results.failed > 0 {
        println!("{}", "✗ Tests failed".red());
        return Err(RscliError::TestFailed(format!(
            "{} tests failed",
            test_results.failed
        )));
    }
    println!(
        "{}",
        format!("✓ Tests passed ({}/{})", test_results.passed, test_results.total()).green()
    );
    println!();

    // 2. Clippy
    println!("{}", "[2/4] Running clippy...".bright_cyan());
    lint(fix, verbose).await?;
    println!();

    // 3. Format
    println!("{}", "[3/4] Checking formatting...".bright_cyan());
    fmt(true, verbose).await?;
    println!();

    // 4. Build
    println!("{}", "[4/4] Building project...".bright_cyan());
    run(false, verbose).await?;
    println!();

    // Summary
    println!("{}", "CI Summary:".bright_blue().bold());
    println!("  {} Tests: {}/{} passed", "✓".green(), test_results.passed, test_results.total());
    println!("  {} Lint: OK", "✓".green());
    println!("  {} Format: OK", "✓".green());
    println!("  {} Build: OK", "✓".green());
    println!();
    println!("{}", "Overall: PASS".green().bold());
    println!("{}", "→ Ready to commit".bright_blue());

    Ok(())
}

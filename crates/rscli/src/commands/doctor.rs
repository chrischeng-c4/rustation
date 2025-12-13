//! Health check command

use crate::Result;
use colored::Colorize;
use std::process::Command;
use which::which;

pub async fn run(_verbose: bool) -> Result<()> {
    println!("{}", "Rust Station Health Check".bright_blue().bold());
    println!("{}", "=".repeat(50));
    println!();

    let mut all_ok = true;

    // Check Rust/Cargo
    println!("{}", "Environment:".bright_cyan());
    all_ok &= check_command("rustc", &["--version"], "Rust");
    all_ok &= check_command("cargo", &["--version"], "Cargo");
    all_ok &= check_command("git", &["--version"], "Git");
    all_ok &= check_command("python3", &["--version"], "Python");
    println!();

    // Check project structure
    println!("{}", "Project:".bright_cyan());
    all_ok &= check_file_exists("Cargo.toml", "Workspace manifest");
    all_ok &= check_file_exists("crates/rush/Cargo.toml", "Rush crate");
    all_ok &= check_file_exists("specs/features.json", "Feature catalog");
    all_ok &= check_file_exists(".specify/scripts/bash/common.sh", "Spec-kit scripts");
    println!();

    // Check development tools
    println!("{}", "Development Tools:".bright_cyan());
    check_optional_command("cargo-watch", &["--version"], "cargo-watch");
    check_optional_command("gh", &["--version"], "GitHub CLI");
    check_optional_command("uv", &["--version"], "uv (Python package manager)");
    check_optional_command("claude", &["--version"], "Claude CLI");
    check_optional_command("wezterm", &["--version"], "WezTerm");
    check_optional_command("gitui", &["--version"], "GitUI");
    println!();

    // Check rscli features
    println!("{}", "Rscli Features:".bright_cyan());
    check_file_exists("crates/rscli-core/Cargo.toml", "rscli-core library");
    check_file_exists("crates/rscli-tui/Cargo.toml", "rscli-tui library");
    check_file_exists(".claude/mcp-registry.json", "MCP registry");
    println!();

    // Overall status
    if all_ok {
        println!("{}", "Overall: ✓ Ready for development".green().bold());
    } else {
        println!("{}", "Overall: ⚠ Some issues found".yellow().bold());
    }

    Ok(())
}

fn check_command(cmd: &str, args: &[&str], name: &str) -> bool {
    match which(cmd) {
        Ok(_path) => {
            if let Ok(output) = Command::new(cmd).args(args).output() {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let version = output_str
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim();
                    println!("  {} {}: {}", "✓".green(), name, version);
                    return true;
                }
            }
            println!("  {} {} found but not working", "✗".red(), name);
            false
        }
        Err(_) => {
            println!("  {} {} not found", "✗".red(), name);
            false
        }
    }
}

fn check_optional_command(cmd: &str, args: &[&str], name: &str) -> bool {
    match which(cmd) {
        Ok(_path) => {
            if let Ok(output) = Command::new(cmd).args(args).output() {
                if output.status.success() {
                    println!("  {} {} installed", "✓".green(), name);
                    return true;
                }
            }
            println!("  {} {} found but not working", "⚠".yellow(), name);
            false
        }
        Err(_) => {
            println!("  {} {} not installed (optional)", "ℹ".bright_blue(), name);
            false
        }
    }
}

fn check_file_exists(path: &str, name: &str) -> bool {
    if std::path::Path::new(path).exists() {
        println!("  {} {}", "✓".green(), name);
        true
    } else {
        println!("  {} {} not found", "✗".red(), name);
        false
    }
}

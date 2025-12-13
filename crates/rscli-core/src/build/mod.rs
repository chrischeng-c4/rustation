//! Build operations (cargo build, check, clippy, fmt)

use crate::errors::{CoreError, Result};
use std::process::Output;
use tokio::process::Command;

/// Run cargo build
pub async fn build(release: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("-p").arg("rush");

    if release {
        cmd.arg("--release");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| CoreError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo check
pub async fn check(verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.arg("-p").arg("rush");

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| CoreError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo clippy
pub async fn clippy(fix: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    cmd.arg("--all-targets");
    cmd.arg("--all-features");

    if fix {
        cmd.arg("--fix");
        cmd.arg("--allow-dirty");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| CoreError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo fmt
pub async fn fmt(check: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");

    if check {
        cmd.arg("--check");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| CoreError::CommandNotFound(format!("cargo: {}", e)))
}

/// Command output collected for display
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    pub lines: Vec<String>,
    pub success: bool,
}

/// Run a generic cargo-style command and collect output
/// Returns collected output for TUI display (doesn't print to stdout)
pub async fn run_cargo_command(name: &str, args: &[String]) -> Result<CommandOutput> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut cmd = Command::new("cargo");

    // Map command names to cargo subcommands
    match name {
        "test" => {
            cmd.arg("test").arg("-p").arg("rush");
            for arg in args {
                if arg == "--lib" {
                    cmd.arg("--lib");
                } else if arg == "--integration" {
                    cmd.arg("--test").arg("*");
                } else {
                    cmd.arg(arg);
                }
            }
        }
        "build" => {
            cmd.arg("build").arg("-p").arg("rush");
            if args.contains(&"--release".to_string()) {
                cmd.arg("--release");
            }
        }
        "check" => {
            cmd.arg("check").arg("-p").arg("rush");
        }
        "lint" => {
            cmd.arg("clippy").arg("--all-targets").arg("--all-features");
            if args.contains(&"--fix".to_string()) {
                cmd.arg("--fix").arg("--allow-dirty");
            }
        }
        "fmt" => {
            cmd.arg("fmt");
            if args.contains(&"--check".to_string()) {
                cmd.arg("--check");
            }
        }
        "ci" => {
            // CI runs multiple commands - just run clippy for now
            cmd.arg("clippy")
                .arg("--all-targets")
                .arg("--all-features")
                .arg("--")
                .arg("-D")
                .arg("warnings");
        }
        _ => {
            return Err(CoreError::CommandNotFound(format!("Unknown command: {}", name)));
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| CoreError::CommandFailed(e.to_string()))?;

    let mut output = CommandOutput::default();

    // Read stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            output.lines.push(line);
        }
    }

    // Read stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            output.lines.push(line);
        }
    }

    let status = child.wait().await?;
    output.success = status.success();
    Ok(output)
}

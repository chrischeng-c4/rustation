//! Bash script executor

use crate::{Result, RscliError};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Get the repository root directory
fn get_repo_root() -> Result<PathBuf> {
    // Try to get git repository root
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)
            .map_err(|e| RscliError::Other(e.into()))?
            .trim()
            .to_string();
        Ok(PathBuf::from(path))
    } else {
        // Fallback to current directory
        std::env::current_dir().map_err(Into::into)
    }
}

/// Execute a bash script from .specify/scripts/bash/
pub async fn run_script(script_name: &str, args: &[&str]) -> Result<String> {
    let repo_root = get_repo_root()?;
    let script_path = repo_root
        .join(".specify/scripts/bash")
        .join(format!("{}.sh", script_name));

    if !script_path.exists() {
        return Err(RscliError::Other(anyhow::anyhow!(
            "Script not found: {}",
            script_path.display()
        )));
    }

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(RscliError::Other(anyhow::anyhow!(
            "Script failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Execute a bash script and print output in real-time
pub async fn run_script_interactive(script_name: &str, args: &[&str]) -> Result<()> {
    let repo_root = get_repo_root()?;
    let script_path = repo_root
        .join(".specify/scripts/bash")
        .join(format!("{}.sh", script_name));

    if !script_path.exists() {
        return Err(RscliError::Other(anyhow::anyhow!(
            "Script not found: {}",
            script_path.display()
        )));
    }

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    cmd.args(args);

    let status = cmd.status().await?;

    if status.success() {
        Ok(())
    } else {
        Err(RscliError::Other(anyhow::anyhow!("Script failed")))
    }
}

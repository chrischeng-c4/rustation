//! Python speckit wrapper

use crate::{Result, RscliError};
use tokio::process::Command;

/// Execute Python speckit command
pub async fn run_speckit(command: &str, args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("python3");
    cmd.arg("-m").arg("speckit");
    cmd.arg(command);
    cmd.args(args);

    let status = cmd.status().await?;

    if status.success() {
        Ok(())
    } else {
        Err(RscliError::Other(anyhow::anyhow!(
            "Spec-kit command failed"
        )))
    }
}

/// Execute Python speckit command and capture output
pub async fn run_speckit_with_output(command: &str, args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("python3");
    cmd.arg("-m").arg("speckit");
    cmd.arg(command);
    cmd.args(args);

    let output = cmd.output().await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(RscliError::Other(anyhow::anyhow!(
            "Spec-kit command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

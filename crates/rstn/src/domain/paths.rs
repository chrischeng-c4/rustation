//! XDG-compliant path management for rustation

use crate::domain::errors::{CoreError, Result};
use std::path::PathBuf;

const APP_NAME: &str = "rustation";

/// Get config directory (~/.config/rustation)
pub fn config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| CoreError::Config("Could not determine config directory".into()))
}

/// Get data directory (~/.local/share/rustation)
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| CoreError::Config("Could not determine data directory".into()))
}

/// Get cache directory (~/.cache/rustation)
pub fn cache_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| CoreError::Config("Could not determine cache directory".into()))
}

/// Get state directory (~/.local/state/rustation)
pub fn state_dir() -> Result<PathBuf> {
    dirs::state_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| CoreError::Config("Could not determine state directory".into()))
}

/// Ensure all directories exist
pub fn ensure_dirs() -> Result<()> {
    for dir in [config_dir()?, data_dir()?, cache_dir()?, state_dir()?] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

// Convenience functions for common paths

/// Get services config file path
pub fn services_config() -> Result<PathBuf> {
    Ok(config_dir()?.join("services.yaml"))
}

/// Get MCP config file path
pub fn mcp_config() -> Result<PathBuf> {
    Ok(config_dir()?.join("mcp-servers.json"))
}

/// Get service logs directory
pub fn service_logs_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("logs/services"))
}

/// Get MCP logs directory
pub fn mcp_logs_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("logs/mcp"))
}

/// Get rustation home directory (~/.rustation)
pub fn rustation_home() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".rustation"))
        .ok_or_else(|| CoreError::Config("Could not determine home directory".into()))
}

/// Get rstn logs directory (~/.rustation/logs)
pub fn rstn_logs_dir() -> Result<PathBuf> {
    Ok(rustation_home()?.join("logs"))
}

/// Get rstn log file path (~/.rustation/logs/rstn.log)
pub fn rstn_log_file() -> Result<PathBuf> {
    Ok(rstn_logs_dir()?.join("rstn.log"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_creation() {
        // Just test that we can call the functions without panicking
        let _ = config_dir();
        let _ = data_dir();
        let _ = cache_dir();
        let _ = state_dir();
    }
}

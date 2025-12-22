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

/// Get rstn home directory (~/.rstn)
pub fn rstn_home() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".rstn"))
        .ok_or_else(|| CoreError::Config("Could not determine home directory".into()))
}

/// Get rustation home directory (~/.rustation) - DEPRECATED, for migration fallback only
#[deprecated(note = "Use rstn_home() instead. Kept for migration fallback.")]
pub fn rustation_home() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".rustation"))
        .ok_or_else(|| CoreError::Config("Could not determine home directory".into()))
}

/// Get MCP config file path (~/.rstn/mcp-session.json)
pub fn mcp_config_path() -> Result<PathBuf> {
    Ok(rstn_home()?.join("mcp-session.json"))
}

/// Get sessions directory (~/.rstn/sessions)
pub fn sessions_dir() -> Result<PathBuf> {
    Ok(rstn_home()?.join("sessions"))
}

/// Get settings file path (~/.rstn/settings.json)
pub fn settings_path() -> Result<PathBuf> {
    Ok(rstn_home()?.join("settings.json"))
}

/// Get rstn logs directory (~/.rstn/logs)
pub fn rstn_logs_dir() -> Result<PathBuf> {
    Ok(rstn_home()?.join("logs"))
}

/// Get rstn log file path (~/.rstn/logs/rstn.log)
pub fn rstn_log_file() -> Result<PathBuf> {
    Ok(rstn_logs_dir()?.join("rstn.log"))
}

/// Get paste temp directory (~/.rstn/tmp/pastes)
pub fn paste_temp_dir() -> Result<PathBuf> {
    Ok(rstn_home()?.join("tmp/pastes"))
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

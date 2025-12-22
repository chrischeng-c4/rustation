//! Unified Claude CLI discovery and caching
//!
//! Provides a single source of truth for finding the Claude executable,
//! with persistent caching to avoid repeated searches.

use crate::domain::errors::{CoreError, Result};
use crate::settings::Settings;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Claude CLI discovery and validation
pub struct ClaudeDiscovery;

impl ClaudeDiscovery {
    /// Find Claude CLI executable with caching
    ///
    /// Search order:
    /// 1. Check cached path in settings (if valid)
    /// 2. Search PATH via which
    /// 3. Search common installation locations:
    ///    - ~/.claude/local/claude
    ///    - ~/.local/bin/claude
    ///    - /usr/local/bin/claude
    ///    - /opt/homebrew/bin/claude
    /// 4. Cache result to settings
    ///
    /// Returns the path to claude executable
    pub async fn find_claude() -> Result<PathBuf> {
        // 1. Try cached path from settings
        if let Some(path) = Self::load_from_settings().await? {
            if Self::validate_path(&path).await {
                debug!("Using cached Claude path: {}", path.display());
                return Ok(path);
            } else {
                warn!(
                    "Cached Claude path is invalid, re-searching: {}",
                    path.display()
                );
            }
        }

        // 2. Search for Claude
        let path = Self::search_locations().await?;

        // 3. Cache the result
        Self::save_to_settings(&path).await?;

        info!("Found and cached Claude CLI: {}", path.display());
        Ok(path)
    }

    /// Load cached path from settings
    async fn load_from_settings() -> Result<Option<PathBuf>> {
        Ok(Settings::load().claude_path.map(PathBuf::from))
    }

    /// Save path to settings cache
    async fn save_to_settings(path: &Path) -> Result<()> {
        let mut settings = Settings::load();
        settings.claude_path = Some(path.to_string_lossy().to_string());
        settings.save()?;
        Ok(())
    }

    /// Validate that a path points to a working Claude executable
    async fn validate_path(path: &Path) -> bool {
        // Check file exists
        if !path.exists() {
            return false;
        }

        // Check it's executable (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path).await {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 == 0 {
                    return false; // Not executable
                }
            } else {
                return false;
            }
        }

        // Verify the binary is functional by running --version
        if let Ok(output) = tokio::process::Command::new(path)
            .arg("--version")
            .output()
            .await
        {
            output.status.success()
        } else {
            false
        }
    }

    /// Search common locations for Claude CLI
    async fn search_locations() -> Result<PathBuf> {
        debug!("Searching for Claude CLI executable");

        // 1. Check PATH via which (fastest, most common)
        if let Ok(path) = which::which("claude") {
            debug!("Found claude in PATH: {:?}", path);
            return Ok(path);
        }

        // 2. Check common installation locations
        let home = std::env::var("HOME").map_err(|_| {
            CoreError::CommandFailed("Could not determine HOME directory".to_string())
        })?;

        let locations = vec![
            format!("{}/.claude/local/claude", home),
            format!("{}/.local/bin/claude", home),
            "/usr/local/bin/claude".to_string(),
            "/opt/homebrew/bin/claude".to_string(),
        ];

        for location in &locations {
            debug!("Checking: {}", location);
            let path = PathBuf::from(location);
            if Self::validate_path(&path).await {
                info!("Found Claude at: {}", location);
                return Ok(path);
            }
        }

        // 3. Not found - provide helpful error
        Err(CoreError::CommandFailed(format!(
            "Claude CLI not found. Searched:\n\
             - PATH (via which)\n\
             - {}\n\
             \n\
             Please install Claude Code CLI:\n\
             https://code.claude.com/download\n\
             \n\
             Or create a symlink to your installation:\n\
             ln -s /path/to/claude ~/.local/bin/claude",
            locations.join("\n - ")
        )))
    }

    /// Clear cached path (force re-discovery)
    pub async fn clear_cache() -> Result<()> {
        let mut settings = Settings::load();
        settings.claude_path = None;
        settings.save()?;
        info!("Cleared Claude CLI cache");
        Ok(())
    }

    /// Get current cached path without searching
    pub fn get_cached_path() -> Option<PathBuf> {
        Settings::load().claude_path.map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/claude");
        assert!(!ClaudeDiscovery::validate_path(&path).await);
    }

    #[tokio::test]
    async fn test_search_finds_claude_or_errors() {
        // This test will pass if claude is installed, or return helpful error if not
        match ClaudeDiscovery::search_locations().await {
            Ok(path) => {
                // Claude found, verify it's a path
                assert!(path.to_string_lossy().contains("claude"));
            }
            Err(e) => {
                // Claude not found, verify error message is helpful
                let msg = format!("{:?}", e);
                assert!(msg.contains("not found") || msg.contains("PATH"));
                assert!(msg.contains("install") || msg.contains("symlink"));
            }
        }
    }
}

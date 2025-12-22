//! Session management commands
//!
//! Provides CLI commands for querying and managing rstn session history:
//! - `rstn session list` - Display recent sessions in table format
//! - `rstn session info <id>` - Show detailed session information
//! - `rstn session logs <id>` - Display log file contents
//! - `rstn session cleanup` - Remove old sessions

use crate::session_manager::SessionManager;
use crate::{Result, RscliError};
use chrono::{Local, TimeZone};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ============================================================================
// Helper Functions
// ============================================================================

/// Format Unix timestamp as human-readable datetime
/// Example: "2024-12-19 10:30:15"
fn format_timestamp(ts: i64) -> String {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Invalid timestamp".to_string())
}

/// Format Unix timestamp as relative time
/// Examples: "2h ago", "3d ago", "just now"
fn format_relative_time(ts: i64) -> String {
    let now = Local::now().timestamp();
    let diff = now - ts;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

/// Resolve session ID from prefix (with ambiguity checking)
///
/// Supports:
/// - Full session ID (exact match)
/// - Short prefix (e.g., first 8 chars)
///
/// Returns error if:
/// - No sessions match
/// - Multiple sessions match (ambiguous)
fn resolve_session_id(manager: &SessionManager, prefix: &str) -> Result<String> {
    // Try exact match first
    if let Ok(Some(_)) = manager.get_session(prefix) {
        return Ok(prefix.to_string());
    }

    // Try prefix match
    let sessions = manager.list_recent_sessions(10000)?;
    let matches: Vec<_> = sessions
        .into_iter()
        .filter(|s| s.session_id.starts_with(prefix))
        .collect();

    match matches.len() {
        0 => Err(RscliError::Other(anyhow::anyhow!(
            "No session found matching '{}'",
            prefix
        ))),
        1 => Ok(matches[0].session_id.clone()),
        _ => {
            let ids: Vec<_> = matches.iter().map(|s| s.session_id.as_str()).collect();
            Err(RscliError::Other(anyhow::anyhow!(
                "Ambiguous session ID '{}'. Matches: {}",
                prefix,
                ids.join(", ")
            )))
        }
    }
}

/// Get color for session status
fn status_color(status: &str) -> TableColor {
    match status {
        "completed" => TableColor::Green,
        "active" => TableColor::Yellow,
        "error" | "failed" => TableColor::Red,
        _ => TableColor::Grey,
    }
}

// ============================================================================
// Command Implementations
// ============================================================================

/// List recent sessions in table format
///
/// # Arguments
/// - `limit`: Maximum number of sessions to display
/// - `filter_type`: Optional command type filter (e.g., "prompt")
/// - `filter_status`: Optional status filter (e.g., "completed")
pub async fn list(
    limit: usize,
    filter_type: Option<String>,
    filter_status: Option<String>,
) -> Result<()> {
    let manager = SessionManager::open()?;
    let sessions = manager.list_recent_sessions(limit)?;

    // Apply filters (in-memory for MVP)
    let filtered: Vec<_> = sessions
        .into_iter()
        .filter(|s| filter_type.as_ref().is_none_or(|t| &s.command_type == t))
        .filter(|s| filter_status.as_ref().is_none_or(|st| &s.status == st))
        .collect();

    if filtered.is_empty() {
        println!("{}", "No sessions found matching filters.".bright_yellow());
        return Ok(());
    }

    // Render table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Type", "Created", "Status", "Logs"]);

    for session in &filtered {
        let id_short = &session.session_id[..8.min(session.session_id.len())];
        let created = format_relative_time(session.created_at);
        let logs_icon = if session.log_file.is_some() {
            "✓"
        } else {
            "✗"
        };

        table.add_row(vec![
            Cell::new(id_short),
            Cell::new(&session.command_type),
            Cell::new(created),
            Cell::new(&session.status).fg(status_color(&session.status)),
            Cell::new(logs_icon),
        ]);
    }

    println!("{table}");
    println!("\nTotal: {} sessions", filtered.len());

    if filter_type.is_some() || filter_status.is_some() {
        println!(
            "{}",
            "Tip: Remove filters to see all sessions".bright_black()
        );
    }

    Ok(())
}

/// Display detailed information about a specific session
///
/// # Arguments
/// - `session_id`: Full session ID or prefix
pub async fn info(session_id: String) -> Result<()> {
    let manager = SessionManager::open()?;
    let full_id = resolve_session_id(&manager, &session_id)?;
    let session = manager
        .get_session(&full_id)?
        .ok_or_else(|| RscliError::Other(anyhow::anyhow!("Session not found: {}", full_id)))?;

    println!("\n{}", "Session Information".bright_blue().bold());
    println!("═══════════════════\n");
    println!("Session ID:    {}", session.session_id);
    println!("Command Type:  {}", session.command_type);
    println!("Created:       {}", format_timestamp(session.created_at));
    println!(
        "               ({})",
        format_relative_time(session.created_at)
    );

    let status_colored = match session.status.as_str() {
        "completed" => session.status.green(),
        "active" => session.status.yellow(),
        "error" | "failed" => session.status.red(),
        _ => session.status.normal(),
    };
    println!("Status:        {}", status_colored);

    if let Some(ref log_file) = session.log_file {
        if Path::new(log_file).exists() {
            let metadata = fs::metadata(log_file)?;
            let size_kb = metadata.len() / 1024;
            println!("Log File:      {} ({} KB)", log_file, size_kb);
            println!(
                "\n{}",
                format!("View logs: rstn session logs {}", &session.session_id[..8]).bright_black()
            );
        } else {
            println!("Log File:      {} {}", log_file, "(missing)".red());
        }
    } else {
        println!("Log File:      {}", "None".bright_black());
    }

    println!();
    Ok(())
}

/// Display log file contents for a session
///
/// # Arguments
/// - `session_id`: Full session ID or prefix
/// - `tail`: Show last N lines only
/// - `head`: Show first N lines only
/// - `follow`: Follow log file (tail -f mode)
pub async fn logs(
    session_id: String,
    tail: Option<usize>,
    head: Option<usize>,
    follow: bool,
) -> Result<()> {
    let manager = SessionManager::open()?;
    let full_id = resolve_session_id(&manager, &session_id)?;
    let session = manager
        .get_session(&full_id)?
        .ok_or_else(|| RscliError::Other(anyhow::anyhow!("Session not found: {}", full_id)))?;

    let log_file = session.log_file.ok_or_else(|| {
        RscliError::Other(anyhow::anyhow!("No log file for session: {}", full_id))
    })?;

    if !Path::new(&log_file).exists() {
        return Err(RscliError::Other(anyhow::anyhow!(
            "Log file not found: {}",
            log_file
        )));
    }

    if follow {
        // tail -f implementation (future enhancement)
        eprintln!("{}", "Follow mode not yet implemented".yellow());
        eprintln!("Use: tail -f {}", log_file);
        return Ok(());
    }

    if let Some(n) = tail {
        show_tail(&log_file, n)?;
    } else if let Some(n) = head {
        show_head(&log_file, n)?;
    } else {
        show_all(&log_file)?;
    }

    Ok(())
}

/// Cleanup old sessions
///
/// # Arguments
/// - `older_than_days`: Remove sessions older than N days
/// - `dry_run`: Preview changes without deleting
/// - `force`: Delete even active sessions
pub async fn cleanup(older_than_days: u32, dry_run: bool, force: bool) -> Result<()> {
    let manager = SessionManager::open()?;
    let cutoff = Local::now().timestamp() - (older_than_days as i64 * 86400);

    let to_delete: Vec<_> = manager
        .list_recent_sessions(10000)?
        .into_iter()
        .filter(|s| s.created_at < cutoff)
        .filter(|s| force || s.status != "active")
        .collect();

    if to_delete.is_empty() {
        println!(
            "{}",
            format!("No sessions older than {} days", older_than_days).yellow()
        );
        return Ok(());
    }

    // Preview
    println!(
        "{}",
        format!("Found {} session(s) to delete:", to_delete.len()).bright_blue()
    );
    for s in &to_delete {
        let age_days = (Local::now().timestamp() - s.created_at) / 86400;
        println!(
            "  - {} ({} days old, {})",
            &s.session_id[..8],
            age_days,
            s.command_type
        );
    }

    if dry_run {
        println!("\n{}", "DRY RUN: No sessions deleted.".yellow());
        println!(
            "{}",
            "Remove --dry-run flag to actually delete.".bright_black()
        );
        return Ok(());
    }

    // Confirm
    println!(
        "\n{}",
        "This will delete session metadata and log files.".bright_yellow()
    );
    print!("{}", "Proceed? [y/N]: ".bright_yellow());
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Cancelled.".bright_black());
        return Ok(());
    }

    // Delete
    let mut deleted_count = 0;
    for s in to_delete {
        // Delete log file if exists
        if let Some(ref log_file) = s.log_file {
            if Path::new(log_file).exists() {
                if let Err(e) = fs::remove_file(log_file) {
                    eprintln!(
                        "{}",
                        format!("Warning: Failed to delete log file {}: {}", log_file, e)
                            .bright_yellow()
                    );
                }
            }
        }

        // Delete session from database
        manager.delete_session(&s.session_id)?;
        deleted_count += 1;
    }

    println!(
        "{}",
        format!("✓ Deleted {} sessions", deleted_count).green()
    );
    Ok(())
}

// ============================================================================
// Log Display Helpers
// ============================================================================

/// Show last N lines of a file
fn show_tail(path: &str, n: usize) -> Result<()> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().collect::<std::io::Result<_>>()?;

    let start = lines.len().saturating_sub(n);
    for line in &lines[start..] {
        println!("{}", line);
    }

    Ok(())
}

/// Show first N lines of a file
fn show_head(path: &str, n: usize) -> Result<()> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        if i >= n {
            break;
        }
        println!("{}", line?);
    }

    Ok(())
}

/// Show all lines of a file (with large file warning)
fn show_all(path: &str) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let size_mb = metadata.len() / (1024 * 1024);

    if size_mb > 10 {
        eprintln!(
            "{}",
            format!(
                "Warning: Log file is {} MB. This may take a while.",
                size_mb
            )
            .yellow()
        );
        eprintln!(
            "{}",
            "Tip: Use --tail or --head to limit output.".bright_black()
        );
        print!("{}", "Continue? [y/N]: ".bright_yellow());
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    let content = fs::read_to_string(path)?;
    println!("{}", content);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let ts = 1734609015; // 2024-12-19 10:30:15 UTC
        let result = format_timestamp(ts);
        assert!(result.contains("2024-12-19"));
    }

    #[test]
    fn test_format_relative_time() {
        let now = Local::now().timestamp();

        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - 120), "2m ago");
        assert_eq!(format_relative_time(now - 7200), "2h ago");
        assert_eq!(format_relative_time(now - 172800), "2d ago");
    }

    #[test]
    fn test_status_color() {
        assert_eq!(status_color("completed"), TableColor::Green);
        assert_eq!(status_color("active"), TableColor::Yellow);
        assert_eq!(status_color("error"), TableColor::Red);
        assert_eq!(status_color("failed"), TableColor::Red);
        assert_eq!(status_color("unknown"), TableColor::Grey);
    }
}

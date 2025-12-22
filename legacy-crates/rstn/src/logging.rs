//! Logging initialization for rstn
//!
//! Provides structured logging using tracing, with output to file.
//! Configuration is read from Settings.

use crate::settings::Settings;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;

/// Custom time formatter for file logs with milliseconds
/// Format: [YYYY-MM-DD HH:MM:SS.mmm]
struct MillisecondTimer;

impl fmt::time::FormatTime for MillisecondTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        use chrono::Local;
        let now = Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

/// Generate a timestamp-based session ID for log files
///
/// Returns a session ID in format: `YYYY-MM-DD-HHMMSS-random`
/// Example: `2025-12-18-142345-a3f9b2c1`
///
/// This format enables:
/// - Instant chronological sorting (timestamp prefix)
/// - Age identification at a glance
/// - Uniqueness guarantee (8-character random suffix = 4.3B possibilities)
pub fn generate_session_id() -> String {
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d-%H%M%S");
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let random = format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    );

    format!("{}-{}", timestamp, random)
}

/// Initialize logging based on settings
///
/// Logs are written to ~/.rstn/logs/rstn.<session_id>.log
/// Old logs are rotated and compressed after each session.
///
/// Returns the session ID for this execution.
pub fn init(settings: &Settings) -> String {
    if !settings.logging_enabled {
        return String::new();
    }

    let log_dir = match crate::domain::paths::rstn_logs_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Warning: Could not determine log directory: {}", e);
            return String::new();
        }
    };

    // Create log directory
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Could not create log directory: {}", e);
        return String::new();
    }

    // Generate session ID for this execution
    let session_id = generate_session_id();
    let log_filename = format!("rstn.{}.log", session_id);
    let log_file = log_dir.join(&log_filename);

    // Rotate and compress old logs (from previous sessions)
    rotate_logs(&log_dir);

    // Create file appender with session-specific filename
    let file_appender = tracing_appender::rolling::never(&log_dir, log_filename.clone());

    // Build filter from settings or RSTN_LOG env var
    let filter = EnvFilter::try_from_env("RSTN_LOG").unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "rstn={},rstn_core={},rstn_tui={}",
            settings.log_level, settings.log_level, settings.log_level
        ))
    });

    // File layer - always enabled with millisecond timestamps
    // Format: [YYYY-MM-DD HH:MM:SS.mmm] LEVEL [TARGET] Message (file:line)
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_timer(MillisecondTimer)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false);

    if settings.log_to_console {
        // Log to both file and console
        let console_layer = fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        // Log to file only
        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .init();
    }

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        session_id = %session_id,
        log_level = %settings.log_level,
        log_file = %log_file.display(),
        "rstn logging initialized"
    );

    // Return session_id so it can be used elsewhere
    session_id
}

/// Rotate old log files and compress them
///
/// Finds all uncompressed rstn.*.log files and compresses them in the background.
/// This allows each session to have its own log file that gets compressed after the session ends.
fn rotate_logs(log_dir: &PathBuf) {
    // Find all rstn.*.log files (uncompressed) that are not .gz
    let entries = match fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process uncompressed rstn.*.log files
        if !path.extension().map(|e| e == "log").unwrap_or(false) {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // Match pattern: rstn.<session_id>.log or rstn.<timestamp>.log (for old format)
        if !filename.starts_with("rstn.") || filename == "rstn.log" {
            continue;
        }

        // Check if file has content
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.len() == 0 {
            continue;
        }

        // Compress this old session log
        let compressed_path = log_dir.join(format!("{}.gz", filename));

        // Clone for background thread
        let path_clone = path.clone();
        let compressed_clone = compressed_path.clone();
        let log_dir_clone = log_dir.clone();

        std::thread::spawn(move || {
            if let Err(e) = compress_file(&path_clone, &compressed_clone) {
                eprintln!("Warning: Could not compress log file: {}", e);
            } else {
                // Remove uncompressed file after successful compression
                let _ = fs::remove_file(&path_clone);
            }

            // Clean up old logs (keep last 7 days)
            cleanup_old_logs(&log_dir_clone, 7);
        });
    }
}

/// Compress a file using gzip
fn compress_file(source: &PathBuf, dest: &PathBuf) -> std::io::Result<()> {
    let input = File::open(source)?;
    let mut reader = BufReader::new(input);

    let output = File::create(dest)?;
    let mut encoder = GzEncoder::new(BufWriter::new(output), Compression::default());

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    encoder.write_all(&buffer)?;
    encoder.finish()?;

    Ok(())
}

/// Remove log files older than specified days
fn cleanup_old_logs(log_dir: &PathBuf, days: u64) {
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 24 * 60 * 60);

    let entries = match fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .gz files
        if path.extension().map(|e| e != "gz").unwrap_or(true) {
            continue;
        }

        // Check file name matches pattern: rstn.*.log.gz or rstn.YYYYMMDD-HHMMSS.log.gz
        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("rstn.") && n.ends_with(".log.gz"))
            .unwrap_or(false)
        {
            continue;
        }

        // Check modification time
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

/// Get the log file path for display
pub fn log_file_path() -> PathBuf {
    crate::domain::paths::rstn_log_file().unwrap_or_else(|_| PathBuf::from("/tmp/rstn.log"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::io::Read as _;
    use tempfile::TempDir;

    #[test]
    fn test_log_file_path_returns_valid_path() {
        let path = log_file_path();
        assert!(path.to_string_lossy().contains("rstn.log"));
        assert!(path.to_string_lossy().contains(".rstn"));
    }

    #[test]
    fn test_compress_file_creates_valid_gzip() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("test.log");
        let dest = temp_dir.path().join("test.log.gz");

        // Create test file with content
        let content = "Hello, this is a test log file!\nLine 2\nLine 3";
        fs::write(&source, content).unwrap();

        // Compress
        compress_file(&source, &dest).unwrap();

        // Verify compressed file exists
        assert!(dest.exists());

        // Verify we can decompress and get original content
        let compressed = fs::read(&dest).unwrap();
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        assert_eq!(decompressed, content);
    }

    #[test]
    fn test_cleanup_old_logs_removes_old_files() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Create some test files
        let old_file = log_dir.join("rstn.20231201-120000.log.gz");
        let new_file = log_dir.join("rstn.20991231-235959.log.gz");
        let non_log_file = log_dir.join("other.gz");

        fs::write(&old_file, "old").unwrap();
        fs::write(&new_file, "new").unwrap();
        fs::write(&non_log_file, "other").unwrap();

        // Set old_file modification time to past (using file metadata won't work easily,
        // so we'll test with 0 days retention to delete all rstn.*.gz files)
        cleanup_old_logs(&log_dir, 0);

        // Old file should be deleted (0 days retention = delete all)
        assert!(!old_file.exists());
        // New file should also be deleted with 0 days retention
        assert!(!new_file.exists());
        // Non-rstn file should remain
        assert!(non_log_file.exists());
    }

    #[test]
    fn test_cleanup_preserves_non_gz_files() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Create non-gz file
        let log_file = log_dir.join("rstn.log");
        fs::write(&log_file, "current log").unwrap();

        cleanup_old_logs(&log_dir, 0);

        // Non-gz file should remain
        assert!(log_file.exists());
    }

    #[test]
    fn test_settings_defaults_enable_logging() {
        let settings = Settings::default();
        assert!(settings.logging_enabled);
        // Debug builds default to "trace", release builds to "info"
        #[cfg(debug_assertions)]
        assert_eq!(settings.log_level, "trace");
        #[cfg(not(debug_assertions))]
        assert_eq!(settings.log_level, "info");
        assert!(!settings.log_to_console);
    }

    #[test]
    fn test_session_id_format() {
        let session_id = generate_session_id();

        // Validate format: YYYY-MM-DD-HHMMSS-random
        // Example: 2025-12-18-142345-a3f9b2c1
        let pattern = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}-\d{6}-[0-9a-f]{8}$").unwrap();
        assert!(
            pattern.is_match(&session_id),
            "Session ID '{}' does not match expected format YYYY-MM-DD-HHMMSS-random",
            session_id
        );

        // Validate timestamp components are reasonable
        let parts: Vec<&str> = session_id.split('-').collect();
        assert_eq!(parts.len(), 5, "Session ID should have 5 parts");

        let year: u32 = parts[0].parse().unwrap();
        let month: u32 = parts[1].parse().unwrap();
        let day: u32 = parts[2].parse().unwrap();
        let time: u32 = parts[3].parse().unwrap();
        let random = parts[4];

        assert!(year >= 2025 && year <= 2100, "Year should be reasonable");
        assert!(month >= 1 && month <= 12, "Month should be 1-12");
        assert!(day >= 1 && day <= 31, "Day should be 1-31");
        assert!(time <= 235959, "Time should be valid HHMMSS");
        assert_eq!(random.len(), 8, "Random suffix should be 8 hex chars");
    }

    #[test]
    fn test_session_id_uniqueness() {
        use std::collections::HashSet;

        let mut ids = HashSet::new();

        // Generate 1000 session IDs
        for _ in 0..1000 {
            let session_id = generate_session_id();
            // All IDs should be unique (HashSet insert returns false if duplicate)
            assert!(
                ids.insert(session_id.clone()),
                "Duplicate session ID generated: {}",
                session_id
            );
        }

        assert_eq!(ids.len(), 1000, "Should have 1000 unique session IDs");
    }

    #[test]
    fn test_session_id_chronological_sorting() {
        use std::thread::sleep;
        use std::time::Duration;

        let id1 = generate_session_id();
        sleep(Duration::from_millis(1100)); // Sleep > 1 second to ensure different timestamp
        let id2 = generate_session_id();

        // IDs should sort chronologically due to timestamp prefix
        assert!(
            id1 < id2,
            "Session IDs should sort chronologically: '{}' should be < '{}'",
            id1,
            id2
        );
    }

    #[test]
    fn test_millisecond_timer_format() {
        use tracing_subscriber::fmt::time::FormatTime;

        let timer = MillisecondTimer;
        let mut writer = String::new();
        let mut fmt_writer = fmt::format::Writer::new(&mut writer);

        // Format time and verify it matches expected pattern
        timer.format_time(&mut fmt_writer).unwrap();

        // Should match: YYYY-MM-DD HH:MM:SS.mmm
        // Example: 2025-12-18 14:23:45.123
        let pattern = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}$").unwrap();
        assert!(
            pattern.is_match(&writer),
            "Timestamp '{}' does not match expected format YYYY-MM-DD HH:MM:SS.mmm",
            writer
        );

        // Verify timestamp components are reasonable
        // Format: 2025-12-18 14:23:45.123
        let parts: Vec<&str> = writer.split(&[' ', ':', '.'][..]).collect();
        assert_eq!(parts.len(), 5, "Should have 5 parts after split");

        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3, "Date should have 3 parts");
        let year: u32 = date_parts[0].parse().unwrap();
        let month: u32 = date_parts[1].parse().unwrap();
        let day: u32 = date_parts[2].parse().unwrap();

        let hour: u32 = parts[1].parse().unwrap();
        let minute: u32 = parts[2].parse().unwrap();
        let second: u32 = parts[3].parse().unwrap();
        let millis: u32 = parts[4].parse().unwrap();

        assert!(year >= 2025 && year <= 2100, "Year should be reasonable");
        assert!(month >= 1 && month <= 12, "Month should be 1-12");
        assert!(day >= 1 && day <= 31, "Day should be 1-31");
        assert!(hour <= 23, "Hour should be 0-23");
        assert!(minute <= 59, "Minute should be 0-59");
        assert!(second <= 59, "Second should be 0-59");
        assert!(millis <= 999, "Milliseconds should be 0-999");
    }
}

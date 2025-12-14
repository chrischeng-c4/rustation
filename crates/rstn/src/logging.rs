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

/// Initialize logging based on settings
///
/// Logs are written to ~/.rustation/logs/rstn.log
/// Old logs are rotated and compressed daily.
pub fn init(settings: &Settings) {
    if !settings.logging_enabled {
        return;
    }

    let log_dir = match rstn_core::paths::rstn_logs_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Warning: Could not determine log directory: {}", e);
            return;
        }
    };

    // Create log directory
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Could not create log directory: {}", e);
        return;
    }

    // Rotate and compress old logs before starting new session
    rotate_logs(&log_dir);

    let log_file = log_dir.join("rstn.log");

    // Create file appender
    let file_appender = tracing_appender::rolling::never(&log_dir, "rstn.log");

    // Build filter from settings or RSTN_LOG env var
    let filter = EnvFilter::try_from_env("RSTN_LOG").unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "rstn={},rstn_core={},rstn_tui={}",
            settings.log_level, settings.log_level, settings.log_level
        ))
    });

    // File layer - always enabled
    let file_layer = fmt::layer()
        .with_writer(file_appender)
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
        log_level = %settings.log_level,
        log_file = %log_file.display(),
        "rstn logging initialized"
    );
}

/// Rotate old log file and compress it
fn rotate_logs(log_dir: &PathBuf) {
    let log_file = log_dir.join("rstn.log");

    if !log_file.exists() {
        return;
    }

    // Check if log file has content
    let metadata = match fs::metadata(&log_file) {
        Ok(m) => m,
        Err(_) => return,
    };

    if metadata.len() == 0 {
        return;
    }

    // Generate timestamp for rotated file
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let rotated_name = format!("rstn.{}.log", timestamp);
    let rotated_path = log_dir.join(&rotated_name);
    let compressed_path = log_dir.join(format!("{}.gz", rotated_name));

    // Rename current log to timestamped version
    if let Err(e) = fs::rename(&log_file, &rotated_path) {
        eprintln!("Warning: Could not rotate log file: {}", e);
        return;
    }

    // Clone log_dir for the thread
    let log_dir_owned = log_dir.clone();

    // Compress the rotated log in background
    std::thread::spawn(move || {
        if let Err(e) = compress_file(&rotated_path, &compressed_path) {
            eprintln!("Warning: Could not compress log file: {}", e);
        } else {
            // Remove uncompressed file after successful compression
            let _ = fs::remove_file(&rotated_path);
        }

        // Clean up old logs (keep last 7 days)
        cleanup_old_logs(&log_dir_owned, 7);
    });
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

        // Check file name starts with "rstn."
        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("rstn."))
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
    rstn_core::paths::rstn_log_file().unwrap_or_else(|_| PathBuf::from("/tmp/rstn.log"))
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
        assert!(path.to_string_lossy().contains(".rustation"));
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
        assert_eq!(settings.log_level, "debug");
        assert!(!settings.log_to_console);
    }
}

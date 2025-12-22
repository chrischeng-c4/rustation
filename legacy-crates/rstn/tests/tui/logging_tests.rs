//! Unit tests for TUI logging infrastructure

use rstn::tui::logging::{FileChangeTracker, LogBuffer, LogCategory, LogEntry};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[test]
fn test_log_buffer_push_and_len() {
    let mut buffer = LogBuffer::new();

    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());

    buffer.push(LogEntry::new(LogCategory::System, "Test 1".to_string()));
    assert_eq!(buffer.len(), 1);
    assert!(!buffer.is_empty());

    buffer.push(LogEntry::new(LogCategory::System, "Test 2".to_string()));
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_log_buffer_entries_iterator() {
    let mut buffer = LogBuffer::new();

    buffer.push(LogEntry::new(LogCategory::Command, "Command 1".to_string()));
    buffer.push(LogEntry::new(
        LogCategory::ClaudeStream,
        "Output 1".to_string(),
    ));
    buffer.push(LogEntry::new(
        LogCategory::FileChange,
        "spec.md".to_string(),
    ));

    let entries: Vec<_> = buffer.entries().collect();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].content, "Command 1");
    assert_eq!(entries[1].content, "Output 1");
    assert_eq!(entries[2].content, "spec.md");
}

#[test]
fn test_log_buffer_circular_eviction() {
    let mut buffer = LogBuffer::new();

    // Push 1050 entries (exceeds 1000 capacity)
    for i in 0..1050 {
        buffer.push(LogEntry::new(LogCategory::System, format!("Entry {}", i)));
    }

    // Buffer should maintain exactly 1000 entries
    assert_eq!(buffer.len(), 1000);

    // First entry should be #50 (first 50 evicted)
    let entries: Vec<_> = buffer.entries().collect();
    assert_eq!(entries[0].content, "Entry 50");

    // Last entry should be #1049
    assert_eq!(entries[999].content, "Entry 1049");
}

#[test]
fn test_log_buffer_categories() {
    let mut buffer = LogBuffer::new();

    buffer.push(LogEntry::new(LogCategory::Command, "cmd".to_string()));
    buffer.push(LogEntry::new(
        LogCategory::ClaudeStream,
        "stream".to_string(),
    ));
    buffer.push(LogEntry::new(LogCategory::FileChange, "file".to_string()));
    buffer.push(LogEntry::new(LogCategory::Hook, "shell".to_string()));
    buffer.push(LogEntry::new(LogCategory::System, "system".to_string()));

    let entries: Vec<_> = buffer.entries().collect();
    assert_eq!(entries[0].category, LogCategory::Command);
    assert_eq!(entries[1].category, LogCategory::ClaudeStream);
    assert_eq!(entries[2].category, LogCategory::FileChange);
    assert_eq!(entries[3].category, LogCategory::Hook);
    assert_eq!(entries[4].category, LogCategory::System);
}

#[test]
fn test_log_entry_category_icons() {
    assert_eq!(LogCategory::User.icon(), "🧑");
    assert_eq!(LogCategory::Command.icon(), "⚡");
    assert_eq!(LogCategory::ClaudeStream.icon(), "🤖");
    assert_eq!(LogCategory::Mcp.icon(), "🔌");
    assert_eq!(LogCategory::Hook.icon(), "🔧");
    assert_eq!(LogCategory::FileChange.icon(), "📝");
    assert_eq!(LogCategory::Error.icon(), "❌");
    assert_eq!(LogCategory::System.icon(), "ℹ️");
}

#[test]
fn test_log_entry_category_colors() {
    use ratatui::style::Color;

    assert_eq!(LogCategory::User.color(), Color::Blue);
    assert_eq!(LogCategory::Command.color(), Color::Cyan);
    assert_eq!(LogCategory::ClaudeStream.color(), Color::White);
    assert_eq!(LogCategory::Mcp.color(), Color::Magenta);
    assert_eq!(LogCategory::Hook.color(), Color::Yellow);
    assert_eq!(LogCategory::FileChange.color(), Color::Green);
    assert_eq!(LogCategory::Error.color(), Color::Red);
    assert_eq!(LogCategory::System.color(), Color::DarkGray);
}

#[test]
fn test_log_entry_timestamp_format() {
    let entry = LogEntry::new(LogCategory::System, "test".to_string());
    let timestamp = entry.format_timestamp();

    // Should be in HH:MM:SS format
    assert!(timestamp.contains(':'));
    assert_eq!(timestamp.len(), 8); // HH:MM:SS
}

#[test]
fn test_file_tracker_first_check() {
    let mut tracker = FileChangeTracker::new();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("test_tracker_{}.txt", std::process::id()));

    // Create test file
    fs::write(&test_file, "initial content").unwrap();

    // First check should return empty (file just registered)
    let changed = tracker.check_files(&[test_file.clone()]);
    assert_eq!(changed.len(), 0);

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_tracker_detects_modification() {
    let mut tracker = FileChangeTracker::new();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("test_tracker_mod_{}.txt", std::process::id()));

    // Create and register file
    fs::write(&test_file, "initial").unwrap();
    tracker.check_files(&[test_file.clone()]);

    // Wait to ensure different mtime
    thread::sleep(Duration::from_millis(10));

    // Modify file
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&test_file)
        .unwrap();
    file.write_all(b"modified content").unwrap();
    drop(file);

    // Should detect change
    let changed = tracker.check_files(&[test_file.clone()]);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0], test_file);

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_tracker_no_change() {
    let mut tracker = FileChangeTracker::new();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("test_tracker_nochange_{}.txt", std::process::id()));

    // Create and register file
    fs::write(&test_file, "content").unwrap();
    tracker.check_files(&[test_file.clone()]);

    // Check again without modification
    let changed = tracker.check_files(&[test_file.clone()]);
    assert_eq!(changed.len(), 0);

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_tracker_multiple_files() {
    let mut tracker = FileChangeTracker::new();
    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();

    let file1 = temp_dir.join(format!("test_multi_1_{}.txt", pid));
    let file2 = temp_dir.join(format!("test_multi_2_{}.txt", pid));
    let file3 = temp_dir.join(format!("test_multi_3_{}.txt", pid));

    // Create files
    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();
    fs::write(&file3, "content3").unwrap();

    // Register all files
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    tracker.check_files(&files);

    // Wait and modify only file2
    thread::sleep(Duration::from_millis(10));
    fs::write(&file2, "modified2").unwrap();

    // Should detect only file2 changed
    let changed = tracker.check_files(&files);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0], file2);

    // Cleanup
    let _ = fs::remove_file(&file1);
    let _ = fs::remove_file(&file2);
    let _ = fs::remove_file(&file3);
}

#[test]
fn test_file_tracker_nonexistent_file() {
    let mut tracker = FileChangeTracker::new();
    let nonexistent = PathBuf::from("/tmp/this_file_does_not_exist_12345.txt");

    // Should handle gracefully (return empty, no panic)
    let changed = tracker.check_files(&[nonexistent]);
    assert_eq!(changed.len(), 0);
}

#[test]
fn test_log_entry_category_icon_method() {
    let entry = LogEntry::new(LogCategory::Command, "test".to_string());
    assert_eq!(entry.category_icon(), "⚡");

    let entry2 = LogEntry::new(LogCategory::FileChange, "test".to_string());
    assert_eq!(entry2.category_icon(), "📝");
}

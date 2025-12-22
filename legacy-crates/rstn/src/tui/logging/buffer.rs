use super::entry::LogEntry;
use std::collections::VecDeque;

const MAX_LOG_ENTRIES: usize = 1000;

/// Circular buffer for log entries with automatic old-entry eviction
pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl LogBuffer {
    /// Create new empty buffer with 1000-entry capacity
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            capacity: MAX_LOG_ENTRIES,
        }
    }

    /// Add entry (evicts oldest if at capacity)
    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Iterate all entries (oldest to newest)
    pub fn entries(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    /// Get current entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Create buffer from existing entries (for state restoration)
    pub fn from_entries(entries: impl IntoIterator<Item = LogEntry>) -> Self {
        let mut buffer = Self::new();
        for entry in entries {
            buffer.push(entry);
        }
        buffer
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

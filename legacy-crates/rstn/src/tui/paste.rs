//! Paste placeholder storage for handling large text and image pastes
//!
//! Implements Claude Code-style placeholder behavior:
//! - Text pastes: `[pasted N lines #ID]`
//! - Image pastes: `[image #ID]`
//!
//! Images are kept in memory until submission, then saved to temp files for Claude access.

use crate::domain::paths::paste_temp_dir;
use crate::{Result, RscliError};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

/// Image data stored in memory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageData {
    /// Decoded image bytes (kept in memory)
    pub bytes: Vec<u8>,
    /// Image format (png, jpg, gif, etc.)
    pub format: String,
}

/// Content of a paste operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PasteContent {
    /// Text content with line count
    Text { content: String, line_count: usize },
    /// Image content (kept in memory until submission)
    Image { data: ImageData },
}

/// Storage for paste placeholders and their content
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasteStorage {
    /// Map from paste ID to content
    items: HashMap<usize, PasteContent>,
    /// Next ID to assign
    next_id: usize,
    /// Temp directory for images (only used during submission)
    temp_dir: PathBuf,
}

impl PasteStorage {
    /// Create new paste storage
    pub fn new() -> Result<Self> {
        let temp_dir = paste_temp_dir()?;
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            items: HashMap::new(),
            next_id: 1,
            temp_dir,
        })
    }

    /// Add text paste, return placeholder string
    ///
    /// Threshold: Only create placeholder if:
    /// - More than 1 line, OR
    /// - More than 100 characters
    ///
    /// Otherwise return None (caller should insert text directly)
    pub fn add_text(&mut self, text: String) -> Option<String> {
        let line_count = text.lines().count();
        let char_count = text.chars().count();

        // Small single-line paste: insert directly
        if line_count == 1 && char_count <= 100 {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.items.insert(
            id,
            PasteContent::Text {
                content: text,
                line_count,
            },
        );

        let placeholder = if line_count == 1 {
            format!("[pasted 1 line #{}]", id)
        } else {
            format!("[pasted {} lines #{}]", line_count, id)
        };

        Some(placeholder)
    }

    /// Add image paste from data URL, return placeholder
    ///
    /// Parses data URL and stores decoded bytes IN MEMORY.
    /// Does NOT write to disk yet - that happens in save_images_to_temp().
    pub fn add_image(&mut self, data_url: &str) -> Result<String> {
        // Parse data URL: data:image/png;base64,iVBORw0KG...
        let (image_bytes, format) = Self::parse_image_data_url(data_url)?;

        let id = self.next_id;
        self.next_id += 1;

        let image_data = ImageData {
            bytes: image_bytes,
            format,
        };

        self.items
            .insert(id, PasteContent::Image { data: image_data });

        Ok(format!("[image #{}]", id))
    }

    /// Save all images to temp files (call RIGHT BEFORE sending to Claude)
    ///
    /// Returns list of temp file paths for cleanup
    pub fn save_images_to_temp(&self) -> Result<Vec<PathBuf>> {
        let mut temp_files = Vec::new();

        for (id, content) in &self.items {
            if let PasteContent::Image { data } = content {
                let filename = format!("paste-{:03}.{}", id, data.format);
                let temp_file_path = self.temp_dir.join(&filename);

                std::fs::write(&temp_file_path, &data.bytes)?;
                temp_files.push(temp_file_path);
            }
        }

        Ok(temp_files)
    }

    /// Replace all placeholders in text with original content
    pub fn replace_placeholders(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Replace text placeholders: [pasted N lines #ID]
        let text_re = Regex::new(r"\[pasted (\d+) lines? #(\d+)\]").unwrap();
        result = text_re
            .replace_all(&result, |caps: &regex::Captures| {
                let id: usize = caps[2].parse().unwrap();

                if let Some(PasteContent::Text { content, .. }) = self.items.get(&id) {
                    content.clone()
                } else {
                    caps[0].to_string() // Keep placeholder if content not found
                }
            })
            .to_string();

        // Replace image placeholders: [image #ID]
        let image_re = Regex::new(r"\[image #(\d+)\]").unwrap();
        result = image_re
            .replace_all(&result, |caps: &regex::Captures| {
                let id: usize = caps[1].parse().unwrap();

                if let Some(PasteContent::Image { data }) = self.items.get(&id) {
                    // Mention filename so Claude can access it via --add-dir
                    format!("(See attached image: paste-{:03}.{})", id, data.format)
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        result
    }

    /// Get directories containing images (for --add-dir)
    pub fn get_image_dirs(&self) -> Vec<PathBuf> {
        // Check if any images exist
        let has_images = self
            .items
            .values()
            .any(|content| matches!(content, PasteContent::Image { .. }));

        if has_images {
            vec![self.temp_dir.clone()]
        } else {
            vec![]
        }
    }

    /// Clear all paste data (call after submission or discard)
    ///
    /// Removes temp files if they exist and clears in-memory content
    pub fn clear(&mut self) {
        // Try to delete temp files (may not exist if images were only in memory)
        if self.temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&self.temp_dir);
            let _ = std::fs::create_dir_all(&self.temp_dir);
        }

        self.items.clear();
        self.next_id = 1;
    }

    /// Check if text looks like an image data URL
    pub fn is_image_data_url(text: &str) -> bool {
        text.starts_with("data:image/")
    }

    /// Parse image data URL
    ///
    /// Format: data:image/<format>;base64,<data>
    /// Returns: (decoded_bytes, format)
    fn parse_image_data_url(data_url: &str) -> Result<(Vec<u8>, String)> {
        use base64::{engine::general_purpose, Engine as _};

        // Check prefix
        if !data_url.starts_with("data:image/") {
            return Err(RscliError::CommandFailed(
                "Invalid image data URL: must start with 'data:image/'".into(),
            ));
        }

        // Extract format and data
        // Format: data:image/png;base64,iVBORw0KG...
        let parts: Vec<&str> = data_url.split(',').collect();
        if parts.len() != 2 {
            return Err(RscliError::CommandFailed(
                "Invalid image data URL: missing base64 data".into(),
            ));
        }

        let header = parts[0];
        let data = parts[1];

        // Extract format from header
        let format_parts: Vec<&str> = header.split('/').collect();
        if format_parts.len() < 2 {
            return Err(RscliError::CommandFailed(
                "Invalid image data URL: cannot extract format".into(),
            ));
        }

        let format_with_encoding = format_parts[1];
        let format = format_with_encoding
            .split(';')
            .next()
            .unwrap_or("png")
            .to_string();

        // Decode base64
        let decoded = general_purpose::STANDARD.decode(data).map_err(|e| {
            RscliError::CommandFailed(format!("Failed to decode base64 image data: {}", e))
        })?;

        Ok((decoded, format))
    }
}

impl Default for PasteStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            items: HashMap::new(),
            next_id: 1,
            temp_dir: PathBuf::from("/tmp/rstn-pastes"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_small_text_returns_none() {
        let mut storage = PasteStorage::default();
        let result = storage.add_text("short".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_add_large_text_creates_placeholder() {
        let mut storage = PasteStorage::default();
        let text = "line1\nline2\nline3".to_string();
        let placeholder = storage.add_text(text).unwrap();
        assert_eq!(placeholder, "[pasted 3 lines #1]");
    }

    #[test]
    fn test_multiple_pastes_get_unique_ids() {
        let mut storage = PasteStorage::default();
        let p1 = storage.add_text("a\nb".to_string()).unwrap();
        let p2 = storage.add_text("c\nd".to_string()).unwrap();
        assert_eq!(p1, "[pasted 2 lines #1]");
        assert_eq!(p2, "[pasted 2 lines #2]");
    }

    #[test]
    fn test_replace_placeholders() {
        let mut storage = PasteStorage::default();
        let text = "Hello\nWorld".to_string();
        let placeholder = storage.add_text(text.clone()).unwrap();

        let input = format!("Please analyze: {}", placeholder);
        let output = storage.replace_placeholders(&input);
        assert_eq!(output, "Please analyze: Hello\nWorld");
    }

    #[test]
    fn test_replace_multiple_placeholders() {
        let mut storage = PasteStorage::default();
        let p1 = storage.add_text("A\nB".to_string()).unwrap();
        let p2 = storage.add_text("C\nD".to_string()).unwrap();

        let input = format!("First: {} Second: {}", p1, p2);
        let output = storage.replace_placeholders(&input);
        assert_eq!(output, "First: A\nB Second: C\nD");
    }

    #[test]
    fn test_is_image_data_url() {
        assert!(PasteStorage::is_image_data_url("data:image/png;base64,abc"));
        assert!(!PasteStorage::is_image_data_url("regular text"));
    }

    #[test]
    fn test_parse_image_data_url() {
        // Tiny 1x1 transparent PNG
        let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let result = PasteStorage::parse_image_data_url(data_url);
        assert!(result.is_ok());
        let (bytes, format) = result.unwrap();
        assert_eq!(format, "png");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_clear_removes_items() {
        let mut storage = PasteStorage::default();
        storage.add_text("test\ntext".to_string());
        storage.clear();
        assert_eq!(storage.items.len(), 0);
        assert_eq!(storage.next_id, 1);
    }
}

//! Artifact writing with atomic operations and rollback support
//!
//! Provides safe file writing using temp file + atomic rename pattern,
//! with rollback capability to clean up partially created artifacts on failure.

use std::path::{Path, PathBuf};

use super::PlanError;

/// Safe artifact writer with rollback support
///
/// Tracks created artifacts and can rollback (delete) them on failure.
/// Uses atomic file writes (temp file + rename) to prevent corruption.
#[derive(Debug)]
pub struct ArtifactWriter {
    /// Target directory for artifacts
    feature_dir: PathBuf,

    /// Track created artifacts for rollback
    created_artifacts: Vec<PathBuf>,
}

impl ArtifactWriter {
    /// Create a new artifact writer for the given feature directory
    pub fn new(feature_dir: PathBuf) -> Self {
        Self {
            feature_dir,
            created_artifacts: Vec::new(),
        }
    }

    /// Write an artifact atomically
    ///
    /// Uses temp file + atomic rename pattern:
    /// 1. Write content to `{name}.tmp`
    /// 2. Rename to `{name}` (atomic on POSIX systems)
    ///
    /// # Arguments
    ///
    /// * `name` - Filename to write (e.g., "plan.md")
    /// * `content` - Content to write
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the created file
    /// * `Err(PlanError)` - On write or rename failure
    pub fn write(&mut self, name: &str, content: &str) -> Result<PathBuf, PlanError> {
        let path = self.feature_dir.join(name);
        let temp_path = self.feature_dir.join(format!("{}.tmp", name));

        // Write to temp file
        std::fs::write(&temp_path, content).map_err(PlanError::ArtifactWrite)?;

        // Atomic rename
        std::fs::rename(&temp_path, &path).map_err(|e| {
            // Clean up temp file on rename failure
            let _ = std::fs::remove_file(&temp_path);
            PlanError::ArtifactWrite(e)
        })?;

        self.created_artifacts.push(path.clone());
        tracing::debug!("Wrote artifact: {:?}", path);

        Ok(path)
    }

    /// Rollback all created artifacts
    ///
    /// Removes all artifacts created by this writer in reverse order.
    /// Continues on individual file removal failures but reports them.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All artifacts successfully removed
    /// * `Err(PlanError)` - At least one artifact could not be removed
    pub fn rollback(&self) -> Result<(), PlanError> {
        let mut errors = Vec::new();

        for path in self.created_artifacts.iter().rev() {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    tracing::warn!("Failed to rollback {:?}: {}", path, e);
                    errors.push(format!("{}: {}", path.display(), e));
                } else {
                    tracing::debug!("Rolled back: {:?}", path);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(PlanError::RollbackFailed(errors.join("; ")))
        }
    }

    /// Get the list of created artifacts
    pub fn created_artifacts(&self) -> &[PathBuf] {
        &self.created_artifacts
    }

    /// Get the feature directory
    pub fn feature_dir(&self) -> &Path {
        &self.feature_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_artifact_writer_new() {
        let temp = TempDir::new().unwrap();
        let writer = ArtifactWriter::new(temp.path().to_path_buf());

        assert_eq!(writer.feature_dir(), temp.path());
        assert!(writer.created_artifacts().is_empty());
    }

    #[test]
    fn test_artifact_writer_write() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        let path = writer.write("test.md", "# Test Content").unwrap();

        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# Test Content");
        assert_eq!(writer.created_artifacts().len(), 1);
        assert_eq!(writer.created_artifacts()[0], path);
    }

    #[test]
    fn test_artifact_writer_write_multiple() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        writer.write("file1.md", "Content 1").unwrap();
        writer.write("file2.md", "Content 2").unwrap();
        writer.write("file3.md", "Content 3").unwrap();

        assert_eq!(writer.created_artifacts().len(), 3);
        assert!(temp.path().join("file1.md").exists());
        assert!(temp.path().join("file2.md").exists());
        assert!(temp.path().join("file3.md").exists());
    }

    #[test]
    fn test_artifact_writer_rollback() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        // Create some artifacts
        let path1 = writer.write("file1.md", "Content 1").unwrap();
        let path2 = writer.write("file2.md", "Content 2").unwrap();

        assert!(path1.exists());
        assert!(path2.exists());

        // Rollback
        writer.rollback().unwrap();

        assert!(!path1.exists());
        assert!(!path2.exists());
    }

    #[test]
    fn test_artifact_writer_rollback_empty() {
        let temp = TempDir::new().unwrap();
        let writer = ArtifactWriter::new(temp.path().to_path_buf());

        // Rollback with no artifacts should succeed
        writer.rollback().unwrap();
    }

    #[test]
    fn test_artifact_writer_rollback_already_deleted() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        let path = writer.write("test.md", "Content").unwrap();

        // Manually delete the file
        std::fs::remove_file(&path).unwrap();

        // Rollback should still succeed (file already gone)
        writer.rollback().unwrap();
    }

    #[test]
    fn test_artifact_writer_atomic_write() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        // Write should be atomic - no temp file left behind
        writer.write("test.md", "Content").unwrap();

        // Check no .tmp file exists
        assert!(!temp.path().join("test.md.tmp").exists());
    }

    #[test]
    fn test_artifact_writer_overwrite() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        // Create initial file
        std::fs::write(temp.path().join("test.md"), "Original").unwrap();

        // Overwrite with writer
        writer.write("test.md", "New Content").unwrap();

        assert_eq!(
            std::fs::read_to_string(temp.path().join("test.md")).unwrap(),
            "New Content"
        );
    }
}

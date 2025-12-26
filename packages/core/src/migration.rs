//! Data Migration System
//!
//! Ensures user data is preserved and correctly updated when the application
//! state structure changes between versions.
//!
//! ## Architecture
//!
//! - Schema versions are integers (1, 2, 3, ...)
//! - Migrations transform JSON from one version to the next
//! - Migrations are applied sequentially until reaching CURRENT_SCHEMA_VERSION
//! - Backups are created before any migration
//!
//! ## Adding a New Migration
//!
//! 1. Increment CURRENT_SCHEMA_VERSION
//! 2. Create a struct implementing Migration trait
//! 3. Register it in MigrationManager::new()

use serde_json::Value;
use std::fs;
use std::path::Path;

/// Current schema version for persisted state.
/// Increment this when making breaking changes to state structure.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Key used to store schema version in JSON
pub const SCHEMA_VERSION_KEY: &str = "schema_version";

// ============================================================================
// Migration Trait
// ============================================================================

/// Trait for implementing state migrations.
///
/// Each migration transforms state from one version to the next.
pub trait Migration: Send + Sync {
    /// The version this migration upgrades FROM
    fn source_version(&self) -> u32;

    /// The version this migration upgrades TO
    fn target_version(&self) -> u32 {
        self.source_version() + 1
    }

    /// Name/description of this migration
    fn name(&self) -> &'static str;

    /// Apply the migration to transform JSON value
    fn migrate(&self, value: Value) -> Result<Value, MigrationError>;
}

// ============================================================================
// Migration Error
// ============================================================================

/// Errors that can occur during migration
#[derive(Debug, Clone)]
pub enum MigrationError {
    /// Schema version is newer than what this app supports
    FutureVersion { found: u32, max_supported: u32 },
    /// Migration failed to transform the data
    TransformFailed { version: u32, reason: String },
    /// Failed to parse JSON
    ParseError(String),
    /// Failed to write backup
    BackupFailed(String),
    /// Failed to write migrated state
    WriteFailed(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::FutureVersion {
                found,
                max_supported,
            } => {
                write!(
                    f,
                    "State file is from a newer version (v{}) than this app supports (v{})",
                    found, max_supported
                )
            }
            MigrationError::TransformFailed { version, reason } => {
                write!(f, "Migration from v{} failed: {}", version, reason)
            }
            MigrationError::ParseError(msg) => write!(f, "Failed to parse state: {}", msg),
            MigrationError::BackupFailed(msg) => write!(f, "Failed to create backup: {}", msg),
            MigrationError::WriteFailed(msg) => write!(f, "Failed to write state: {}", msg),
        }
    }
}

impl std::error::Error for MigrationError {}

// ============================================================================
// Migration Manager
// ============================================================================

/// Manages and applies migrations to persisted state.
pub struct MigrationManager {
    migrations: Vec<Box<dyn Migration>>,
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationManager {
    /// Create a new migration manager with all registered migrations.
    pub fn new() -> Self {
        let migrations: Vec<Box<dyn Migration>> = vec![
            // Register migrations here as they are created
            // Example: Box::new(MigrationV1ToV2),
        ];

        Self { migrations }
    }

    /// Get the schema version from a JSON value.
    /// Returns 1 if no version field exists (legacy data).
    pub fn get_version(value: &Value) -> u32 {
        value
            .get(SCHEMA_VERSION_KEY)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(1)
    }

    /// Set the schema version in a JSON value.
    pub fn set_version(value: &mut Value, version: u32) {
        if let Value::Object(map) = value {
            map.insert(
                SCHEMA_VERSION_KEY.to_string(),
                Value::Number(version.into()),
            );
        }
    }

    /// Check if migration is needed.
    pub fn needs_migration(value: &Value) -> bool {
        Self::get_version(value) < CURRENT_SCHEMA_VERSION
    }

    /// Apply all necessary migrations to bring state up to current version.
    ///
    /// Returns the migrated JSON value with updated schema_version.
    pub fn migrate(&self, mut value: Value) -> Result<Value, MigrationError> {
        let mut current_version = Self::get_version(&value);

        // Check for future version
        if current_version > CURRENT_SCHEMA_VERSION {
            return Err(MigrationError::FutureVersion {
                found: current_version,
                max_supported: CURRENT_SCHEMA_VERSION,
            });
        }

        // Apply migrations in sequence
        while current_version < CURRENT_SCHEMA_VERSION {
            let migration = self
                .migrations
                .iter()
                .find(|m| m.source_version() == current_version);

            match migration {
                Some(m) => {
                    tracing::info!(
                        "Applying migration: {} (v{} -> v{})",
                        m.name(),
                        m.source_version(),
                        m.target_version()
                    );
                    value = m.migrate(value)?;
                    current_version = m.target_version();
                }
                None => {
                    // No migration needed for this version jump
                    // (This happens when we add the version field but don't change structure)
                    current_version += 1;
                }
            }
        }

        // Update the version in the migrated value
        Self::set_version(&mut value, CURRENT_SCHEMA_VERSION);

        Ok(value)
    }

    /// Create a backup of a file before migration.
    pub fn create_backup(path: &Path) -> Result<(), MigrationError> {
        if !path.exists() {
            return Ok(());
        }

        let backup_path = path.with_extension("json.bak");
        fs::copy(path, &backup_path).map_err(|e| {
            MigrationError::BackupFailed(format!(
                "Failed to copy {} to {}: {}",
                path.display(),
                backup_path.display(),
                e
            ))
        })?;

        tracing::info!("Created backup at {}", backup_path.display());
        Ok(())
    }

    /// Load, migrate, and optionally save state from a file.
    ///
    /// This is the main entry point for loading persisted state with migration support.
    pub fn load_and_migrate(
        &self,
        path: &Path,
        save_if_migrated: bool,
    ) -> Result<Option<Value>, MigrationError> {
        // Check if file exists
        if !path.exists() {
            return Ok(None);
        }

        // Read file
        let json_str = fs::read_to_string(path)
            .map_err(|e| MigrationError::ParseError(format!("Failed to read file: {}", e)))?;

        // Parse JSON
        let mut value: Value = serde_json::from_str(&json_str)
            .map_err(|e| MigrationError::ParseError(format!("Invalid JSON: {}", e)))?;

        // Check if migration needed
        if Self::needs_migration(&value) {
            // Create backup before migrating
            Self::create_backup(path)?;

            // Apply migrations
            value = self.migrate(value)?;

            // Save migrated state
            if save_if_migrated {
                let migrated_json = serde_json::to_string_pretty(&value)
                    .map_err(|e| MigrationError::WriteFailed(format!("Serialize failed: {}", e)))?;

                fs::write(path, migrated_json)
                    .map_err(|e| MigrationError::WriteFailed(format!("Write failed: {}", e)))?;

                tracing::info!("Saved migrated state to {}", path.display());
            }
        }

        Ok(Some(value))
    }
}

// ============================================================================
// Example Migration (template for future migrations)
// ============================================================================

/// Example migration template - rename a field
#[allow(dead_code)]
struct ExampleMigrationV1ToV2;

#[allow(dead_code)]
impl Migration for ExampleMigrationV1ToV2 {
    fn source_version(&self) -> u32 {
        1
    }

    fn name(&self) -> &'static str {
        "Rename project_path to default_path"
    }

    fn migrate(&self, mut value: Value) -> Result<Value, MigrationError> {
        // Example: Rename a field in global_settings
        if let Some(settings) = value.get_mut("global_settings") {
            if let Some(obj) = settings.as_object_mut() {
                if let Some(old_value) = obj.remove("project_path") {
                    obj.insert("default_path".to_string(), old_value);
                }
            }
        }
        Ok(value)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_version_with_field() {
        let value = json!({
            "schema_version": 2,
            "data": "test"
        });
        assert_eq!(MigrationManager::get_version(&value), 2);
    }

    #[test]
    fn test_get_version_without_field() {
        let value = json!({
            "data": "test"
        });
        // Legacy data without version defaults to 1
        assert_eq!(MigrationManager::get_version(&value), 1);
    }

    #[test]
    fn test_set_version() {
        let mut value = json!({
            "data": "test"
        });
        MigrationManager::set_version(&mut value, 5);
        assert_eq!(MigrationManager::get_version(&value), 5);
    }

    #[test]
    fn test_needs_migration_true() {
        let value = json!({
            "schema_version": 0,
            "data": "test"
        });
        assert!(MigrationManager::needs_migration(&value));
    }

    #[test]
    fn test_needs_migration_false() {
        let value = json!({
            "schema_version": CURRENT_SCHEMA_VERSION,
            "data": "test"
        });
        assert!(!MigrationManager::needs_migration(&value));
    }

    #[test]
    fn test_migrate_no_change_needed() {
        let manager = MigrationManager::new();
        let value = json!({
            "schema_version": CURRENT_SCHEMA_VERSION,
            "data": "test"
        });

        let result = manager.migrate(value.clone()).unwrap();
        assert_eq!(
            MigrationManager::get_version(&result),
            CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    fn test_migrate_future_version_error() {
        let manager = MigrationManager::new();
        let value = json!({
            "schema_version": CURRENT_SCHEMA_VERSION + 10,
            "data": "test"
        });

        let result = manager.migrate(value);
        assert!(matches!(result, Err(MigrationError::FutureVersion { .. })));
    }

    #[test]
    fn test_migrate_legacy_data() {
        let manager = MigrationManager::new();
        // Legacy data without version field
        let value = json!({
            "version": "0.1.0",
            "recent_projects": [],
            "global_settings": {}
        });

        let result = manager.migrate(value).unwrap();
        assert_eq!(
            MigrationManager::get_version(&result),
            CURRENT_SCHEMA_VERSION
        );
    }

    // Test the example migration
    #[test]
    fn test_example_migration() {
        let migration = ExampleMigrationV1ToV2;
        let value = json!({
            "global_settings": {
                "project_path": "/some/path",
                "theme": "dark"
            }
        });

        let result = migration.migrate(value).unwrap();

        // project_path should be renamed to default_path
        assert!(result["global_settings"]["project_path"].is_null());
        assert_eq!(
            result["global_settings"]["default_path"].as_str(),
            Some("/some/path")
        );
        // Other fields preserved
        assert_eq!(result["global_settings"]["theme"].as_str(), Some("dark"));
    }

    #[test]
    fn test_migration_error_display() {
        let err = MigrationError::FutureVersion {
            found: 5,
            max_supported: 3,
        };
        assert!(err.to_string().contains("newer version"));
    }
}

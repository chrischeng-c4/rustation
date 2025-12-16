//! Spec generation module for rstn
//!
//! This module provides functionality to generate feature specifications
//! from user descriptions, replacing the external bash script with native Rust.
//!
//! # Components
//!
//! - [`number_allocator`]: Allocates unique feature numbers
//! - [`name_generator`]: Generates kebab-case names from descriptions
//! - [`directory_setup`]: Creates feature directory structure
//! - [`spec_generator`]: Integrates with Claude Code CLI
//! - [`catalog_updater`]: Manages the features.json catalog

pub mod catalog_updater;
pub mod directory_setup;
pub mod name_generator;
pub mod number_allocator;
pub mod spec_generator;

use std::path::PathBuf;

pub use catalog_updater::{FeatureEntry, FeaturesCatalog};

/// Information about a new feature being created
#[derive(Debug, Clone)]
pub struct NewFeature {
    /// Zero-padded 3-digit number (e.g., "052")
    pub number: String,

    /// Kebab-case short name (e.g., "internalize-spec-generation")
    pub name: String,

    /// Human-readable title (e.g., "Internalize Spec Generation")
    pub title: String,

    /// Original user description
    pub description: String,
}

impl NewFeature {
    /// Returns the full branch/directory name (e.g., "052-internalize-spec-generation")
    pub fn full_name(&self) -> String {
        format!("{}-{}", self.number, self.name)
    }
}

/// Result of successful spec generation
#[derive(Debug)]
pub struct SpecResult {
    /// The created feature info
    pub feature: NewFeature,

    /// Path to the created spec file
    pub spec_path: PathBuf,

    /// Generated spec content
    pub spec_content: String,

    /// Path to the feature directory
    pub feature_dir: PathBuf,
}

/// Configuration for spec generation (all fields have defaults)
#[derive(Debug, Clone)]
pub struct SpecifyConfig {
    /// Timeout for Claude CLI in seconds (default: 120)
    pub claude_timeout_secs: u64,

    /// Maximum name length (default: 50)
    pub max_name_length: usize,

    /// Whether to create git branch (default: true)
    pub create_branch: bool,

    /// Custom spec template path (default: .specify/templates/spec-template.md)
    pub template_path: Option<PathBuf>,
}

impl Default for SpecifyConfig {
    fn default() -> Self {
        Self {
            claude_timeout_secs: 120,
            max_name_length: 50,
            create_branch: true,
            template_path: None,
        }
    }
}

/// Errors that can occur during spec generation
#[derive(Debug, thiserror::Error)]
pub enum SpecifyError {
    #[error("Failed to allocate feature number: {0}")]
    NumberAllocation(String),

    #[error("Feature number {0} already exists")]
    NumberConflict(String),

    #[error("Failed to generate feature name from description")]
    NameGeneration,

    #[error("Failed to create directory structure: {0}")]
    DirectorySetup(#[source] std::io::Error),

    #[error("Directory already exists: {0}")]
    DirectoryExists(PathBuf),

    #[error("Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code")]
    ClaudeNotFound,

    #[error("Claude Code CLI execution failed: {0}")]
    ClaudeExecution(String),

    #[error("Claude Code CLI timed out after {0} seconds")]
    ClaudeTimeout(u64),

    #[error("Failed to read features catalog: {0}")]
    CatalogRead(#[source] std::io::Error),

    #[error("Failed to parse features catalog: {0}")]
    CatalogParse(#[source] serde_json::Error),

    #[error("Failed to write features catalog: {0}")]
    CatalogWrite(#[source] std::io::Error),

    #[error("Failed to read spec template: {0}")]
    TemplateRead(#[source] std::io::Error),

    #[error("Rollback failed while cleaning up: {0}")]
    RollbackFailed(String),

    #[error("Workspace root not found")]
    WorkspaceNotFound,
}

/// Generate a new feature spec from a description
///
/// This is the main entry point for spec generation. It orchestrates:
/// 1. Feature number allocation
/// 2. Feature name generation
/// 3. Directory structure creation
/// 4. Spec content generation via Claude CLI
/// 5. Catalog update
///
/// On error, it performs rollback to clean up any partial state.
///
/// # Arguments
///
/// * `description` - The user's feature description
/// * `workspace_root` - Path to the workspace root
/// * `config` - Optional configuration (uses defaults if None)
///
/// # Returns
///
/// * `Ok(SpecResult)` - On success, with feature info and paths
/// * `Err(SpecifyError)` - On any failure (directories cleaned up)
///
/// # Example
///
/// ```ignore
/// use rstn_core::specify::generate_spec;
/// use std::path::PathBuf;
///
/// let result = generate_spec(
///     "Add user authentication".to_string(),
///     PathBuf::from("/path/to/project"),
///     None,
/// ).await?;
///
/// println!("Created feature: {}", result.feature.full_name());
/// ```
pub async fn generate_spec(
    description: String,
    workspace_root: PathBuf,
    config: Option<SpecifyConfig>,
) -> Result<SpecResult, SpecifyError> {
    let config = config.unwrap_or_default();

    tracing::info!("Generating spec for: {}", description);

    // 1. Allocate feature number
    let number = number_allocator::allocate_feature_number(&workspace_root)?;
    tracing::debug!("Allocated feature number: {}", number);

    // 2. Generate feature name and title
    let name = name_generator::generate_feature_name(&description, config.max_name_length);
    let title = name_generator::extract_title(&description);

    if name.is_empty() {
        return Err(SpecifyError::NameGeneration);
    }

    tracing::debug!("Generated name: {}, title: {}", name, title);

    // 3. Create feature struct
    let feature = NewFeature {
        number,
        name,
        title,
        description: description.clone(),
    };

    // 4. Create directory structure
    let feature_dir = directory_setup::setup_feature_directory(
        &workspace_root,
        &feature,
        config.template_path.as_deref(),
    )?;
    tracing::debug!("Created directory: {:?}", feature_dir);

    // 5. Generate spec via Claude Code (with rollback on error)
    match spec_generator::generate_spec_content(&feature, &workspace_root, &config).await {
        Ok(spec_content) => {
            // 6. Write spec file
            let spec_path = feature_dir.join("spec.md");
            std::fs::write(&spec_path, &spec_content).map_err(SpecifyError::DirectorySetup)?;

            tracing::debug!("Wrote spec to: {:?}", spec_path);

            // 7. Update features catalog
            catalog_updater::update_features_catalog(&workspace_root, &feature)?;
            tracing::debug!("Updated features catalog");

            tracing::info!("Successfully created feature: {}", feature.full_name());

            Ok(SpecResult {
                feature,
                spec_path,
                spec_content,
                feature_dir,
            })
        }
        Err(e) => {
            // Rollback: Remove created directory
            tracing::warn!("Spec generation failed, rolling back: {}", e);
            if let Err(rollback_err) = directory_setup::rollback_directory(&feature_dir) {
                tracing::error!("Rollback also failed: {}", rollback_err);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to set up a test workspace with minimal structure
    pub fn setup_test_workspace(temp: &TempDir) -> PathBuf {
        let workspace = temp.path().to_path_buf();
        let specs_dir = workspace.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Create empty features.json
        let catalog = r#"{
            "project": "test",
            "description": "Test project",
            "features": []
        }"#;
        std::fs::write(specs_dir.join("features.json"), catalog).unwrap();

        workspace
    }

    #[test]
    fn test_new_feature_full_name() {
        let feature = NewFeature {
            number: "052".to_string(),
            name: "test-feature".to_string(),
            title: "Test Feature".to_string(),
            description: "A test".to_string(),
        };

        assert_eq!(feature.full_name(), "052-test-feature");
    }

    #[test]
    fn test_specify_config_default() {
        let config = SpecifyConfig::default();

        assert_eq!(config.claude_timeout_secs, 120);
        assert_eq!(config.max_name_length, 50);
        assert!(config.create_branch);
        assert!(config.template_path.is_none());
    }
}

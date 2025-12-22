//! Plan generation module for rstn
//!
//! This module provides functionality to generate implementation plans
//! from feature specifications, replacing the external bash script with native Rust.
//!
//! # Components
//!
//! - [`context`]: Context loading for plan generation
//! - [`generator`]: Claude CLI integration for plan content
//! - [`writer`]: Atomic artifact writing with rollback

pub mod context;
pub mod generator;
pub mod writer;

use std::path::PathBuf;

pub use context::PlanContext;
pub use writer::ArtifactWriter;

/// Configuration for plan generation (all fields have defaults)
#[derive(Debug, Clone)]
pub struct PlanConfig {
    /// Timeout for Claude CLI in seconds (default: 120)
    pub claude_timeout_secs: u64,

    /// Whether to generate research.md (default: true)
    pub generate_research: bool,

    /// Whether to generate data-model.md (default: true)
    pub generate_data_model: bool,

    /// Whether to generate quickstart.md (default: true)
    pub generate_quickstart: bool,

    /// Custom plan template path (default: .specify/templates/plan-template.md)
    pub template_path: Option<PathBuf>,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            claude_timeout_secs: 120,
            generate_research: true,
            generate_data_model: true,
            generate_quickstart: true,
            template_path: None,
        }
    }
}

/// Type of plan artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    /// plan.md - main implementation plan
    Plan,
    /// research.md - resolved unknowns and decisions
    Research,
    /// data-model.md - entity definitions
    DataModel,
    /// quickstart.md - getting started guide
    Quickstart,
}

impl ArtifactKind {
    /// Get the filename for this artifact kind
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Plan => "plan.md",
            Self::Research => "research.md",
            Self::DataModel => "data-model.md",
            Self::Quickstart => "quickstart.md",
        }
    }
}

/// Individual output artifact from plan generation
#[derive(Debug, Clone)]
pub struct PlanArtifact {
    /// Artifact type
    pub kind: ArtifactKind,

    /// File path
    pub path: PathBuf,

    /// Whether artifact was generated (vs skipped)
    pub generated: bool,
}

/// Result of successful plan generation
#[derive(Debug)]
pub struct PlanResult {
    /// Path to generated plan.md
    pub plan_path: PathBuf,

    /// List of all generated artifacts
    pub artifacts: Vec<PlanArtifact>,

    /// Feature name
    pub feature_name: String,

    /// Feature directory
    pub feature_dir: PathBuf,
}

/// Errors that can occur during plan generation
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    /// Spec file not found
    #[error("Spec file not found: {0}")]
    SpecNotFound(PathBuf),

    /// Failed to read spec file
    #[error("Failed to read spec: {0}")]
    SpecRead(#[source] std::io::Error),

    /// Failed to read constitution file
    #[error("Failed to read constitution: {0}")]
    ConstitutionRead(#[source] std::io::Error),

    /// Plan template not found
    #[error("Plan template not found: {0}")]
    TemplateNotFound(PathBuf),

    /// Failed to read plan template
    #[error("Failed to read template: {0}")]
    TemplateRead(#[source] std::io::Error),

    /// Claude Code CLI not available
    #[error("Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code")]
    ClaudeNotFound,

    /// Claude CLI execution failed
    #[error("Claude CLI execution failed: {0}")]
    ClaudeExecution(String),

    /// Claude CLI timed out
    #[error("Claude CLI timed out after {0} seconds")]
    ClaudeTimeout(u64),

    /// Failed to write artifact
    #[error("Failed to write artifact: {0}")]
    ArtifactWrite(#[source] std::io::Error),

    /// Rollback failed during cleanup
    #[error("Rollback failed while cleaning up: {0}")]
    RollbackFailed(String),

    /// Feature directory not found
    #[error("Feature directory not found: {0}")]
    FeatureNotFound(PathBuf),
}

/// Generate an implementation plan from a feature spec
///
/// This is the main entry point for plan generation. It orchestrates:
/// 1. Context loading (spec, constitution, template)
/// 2. Plan content generation via Claude CLI
/// 3. Artifact generation (research.md, data-model.md, quickstart.md)
/// 4. Rollback on failure
///
/// # Arguments
///
/// * `feature_dir` - Path to the feature directory (specs/{NNN}-{name}/)
/// * `workspace_root` - Path to the workspace root
/// * `config` - Optional configuration (uses defaults if None)
///
/// # Returns
///
/// * `Ok(PlanResult)` - On success, with paths to all generated artifacts
/// * `Err(PlanError)` - On any failure (partial artifacts cleaned up)
pub async fn generate_plan(
    feature_dir: PathBuf,
    workspace_root: PathBuf,
    config: Option<PlanConfig>,
) -> Result<PlanResult, PlanError> {
    let config = config.unwrap_or_default();

    tracing::info!("Generating plan for: {:?}", feature_dir);

    // Validate feature directory exists
    if !feature_dir.exists() {
        return Err(PlanError::FeatureNotFound(feature_dir));
    }

    // Load context
    let context = PlanContext::load(
        &feature_dir,
        &workspace_root,
        config.template_path.as_deref(),
    )?;
    tracing::debug!("Loaded context for feature: {}", context.feature_name);

    // Create artifact writer for rollback support
    let mut writer = ArtifactWriter::new(feature_dir.clone());
    let mut artifacts = Vec::new();

    // Generate plan content
    match generator::generate_plan_content(&context, &config).await {
        Ok(plan_content) => {
            // Write plan.md
            let plan_path = writer.write(ArtifactKind::Plan.filename(), &plan_content)?;
            artifacts.push(PlanArtifact {
                kind: ArtifactKind::Plan,
                path: plan_path.clone(),
                generated: true,
            });

            // Generate additional artifacts if configured
            if config.generate_research {
                match generator::generate_artifact_content(
                    &context,
                    &config,
                    ArtifactKind::Research,
                )
                .await
                {
                    Ok(content) => {
                        let path = writer.write(ArtifactKind::Research.filename(), &content)?;
                        artifacts.push(PlanArtifact {
                            kind: ArtifactKind::Research,
                            path,
                            generated: true,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate research.md: {}", e);
                        // Continue with other artifacts
                    }
                }
            }

            if config.generate_data_model {
                match generator::generate_artifact_content(
                    &context,
                    &config,
                    ArtifactKind::DataModel,
                )
                .await
                {
                    Ok(content) => {
                        let path = writer.write(ArtifactKind::DataModel.filename(), &content)?;
                        artifacts.push(PlanArtifact {
                            kind: ArtifactKind::DataModel,
                            path,
                            generated: true,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate data-model.md: {}", e);
                    }
                }
            }

            if config.generate_quickstart {
                match generator::generate_artifact_content(
                    &context,
                    &config,
                    ArtifactKind::Quickstart,
                )
                .await
                {
                    Ok(content) => {
                        let path = writer.write(ArtifactKind::Quickstart.filename(), &content)?;
                        artifacts.push(PlanArtifact {
                            kind: ArtifactKind::Quickstart,
                            path,
                            generated: true,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate quickstart.md: {}", e);
                    }
                }
            }

            tracing::info!(
                "Successfully generated plan with {} artifacts",
                artifacts.len()
            );

            Ok(PlanResult {
                plan_path,
                artifacts,
                feature_name: context.feature_name,
                feature_dir,
            })
        }
        Err(e) => {
            // Rollback any created artifacts
            tracing::warn!("Plan generation failed, rolling back: {}", e);
            if let Err(rollback_err) = writer.rollback() {
                tracing::error!("Rollback also failed: {}", rollback_err);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_config_default() {
        let config = PlanConfig::default();

        assert_eq!(config.claude_timeout_secs, 120);
        assert!(config.generate_research);
        assert!(config.generate_data_model);
        assert!(config.generate_quickstart);
        assert!(config.template_path.is_none());
    }

    #[test]
    fn test_artifact_kind_filename() {
        assert_eq!(ArtifactKind::Plan.filename(), "plan.md");
        assert_eq!(ArtifactKind::Research.filename(), "research.md");
        assert_eq!(ArtifactKind::DataModel.filename(), "data-model.md");
        assert_eq!(ArtifactKind::Quickstart.filename(), "quickstart.md");
    }

    #[test]
    fn test_plan_error_display() {
        let err = PlanError::SpecNotFound(PathBuf::from("/path/to/spec.md"));
        assert!(err.to_string().contains("/path/to/spec.md"));

        let err = PlanError::ClaudeNotFound;
        assert!(err.to_string().contains("npm install"));

        let err = PlanError::ClaudeTimeout(60);
        assert!(err.to_string().contains("60"));
    }
}

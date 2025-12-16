//! Context loading for plan generation
//!
//! Loads and aggregates all context needed for plan generation:
//! - Feature spec (spec.md)
//! - Constitution principles (constitution.md)
//! - Plan template (plan-template.md)

use std::path::{Path, PathBuf};

use super::PlanError;

/// Aggregated context for plan generation
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// Content of the feature spec (required)
    pub spec_content: String,

    /// Path to spec.md
    pub spec_path: PathBuf,

    /// Content of constitution.md (optional)
    pub constitution: Option<String>,

    /// Plan template content
    pub plan_template: String,

    /// Feature identifier (e.g., "054-internalize-plan")
    pub feature_name: String,

    /// Path to feature directory (e.g., specs/054-internalize-plan/)
    pub feature_dir: PathBuf,
}

impl PlanContext {
    /// Load plan context from disk
    ///
    /// # Arguments
    ///
    /// * `feature_dir` - Path to the feature directory
    /// * `workspace_root` - Path to the workspace root
    /// * `template_path` - Optional custom template path
    ///
    /// # Returns
    ///
    /// * `Ok(PlanContext)` - On success with all loaded context
    /// * `Err(PlanError)` - If required files are missing or unreadable
    pub fn load(
        feature_dir: &Path,
        workspace_root: &Path,
        template_path: Option<&Path>,
    ) -> Result<Self, PlanError> {
        // Load spec.md (required)
        let spec_path = feature_dir.join("spec.md");
        if !spec_path.exists() {
            return Err(PlanError::SpecNotFound(spec_path));
        }
        let spec_content = std::fs::read_to_string(&spec_path).map_err(PlanError::SpecRead)?;

        // Load constitution.md (optional)
        let constitution_path = workspace_root.join(".specify/memory/constitution.md");
        let constitution = std::fs::read_to_string(&constitution_path).ok();

        // Load plan template (required)
        let template_path = template_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| workspace_root.join(".specify/templates/plan-template.md"));

        if !template_path.exists() {
            return Err(PlanError::TemplateNotFound(template_path));
        }

        let plan_template =
            std::fs::read_to_string(&template_path).map_err(PlanError::TemplateRead)?;

        // Extract feature name from directory
        let feature_name = feature_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            spec_content,
            spec_path,
            constitution,
            plan_template,
            feature_name,
            feature_dir: feature_dir.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_workspace(temp: &TempDir) -> PathBuf {
        let workspace = temp.path().to_path_buf();

        // Create specs directory and feature directory
        let feature_dir = workspace.join("specs/001-test-feature");
        std::fs::create_dir_all(&feature_dir).unwrap();

        // Create spec.md
        std::fs::write(
            feature_dir.join("spec.md"),
            "# Test Feature\n\nThis is a test spec.",
        )
        .unwrap();

        // Create .specify directory structure
        let specify_dir = workspace.join(".specify");
        std::fs::create_dir_all(specify_dir.join("templates")).unwrap();
        std::fs::create_dir_all(specify_dir.join("memory")).unwrap();

        // Create plan template
        std::fs::write(
            specify_dir.join("templates/plan-template.md"),
            "# Plan Template\n\n## Technical Context",
        )
        .unwrap();

        // Create constitution (optional)
        std::fs::write(
            specify_dir.join("memory/constitution.md"),
            "# Constitution\n\n## Principles",
        )
        .unwrap();

        workspace
    }

    #[test]
    fn test_plan_context_load_success() {
        let temp = TempDir::new().unwrap();
        let workspace = setup_test_workspace(&temp);
        let feature_dir = workspace.join("specs/001-test-feature");

        let context = PlanContext::load(&feature_dir, &workspace, None).unwrap();

        assert_eq!(context.feature_name, "001-test-feature");
        assert!(context.spec_content.contains("Test Feature"));
        assert!(context.constitution.is_some());
        assert!(context.plan_template.contains("Plan Template"));
    }

    #[test]
    fn test_plan_context_load_spec_not_found() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_path_buf();
        let feature_dir = workspace.join("specs/nonexistent");

        std::fs::create_dir_all(&feature_dir).unwrap();

        let result = PlanContext::load(&feature_dir, &workspace, None);

        assert!(matches!(result, Err(PlanError::SpecNotFound(_))));
    }

    #[test]
    fn test_plan_context_load_template_not_found() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_path_buf();
        let feature_dir = workspace.join("specs/001-test");

        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(feature_dir.join("spec.md"), "# Spec").unwrap();

        let result = PlanContext::load(&feature_dir, &workspace, None);

        assert!(matches!(result, Err(PlanError::TemplateNotFound(_))));
    }

    #[test]
    fn test_plan_context_load_without_constitution() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_path_buf();
        let feature_dir = workspace.join("specs/001-test");

        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(feature_dir.join("spec.md"), "# Spec").unwrap();

        let specify_dir = workspace.join(".specify/templates");
        std::fs::create_dir_all(&specify_dir).unwrap();
        std::fs::write(specify_dir.join("plan-template.md"), "# Template").unwrap();

        let context = PlanContext::load(&feature_dir, &workspace, None).unwrap();

        assert!(context.constitution.is_none());
    }
}

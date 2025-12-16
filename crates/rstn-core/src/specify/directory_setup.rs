//! Feature directory setup
//!
//! Creates the directory structure for a new feature and copies
//! the spec template.

use std::fs;
use std::path::{Path, PathBuf};

use super::{NewFeature, SpecifyError};

/// Default spec template path relative to workspace root
const DEFAULT_TEMPLATE_PATH: &str = ".specify/templates/spec-template.md";

/// Setup the feature directory structure
///
/// Creates:
/// - `specs/{NNN}-{name}/`
/// - `specs/{NNN}-{name}/spec.md` (from template)
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `feature` - The feature information
/// * `template_path` - Optional custom template path
///
/// # Returns
///
/// Path to the created feature directory
pub fn setup_feature_directory(
    workspace_root: &Path,
    feature: &NewFeature,
    template_path: Option<&Path>,
) -> Result<PathBuf, SpecifyError> {
    let specs_dir = workspace_root.join("specs");
    let feature_dir = specs_dir.join(feature.full_name());

    // Check if directory already exists
    if feature_dir.exists() {
        return Err(SpecifyError::DirectoryExists(feature_dir));
    }

    // Create the feature directory
    fs::create_dir_all(&feature_dir).map_err(SpecifyError::DirectorySetup)?;

    // Copy template to spec.md
    let template = load_template(workspace_root, template_path)?;
    let spec_path = feature_dir.join("spec.md");
    fs::write(&spec_path, template).map_err(SpecifyError::DirectorySetup)?;

    Ok(feature_dir)
}

/// Load the spec template from the configured path
fn load_template(
    workspace_root: &Path,
    custom_path: Option<&Path>,
) -> Result<String, SpecifyError> {
    let template_path = match custom_path {
        Some(path) => path.to_path_buf(),
        None => workspace_root.join(DEFAULT_TEMPLATE_PATH),
    };

    if template_path.exists() {
        fs::read_to_string(&template_path).map_err(SpecifyError::TemplateRead)
    } else {
        // Return a minimal default template if file doesn't exist
        Ok(default_template())
    }
}

/// Minimal default template when no template file exists
fn default_template() -> String {
    r#"# Feature Specification

## Overview

[Describe the feature]

## User Stories

### User Story 1

[Describe the user story]

## Requirements

### Functional Requirements

- **FR-001**: [Requirement description]

## Success Criteria

- [Success criterion 1]
"#
    .to_string()
}

/// Rollback directory creation on error
///
/// Removes the feature directory if it exists.
/// Used when spec generation fails to clean up partial state.
pub fn rollback_directory(feature_dir: &Path) -> Result<(), SpecifyError> {
    if feature_dir.exists() {
        fs::remove_dir_all(feature_dir).map_err(|e| {
            SpecifyError::RollbackFailed(format!(
                "Failed to remove directory {}: {}",
                feature_dir.display(),
                e
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_feature() -> NewFeature {
        NewFeature {
            number: "001".to_string(),
            name: "test-feature".to_string(),
            title: "Test Feature".to_string(),
            description: "A test feature".to_string(),
        }
    }

    #[test]
    fn test_setup_creates_directory() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let feature = create_test_feature();
        let result = setup_feature_directory(temp.path(), &feature, None).unwrap();

        assert!(result.exists());
        assert!(result.is_dir());
        assert_eq!(
            result.file_name().unwrap().to_str().unwrap(),
            "001-test-feature"
        );
    }

    #[test]
    fn test_setup_creates_spec_file() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let feature = create_test_feature();
        let result = setup_feature_directory(temp.path(), &feature, None).unwrap();

        let spec_path = result.join("spec.md");
        assert!(spec_path.exists());
    }

    #[test]
    fn test_setup_uses_custom_template() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create custom template
        let template_path = temp.path().join("custom-template.md");
        fs::write(&template_path, "# Custom Template\n\nCustom content").unwrap();

        let feature = create_test_feature();
        let result = setup_feature_directory(temp.path(), &feature, Some(&template_path)).unwrap();

        let spec_content = fs::read_to_string(result.join("spec.md")).unwrap();
        assert!(spec_content.contains("Custom Template"));
    }

    #[test]
    fn test_setup_fails_if_exists() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(specs_dir.join("001-test-feature")).unwrap();

        let feature = create_test_feature();
        let result = setup_feature_directory(temp.path(), &feature, None);

        assert!(matches!(result, Err(SpecifyError::DirectoryExists(_))));
    }

    #[test]
    fn test_rollback_removes_directory() {
        let temp = TempDir::new().unwrap();
        let feature_dir = temp.path().join("specs").join("001-test");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(feature_dir.join("spec.md"), "content").unwrap();

        assert!(feature_dir.exists());
        rollback_directory(&feature_dir).unwrap();
        assert!(!feature_dir.exists());
    }

    #[test]
    fn test_rollback_noop_if_not_exists() {
        let temp = TempDir::new().unwrap();
        let feature_dir = temp.path().join("nonexistent");

        // Should not error if directory doesn't exist
        let result = rollback_directory(&feature_dir);
        assert!(result.is_ok());
    }
}

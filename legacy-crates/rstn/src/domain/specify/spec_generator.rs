//! Spec generation via Claude Code CLI
//!
//! Integrates with Claude Code CLI to generate feature specifications
//! from user descriptions.

use std::path::Path;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

use super::{NewFeature, SpecifyConfig, SpecifyError};

/// Generate spec content using Claude Code CLI
///
/// Calls Claude Code CLI in headless mode to generate a specification
/// from the feature description.
///
/// # Arguments
///
/// * `feature` - The feature information
/// * `workspace_root` - Path to the workspace root
/// * `config` - Configuration options
///
/// # Returns
///
/// The generated spec content as a string
pub async fn generate_spec_content(
    feature: &NewFeature,
    workspace_root: &Path,
    config: &SpecifyConfig,
) -> Result<String, SpecifyError> {
    // Find Claude CLI using unified discovery
    let claude_path = crate::claude_discovery::ClaudeDiscovery::find_claude()
        .await
        .map_err(|_| SpecifyError::ClaudeNotFound)?;

    // Load template if it exists
    let template = load_spec_template(workspace_root, config.template_path.as_deref())?;

    // Build the prompt
    let prompt = build_prompt(feature, &template);

    // Call Claude CLI with timeout
    let timeout_duration = Duration::from_secs(config.claude_timeout_secs);

    let output = timeout(
        timeout_duration,
        Command::new(&claude_path)
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .arg(&prompt)
            .current_dir(workspace_root)
            .output(),
    )
    .await
    .map_err(|_| SpecifyError::ClaudeTimeout(config.claude_timeout_secs))?
    .map_err(|e| SpecifyError::ClaudeExecution(e.to_string()))?;

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(SpecifyError::ClaudeExecution(stderr))
    }
}

/// Load the spec template from disk
fn load_spec_template(
    workspace_root: &Path,
    custom_path: Option<&Path>,
) -> Result<String, SpecifyError> {
    let template_path = match custom_path {
        Some(path) => path.to_path_buf(),
        None => workspace_root.join(".specify/templates/spec-template.md"),
    };

    if template_path.exists() {
        std::fs::read_to_string(&template_path).map_err(SpecifyError::TemplateRead)
    } else {
        Ok(default_template())
    }
}

/// Build the prompt for Claude CLI
fn build_prompt(feature: &NewFeature, template: &str) -> String {
    format!(
        r#"Generate a feature specification for: {}

Feature description: {}

Use this template structure:
{}

Fill in the template with appropriate content based on the feature description.
Replace all placeholder text with actual specification content.
Include realistic user stories, acceptance criteria, and requirements.
Output ONLY the filled-in markdown specification, no additional commentary."#,
        feature.title, feature.description, template
    )
}

/// Default template when no template file exists
fn default_template() -> String {
    r#"# Feature Specification: [FEATURE NAME]

## Overview

[Brief description of the feature]

## User Stories

### User Story 1 - [Title] (Priority: P1)

[User story description]

**Acceptance Scenarios**:

1. **Given** [context], **When** [action], **Then** [outcome]

## Requirements

### Functional Requirements

- **FR-001**: [Requirement]

## Success Criteria

- [Measurable criterion]
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let feature = NewFeature {
            number: "001".to_string(),
            name: "test-feature".to_string(),
            title: "Test Feature".to_string(),
            description: "A test feature description".to_string(),
        };

        let template = "# Template\n\n## Section";
        let prompt = build_prompt(&feature, template);

        assert!(prompt.contains("Test Feature"));
        assert!(prompt.contains("A test feature description"));
        assert!(prompt.contains("# Template"));
    }

    #[test]
    fn test_default_template_structure() {
        let template = default_template();

        assert!(template.contains("# Feature Specification"));
        assert!(template.contains("## Overview"));
        assert!(template.contains("## User Stories"));
        assert!(template.contains("## Requirements"));
        assert!(template.contains("## Success Criteria"));
    }
}

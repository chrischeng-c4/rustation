//! Plan generation via Claude Code CLI
//!
//! Integrates with Claude Code CLI to generate implementation plans
//! and supporting artifacts from feature specifications.

use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

use super::{ArtifactKind, PlanConfig, PlanContext, PlanError};

/// Check if Claude Code CLI is available
///
/// Verifies that the `claude` command exists and is executable.
pub fn check_claude_cli_available() -> Result<(), PlanError> {
    match which::which("claude") {
        Ok(_) => Ok(()),
        Err(_) => Err(PlanError::ClaudeNotFound),
    }
}

/// Generate plan content using Claude Code CLI
///
/// Calls Claude Code CLI in headless mode to generate a plan
/// from the feature specification.
///
/// # Arguments
///
/// * `context` - The plan context with spec, constitution, and template
/// * `config` - Configuration options
///
/// # Returns
///
/// The generated plan content as a string
pub async fn generate_plan_content(
    context: &PlanContext,
    config: &PlanConfig,
) -> Result<String, PlanError> {
    let prompt = build_plan_prompt(context);
    execute_claude_cli(&prompt, config.claude_timeout_secs).await
}

/// Generate artifact content using Claude Code CLI
///
/// Generates supporting artifacts like research.md, data-model.md, quickstart.md
pub async fn generate_artifact_content(
    context: &PlanContext,
    config: &PlanConfig,
    kind: ArtifactKind,
) -> Result<String, PlanError> {
    let prompt = match kind {
        ArtifactKind::Plan => build_plan_prompt(context),
        ArtifactKind::Research => build_research_prompt(context),
        ArtifactKind::DataModel => build_data_model_prompt(context),
        ArtifactKind::Quickstart => build_quickstart_prompt(context),
    };

    execute_claude_cli(&prompt, config.claude_timeout_secs).await
}

/// Execute Claude CLI with the given prompt
async fn execute_claude_cli(prompt: &str, timeout_secs: u64) -> Result<String, PlanError> {
    let timeout_duration = Duration::from_secs(timeout_secs);

    let output = timeout(
        timeout_duration,
        Command::new("claude")
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .arg(prompt)
            .output(),
    )
    .await
    .map_err(|_| PlanError::ClaudeTimeout(timeout_secs))?
    .map_err(|e| PlanError::ClaudeExecution(e.to_string()))?;

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(PlanError::ClaudeExecution(stderr))
    }
}

/// Build the prompt for plan generation
pub fn build_plan_prompt(context: &PlanContext) -> String {
    let constitution_section = context
        .constitution
        .as_deref()
        .unwrap_or("(No constitution provided)");

    format!(
        r#"Generate an implementation plan for this feature.

## Feature Specification

{}

## Constitution Principles

{}

## Plan Template

{}

Instructions:
1. Fill in the plan template based on the specification
2. Follow the constitution principles when making design decisions
3. Replace all placeholder text with actual content
4. Include specific file paths and technical decisions
5. Output ONLY the filled-in markdown plan, no additional commentary"#,
        context.spec_content, constitution_section, context.plan_template
    )
}

/// Build the prompt for research.md generation
pub fn build_research_prompt(context: &PlanContext) -> String {
    format!(
        r#"Generate a research document for this feature.

## Feature Specification

{}

## Constitution Principles

{}

Instructions:
1. Identify technical unknowns and decisions needed
2. Research each unknown and document findings
3. For each decision, include:
   - Decision: What was chosen
   - Rationale: Why it was chosen
   - Alternatives considered: What else was evaluated
4. Output in markdown format as research.md
5. Output ONLY the markdown content, no additional commentary"#,
        context.spec_content,
        context
            .constitution
            .as_deref()
            .unwrap_or("(No constitution)")
    )
}

/// Build the prompt for data-model.md generation
pub fn build_data_model_prompt(context: &PlanContext) -> String {
    format!(
        r#"Generate a data model document for this feature.

## Feature Specification

{}

Instructions:
1. Extract all entities mentioned in the specification
2. For each entity, document:
   - Fields and their types
   - Validation rules
   - Relationships to other entities
   - State transitions if applicable
3. Include code examples showing struct definitions
4. Output in markdown format as data-model.md
5. Output ONLY the markdown content, no additional commentary"#,
        context.spec_content
    )
}

/// Build the prompt for quickstart.md generation
pub fn build_quickstart_prompt(context: &PlanContext) -> String {
    format!(
        r#"Generate a quickstart guide for implementing this feature.

## Feature Specification

{}

Instructions:
1. Create a step-by-step getting started guide
2. Include:
   - Prerequisites
   - Quick start example code
   - Implementation steps
   - Testing guidance
   - Common issues and solutions
3. Keep examples concise and runnable
4. Output in markdown format as quickstart.md
5. Output ONLY the markdown content, no additional commentary"#,
        context.spec_content
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_context() -> PlanContext {
        PlanContext {
            spec_content: "# Test Feature\n\nThis is a test specification.".to_string(),
            spec_path: PathBuf::from("/test/spec.md"),
            constitution: Some("## Principles\n\n1. Performance first".to_string()),
            plan_template: "# Plan\n\n## Technical Context".to_string(),
            feature_name: "001-test-feature".to_string(),
            feature_dir: PathBuf::from("/test/specs/001-test-feature"),
        }
    }

    #[test]
    fn test_build_plan_prompt() {
        let context = create_test_context();
        let prompt = build_plan_prompt(&context);

        assert!(prompt.contains("Test Feature"));
        assert!(prompt.contains("test specification"));
        assert!(prompt.contains("Performance first"));
        assert!(prompt.contains("Technical Context"));
        assert!(prompt.contains("Fill in the plan template"));
    }

    #[test]
    fn test_build_plan_prompt_without_constitution() {
        let mut context = create_test_context();
        context.constitution = None;

        let prompt = build_plan_prompt(&context);

        assert!(prompt.contains("No constitution provided"));
    }

    #[test]
    fn test_build_research_prompt() {
        let context = create_test_context();
        let prompt = build_research_prompt(&context);

        assert!(prompt.contains("Test Feature"));
        assert!(prompt.contains("research document"));
        assert!(prompt.contains("Decision"));
        assert!(prompt.contains("Rationale"));
    }

    #[test]
    fn test_build_data_model_prompt() {
        let context = create_test_context();
        let prompt = build_data_model_prompt(&context);

        assert!(prompt.contains("Test Feature"));
        assert!(prompt.contains("data model"));
        assert!(prompt.contains("entities"));
        assert!(prompt.contains("Fields"));
    }

    #[test]
    fn test_build_quickstart_prompt() {
        let context = create_test_context();
        let prompt = build_quickstart_prompt(&context);

        assert!(prompt.contains("Test Feature"));
        assert!(prompt.contains("quickstart"));
        assert!(prompt.contains("Prerequisites"));
        assert!(prompt.contains("Implementation steps"));
    }

    #[tokio::test]
    async fn test_claude_cli_check() {
        // This test verifies the function doesn't panic
        // Result depends on whether claude is installed
        let result = check_claude_cli_available();
        match result {
            Ok(()) => {
                // Claude is installed
            }
            Err(PlanError::ClaudeNotFound) => {
                // Claude is not installed, expected in CI
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }
}

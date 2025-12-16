# Quickstart: Internalize Plan Workflow

**Feature**: 054-internalize-plan
**Date**: 2025-12-16

## Overview

This feature moves the `/speckit.plan` workflow from bash scripts to native Rust code. The `plan` module sits alongside the existing `specify` and `clarify` modules in rstn-core.

## Prerequisites

- Rust 1.75+ installed
- Claude Code CLI installed (`npm install -g @anthropic-ai/claude-code`)
- Workspace with existing spec.md in `specs/{NNN}-{name}/`

## Quick Start

### 1. Create Module Structure

```bash
mkdir -p crates/rstn-core/src/plan
```

Create the following files:
- `crates/rstn-core/src/plan/mod.rs`
- `crates/rstn-core/src/plan/context.rs`
- `crates/rstn-core/src/plan/generator.rs`
- `crates/rstn-core/src/plan/writer.rs`

### 2. Add Module to lib.rs

```rust
// crates/rstn-core/src/lib.rs
pub mod plan;
```

### 3. Basic Usage

```rust
use rstn_core::plan::{generate_plan, PlanConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from("/path/to/workspace");
    let feature_dir = workspace_root.join("specs/054-internalize-plan");

    let result = generate_plan(
        feature_dir,
        workspace_root,
        None, // Use default config
    ).await?;

    println!("Generated plan: {:?}", result.plan_path);
    for artifact in &result.artifacts {
        println!("  - {:?}: {}", artifact.kind, artifact.path.display());
    }

    Ok(())
}
```

### 4. Custom Configuration

```rust
use rstn_core::plan::{generate_plan, PlanConfig};

let config = PlanConfig {
    claude_timeout_secs: 180,  // Longer timeout
    generate_research: true,
    generate_data_model: false, // Skip data-model.md
    generate_quickstart: true,
    template_path: None,
};

let result = generate_plan(feature_dir, workspace_root, Some(config)).await?;
```

## Implementation Steps

### Step 1: Define Types (mod.rs)

```rust
// crates/rstn-core/src/plan/mod.rs
pub mod context;
pub mod generator;
pub mod writer;

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("Spec file not found: {0}")]
    SpecNotFound(PathBuf),
    // ... other error variants
}

#[derive(Debug)]
pub struct PlanResult {
    pub plan_path: PathBuf,
    pub artifacts: Vec<PlanArtifact>,
    pub feature_name: String,
    pub feature_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct PlanConfig {
    pub claude_timeout_secs: u64,
    pub generate_research: bool,
    pub generate_data_model: bool,
    pub generate_quickstart: bool,
    pub template_path: Option<PathBuf>,
}

pub async fn generate_plan(
    feature_dir: PathBuf,
    workspace_root: PathBuf,
    config: Option<PlanConfig>,
) -> Result<PlanResult, PlanError> {
    // 1. Load context
    // 2. Generate plan via Claude
    // 3. Generate artifacts
    // 4. Return result (rollback on error)
    todo!()
}
```

### Step 2: Context Loading (context.rs)

```rust
// crates/rstn-core/src/plan/context.rs
use std::path::{Path, PathBuf};
use super::PlanError;

pub struct PlanContext {
    pub spec_content: String,
    pub spec_path: PathBuf,
    pub constitution: Option<String>,
    pub plan_template: String,
    pub feature_name: String,
    pub feature_dir: PathBuf,
}

impl PlanContext {
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
        let spec_content = std::fs::read_to_string(&spec_path)
            .map_err(PlanError::SpecRead)?;

        // Load constitution.md (optional)
        let constitution_path = workspace_root.join(".specify/memory/constitution.md");
        let constitution = std::fs::read_to_string(&constitution_path).ok();

        // Load plan template
        let template_path = template_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| workspace_root.join(".specify/templates/plan-template.md"));
        // ... load template

        Ok(Self {
            spec_content,
            spec_path,
            constitution,
            plan_template: String::new(), // TODO
            feature_name: feature_dir.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            feature_dir: feature_dir.to_path_buf(),
        })
    }
}
```

### Step 3: Claude CLI Integration (generator.rs)

```rust
// crates/rstn-core/src/plan/generator.rs
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use super::{PlanContext, PlanConfig, PlanError};

pub async fn generate_plan_content(
    context: &PlanContext,
    config: &PlanConfig,
) -> Result<String, PlanError> {
    // Check Claude CLI available
    which::which("claude").map_err(|_| PlanError::ClaudeNotFound)?;

    // Build prompt
    let prompt = build_plan_prompt(context);

    // Call Claude CLI
    let timeout_duration = Duration::from_secs(config.claude_timeout_secs);
    let output = timeout(
        timeout_duration,
        Command::new("claude")
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .arg(&prompt)
            .output(),
    )
    .await
    .map_err(|_| PlanError::ClaudeTimeout(config.claude_timeout_secs))?
    .map_err(|e| PlanError::ClaudeExecution(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(PlanError::ClaudeExecution(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

fn build_plan_prompt(context: &PlanContext) -> String {
    format!(
        r#"Generate an implementation plan for this feature:

## Feature Specification
{}

## Constitution Principles
{}

## Plan Template
{}

Fill in the plan template based on the specification. Follow the constitution principles.
Output ONLY the filled-in markdown, no additional commentary."#,
        context.spec_content,
        context.constitution.as_deref().unwrap_or("(No constitution)"),
        context.plan_template
    )
}
```

### Step 4: Artifact Writer (writer.rs)

```rust
// crates/rstn-core/src/plan/writer.rs
use std::path::{Path, PathBuf};
use super::PlanError;

pub struct ArtifactWriter {
    feature_dir: PathBuf,
    created_artifacts: Vec<PathBuf>,
}

impl ArtifactWriter {
    pub fn new(feature_dir: PathBuf) -> Self {
        Self {
            feature_dir,
            created_artifacts: Vec::new(),
        }
    }

    pub fn write(&mut self, name: &str, content: &str) -> Result<PathBuf, PlanError> {
        let path = self.feature_dir.join(name);
        let temp_path = path.with_extension("tmp");

        // Write to temp file
        std::fs::write(&temp_path, content)
            .map_err(PlanError::ArtifactWrite)?;

        // Atomic rename
        std::fs::rename(&temp_path, &path)
            .map_err(PlanError::ArtifactWrite)?;

        self.created_artifacts.push(path.clone());
        Ok(path)
    }

    pub fn rollback(&self) -> Result<(), PlanError> {
        for path in self.created_artifacts.iter().rev() {
            if path.exists() {
                std::fs::remove_file(path).map_err(|e| {
                    PlanError::RollbackFailed(format!("Failed to remove {}: {}", path.display(), e))
                })?;
            }
        }
        Ok(())
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plan_config_default() {
        let config = PlanConfig::default();
        assert_eq!(config.claude_timeout_secs, 120);
        assert!(config.generate_research);
    }

    #[test]
    fn test_artifact_writer_rollback() {
        let temp = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp.path().to_path_buf());

        // Write artifact
        writer.write("test.md", "content").unwrap();
        assert!(temp.path().join("test.md").exists());

        // Rollback
        writer.rollback().unwrap();
        assert!(!temp.path().join("test.md").exists());
    }
}
```

### Integration Tests

```bash
# Run all plan module tests
cargo test -p rstn-core plan

# Run with verbose output
cargo test -p rstn-core plan -- --nocapture
```

## Common Issues

### Claude CLI Not Found

```
Error: Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code
```

**Solution**: Install Claude Code CLI globally via npm.

### Spec Not Found

```
Error: Spec file not found: specs/054-internalize-plan/spec.md
```

**Solution**: Ensure you've run `/speckit.specify` first and the spec exists.

### Timeout

```
Error: Claude CLI timed out after 120 seconds
```

**Solution**: Increase `claude_timeout_secs` in config or check network connectivity.

## Next Steps

After implementation:
1. Run tests: `cargo test -p rstn-core plan`
2. Check clippy: `cargo clippy -p rstn-core`
3. Update CLAUDE.md with new module info
4. Create PR for review

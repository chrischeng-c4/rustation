# Data Model: Internalize Plan Workflow

**Feature**: 054-internalize-plan
**Date**: 2025-12-16

## Entity Overview

This feature introduces a plan generation module with the following key entities:

```
┌─────────────────┐      ┌─────────────────┐
│   PlanContext   │──────│   PlanConfig    │
└─────────────────┘      └─────────────────┘
        │
        ▼
┌─────────────────┐
│ generate_plan() │
└─────────────────┘
        │
        ├──────────────────┐
        ▼                  ▼
┌─────────────────┐  ┌─────────────────┐
│   PlanResult    │  │   PlanError     │
└─────────────────┘  └─────────────────┘
        │
        ▼
┌─────────────────┐
│  PlanArtifact   │
└─────────────────┘
```

## Entities

### PlanContext

Aggregated context for plan generation.

```rust
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
```

**Validation Rules**:
- `spec_content` must not be empty
- `spec_path` must exist
- `feature_dir` must be a valid directory
- `feature_name` must match pattern `\d{3}-[a-z0-9-]+`

**State Transitions**: N/A (immutable after construction)

### PlanConfig

Configuration options for plan generation.

```rust
pub struct PlanConfig {
    /// Timeout for Claude CLI in seconds (default: 120)
    pub claude_timeout_secs: u64,

    /// Whether to generate research.md (default: true)
    pub generate_research: bool,

    /// Whether to generate data-model.md (default: true if spec has entities)
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
```

### PlanResult

Result of successful plan generation.

```rust
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
```

### PlanArtifact

Individual output artifact from plan generation.

```rust
pub struct PlanArtifact {
    /// Artifact type
    pub kind: ArtifactKind,

    /// File path
    pub path: PathBuf,

    /// Whether artifact was generated (vs skipped)
    pub generated: bool,
}

pub enum ArtifactKind {
    Plan,          // plan.md
    Research,      // research.md
    DataModel,     // data-model.md
    Quickstart,    // quickstart.md
}
```

### PlanError

Typed errors for plan generation failures.

```rust
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
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// Feature directory not found
    #[error("Feature directory not found: {0}")]
    FeatureNotFound(PathBuf),
}
```

## Internal Types

### ArtifactWriter

Internal helper for safe artifact writing with rollback support.

```rust
struct ArtifactWriter {
    /// Target directory for artifacts
    feature_dir: PathBuf,

    /// Track created artifacts for rollback
    created_artifacts: Vec<PathBuf>,
}

impl ArtifactWriter {
    fn new(feature_dir: PathBuf) -> Self;

    /// Write artifact atomically (temp file + rename)
    fn write(&mut self, name: &str, content: &str) -> Result<PathBuf, PlanError>;

    /// Rollback all created artifacts
    fn rollback(&self) -> Result<(), PlanError>;
}
```

## Relationships

```
PlanContext ─────────────────┐
     │                       │
     │ loads from            │ uses
     ▼                       ▼
spec.md ◄──── generate_plan() ───► Claude CLI
constitution.md                        │
plan-template.md                       │
                                       ▼
                              PlanResult
                                   │
                                   │ contains
                                   ▼
                             PlanArtifact[]
                                   │
                                   │ written to
                                   ▼
                              plan.md
                              research.md
                              data-model.md
                              quickstart.md
```

## File Structure

```
specs/{NNN}-{name}/
├── spec.md           # Input (required)
├── plan.md           # Output: filled plan template
├── research.md       # Output: resolved unknowns
├── data-model.md     # Output: entity definitions
└── quickstart.md     # Output: getting started guide
```

## Validation Rules Summary

| Entity | Rule | Error |
|--------|------|-------|
| PlanContext | spec_path exists | SpecNotFound |
| PlanContext | spec_content not empty | SpecNotFound |
| PlanContext | feature_dir is directory | FeatureNotFound |
| PlanConfig | claude_timeout_secs > 0 | N/A (default 120) |
| ArtifactWriter | path writable | ArtifactWrite |

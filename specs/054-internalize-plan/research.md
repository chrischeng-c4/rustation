# Research: Internalize Plan Workflow

**Feature**: 054-internalize-plan
**Date**: 2025-12-16

## Research Tasks

### 1. Existing Shell Script Analysis

**Task**: Analyze `setup-plan.sh` and plan workflow to understand behavior to replicate.

**Findings**:
- Location: `.specify/scripts/bash/setup-plan.sh`
- Key functions:
  - Validates feature branch exists
  - Creates feature directory if needed
  - Copies plan template from `.specify/templates/plan-template.md`
  - Returns JSON with paths (FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, BRANCH)
- Plan workflow (from slash command) then:
  - Fills Technical Context section
  - Runs Constitution Check
  - Phase 0: Generates research.md
  - Phase 1: Generates data-model.md, quickstart.md
  - Updates agent context via `update-agent-context.sh`

**Decision**: Replicate core setup and context loading in Rust. Keep agent context update as shell script initially (complex multi-agent logic).

**Rationale**: Focus on plan generation core; agent context can be internalized in a future feature.

### 2. Claude CLI Integration for Plan Generation

**Task**: Determine how to invoke Claude CLI to fill plan template.

**Alternatives Considered**:
1. **Single prompt**: Send spec + constitution + template, get filled plan back
2. **Multi-step prompts**: Separate prompts for each section (Technical Context, Constitution Check, etc.)
3. **Template markers**: Use `{{PLACEHOLDER}}` markers that Claude fills in

**Decision**: Use single prompt (#1) with comprehensive context.

**Rationale**:
- Simpler implementation, fewer API calls
- Claude can see full context and maintain consistency
- Matches existing spec_generator.rs pattern
- Template already has clear section headers for Claude to follow

**Implementation Pattern** (from spec_generator.rs):
```rust
let output = tokio::process::Command::new("claude")
    .arg("--print")
    .arg("--dangerously-skip-permissions")
    .arg(&prompt)
    .current_dir(&workspace_root)
    .output()
    .await?;
```

### 3. Multi-Artifact Generation Strategy

**Task**: Plan how to generate research.md, data-model.md, and quickstart.md.

**Alternatives Considered**:
1. **All-in-one**: Single Claude call generates all artifacts
2. **Sequential**: Separate calls for each artifact (plan → research → data-model → quickstart)
3. **Conditional**: Generate only needed artifacts based on spec content

**Decision**: Use sequential generation (#2) with separate prompts.

**Rationale**:
- Each artifact has different requirements and context
- Smaller prompts = better quality output
- Can skip artifacts that aren't needed (e.g., data-model.md if no entities)
- Easier to test and debug individual artifacts
- Allows partial success (plan.md works even if quickstart fails)

**Artifact Dependencies**:
```
spec.md → plan.md (requires spec)
       → research.md (requires spec, optional constitution)
       → data-model.md (requires spec entities, skip if none)
       → quickstart.md (requires plan, research)
```

### 4. Context Loading Implementation

**Task**: Design context loading for plan generation.

**Decision**: Create `PlanContext` struct that aggregates all needed context.

**Required Context**:
- `spec_content`: Content of spec.md
- `spec_path`: Path to spec.md
- `constitution_content`: Content of constitution.md (optional)
- `plan_template`: Content of plan-template.md
- `feature_name`: e.g., "054-internalize-plan"
- `feature_dir`: Path to specs/{NNN}-{name}/

**Implementation**:
```rust
pub struct PlanContext {
    pub spec_content: String,
    pub spec_path: PathBuf,
    pub constitution: Option<String>,
    pub plan_template: String,
    pub feature_name: String,
    pub feature_dir: PathBuf,
}

impl PlanContext {
    pub fn load(feature_dir: &Path, workspace_root: &Path) -> Result<Self, PlanError> {
        // Load spec.md (required)
        // Load constitution.md (optional)
        // Load plan-template.md (required)
    }
}
```

### 5. Error Handling and Rollback

**Task**: Define error types and rollback strategy.

**Decision**: Create `PlanError` enum similar to `SpecifyError` and `ClarifyError`.

**Error Types**:
```rust
pub enum PlanError {
    SpecNotFound(PathBuf),
    SpecRead(std::io::Error),
    ConstitutionRead(std::io::Error),
    TemplateNotFound(PathBuf),
    TemplateRead(std::io::Error),
    ClaudeNotFound,
    ClaudeExecution(String),
    ClaudeTimeout(u64),
    ArtifactWrite(std::io::Error),
    RollbackFailed(String),
}
```

**Rollback Strategy**:
- Track created artifacts in order
- On failure, delete created artifacts in reverse order
- Use temp file + rename for atomic writes
- Log rollback actions for debugging

### 6. Artifact Writer Design

**Task**: Design safe artifact writing with atomic operations.

**Decision**: Use temp file + atomic rename pattern.

**Implementation**:
```rust
pub struct ArtifactWriter {
    feature_dir: PathBuf,
    created_artifacts: Vec<PathBuf>,
}

impl ArtifactWriter {
    pub fn write_artifact(&mut self, name: &str, content: &str) -> Result<PathBuf, PlanError> {
        let path = self.feature_dir.join(name);
        let temp_path = path.with_extension("tmp");

        // Write to temp file
        std::fs::write(&temp_path, content)?;

        // Atomic rename
        std::fs::rename(&temp_path, &path)?;

        self.created_artifacts.push(path.clone());
        Ok(path)
    }

    pub fn rollback(&self) -> Result<(), PlanError> {
        for path in self.created_artifacts.iter().rev() {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}
```

### 7. Agent Context Update

**Task**: Determine approach for agent context update.

**Decision**: Continue using shell script initially (`update-agent-context.sh`).

**Rationale**:
- Shell script is complex (770+ lines) with multi-agent support
- Internalizing would be a separate feature (055+)
- Can call shell script from Rust for now
- Low priority compared to plan generation core

**Future Consideration**: When internalizing, focus on single-agent (Claude) first.

## Summary of Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Shell Script | Replicate core, keep agent update as shell | Focus on plan generation |
| Claude CLI | Single comprehensive prompt | Simpler, consistent output |
| Multi-Artifact | Sequential generation | Better quality, testable |
| Context Loading | PlanContext struct | Clean aggregation |
| Error Handling | PlanError enum | Consistent with existing modules |
| File Writes | Temp file + atomic rename | Corruption-safe |
| Agent Context | Shell script for now | Complex, separate feature |

## Module Structure

```
crates/rstn-core/src/plan/
├── mod.rs              # PlanError, PlanResult, PlanConfig, generate_plan()
├── context.rs          # PlanContext loading
├── generator.rs        # Claude CLI integration, prompt building
└── writer.rs           # ArtifactWriter with atomic writes and rollback
```

Estimated ~300-400 lines total including tests.

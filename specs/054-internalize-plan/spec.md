# Feature 054: Internalize Plan Workflow

**Feature Branch**: `054-internalize-plan`
**Created**: 2025-12-16
**Status**: Draft

## Overview

Move the `/speckit.plan` workflow logic from Claude Code slash command and bash scripts into native Rust code within rstn. This eliminates the dependency on `.specify/scripts/bash/setup-plan.sh` and provides direct, native integration with Claude Code CLI in headless mode for plan generation.

## Problem Statement

Current plan generation architecture has several limitations:

1. **External dependency**: Relies on bash shell script (`setup-plan.sh`) that must be maintained separately
2. **Limited control**: Cannot easily customize plan generation behavior or handle edge cases
3. **Error handling**: Difficult to provide detailed error messages from shell script
4. **Testing**: Shell scripts harder to unit test than Rust code
5. **Portability**: Bash script may have platform-specific issues
6. **Integration**: Indirect communication between rstn and Claude Code CLI
7. **Multiple outputs**: Plan workflow generates multiple files (research.md, data-model.md, quickstart.md) that need coordinated handling

## Dependencies

**Depends on:**
- Feature 052 (Internalize Spec Generation) - Provides base `specify` module patterns and types
- Feature 053 (Internalize Clarify) - Provides clarify module patterns that can be reused

**Reason:**
- Reuses types like `SpecifyError`, file operations patterns
- Follows established module structure in rstn-core

## User Scenarios & Testing

### User Story 1 - Maintainer Simplification (Priority: P1)

As an rstn maintainer, I want plan generation logic in Rust so that I can maintain a single codebase with consistent tooling and eliminate shell script maintenance burden.

**Why this priority**: Core value proposition - consolidates codebase and reduces maintenance overhead.

**Independent Test**: Can be verified by confirming all plan workflow functionality works without shell script dependency.

**Acceptance Scenarios**:

1. **Given** a feature spec exists at `specs/{NNN}-{name}/spec.md`, **When** the Rust plan module is invoked, **Then** plan.md is created in the same directory using the plan template
2. **Given** the plan workflow runs, **When** it completes successfully, **Then** no bash shell scripts are executed
3. **Given** the constitution.md file exists, **When** planning runs, **Then** constitution principles are checked and incorporated into the plan

---

### User Story 2 - Better Error Handling (Priority: P2)

As an rstn user, I want better error messages when plan generation fails so that I can understand and fix issues quickly.

**Why this priority**: Direct user impact - poor error messages waste time and cause frustration.

**Independent Test**: Can be verified by triggering various error conditions and verifying clear error messages are returned.

**Acceptance Scenarios**:

1. **Given** spec.md doesn't exist, **When** plan generation is attempted, **Then** a clear error message indicates "Spec file not found: {path}"
2. **Given** Claude CLI is not available, **When** plan generation is attempted, **Then** a clear error message indicates "Claude Code CLI not found or not executable"
3. **Given** plan generation partially fails, **When** rollback occurs, **Then** a detailed error message explains what failed and what was cleaned up

---

### User Story 3 - Artifact Generation (Priority: P2)

As an rstn developer, I want the plan workflow to generate all required artifacts (research.md, data-model.md, quickstart.md) so that planning outputs are complete and ready for task generation.

**Why this priority**: Essential for complete planning workflow - incomplete artifacts block subsequent phases.

**Independent Test**: Can be verified by running plan generation and checking all expected output files exist with valid content.

**Acceptance Scenarios**:

1. **Given** a valid spec.md, **When** plan generation completes, **Then** research.md is created with resolved clarifications
2. **Given** a valid spec.md with data entities, **When** plan generation completes, **Then** data-model.md is created with entity definitions
3. **Given** a valid spec.md, **When** plan generation completes, **Then** quickstart.md is created with getting started guidance

---

### User Story 4 - Testable Implementation (Priority: P3)

As a contributor, I want to test plan generation logic so that I can ensure reliability and catch regressions.

**Why this priority**: Quality assurance - enables confident changes and prevents regressions.

**Independent Test**: Can be verified by running unit and integration tests with mock Claude CLI.

**Acceptance Scenarios**:

1. **Given** the plan module, **When** unit tests run, **Then** template loading, context extraction, and file operations are tested
2. **Given** the plan module, **When** integration tests run with mock Claude, **Then** full workflow is tested end-to-end
3. **Given** a failing scenario, **When** rollback logic runs, **Then** partial files are cleaned up correctly

---

### Edge Cases

- What happens when spec.md is empty or malformed?
- How does system handle when constitution.md is missing?
- What happens when plan.md already exists (overwrite vs. error)?
- How does system handle Claude CLI timeout?
- What happens when disk is full during artifact generation?
- How does system handle when spec has unresolved [NEEDS CLARIFICATION] markers?

## Requirements

### Functional Requirements

- **FR-001**: System MUST load and parse the feature spec from `specs/{NNN}-{name}/spec.md`
- **FR-002**: System MUST load constitution principles from `.specify/memory/constitution.md`
- **FR-003**: System MUST copy the plan template from `.specify/templates/plan-template.md` to `specs/{NNN}-{name}/plan.md`
- **FR-004**: System MUST invoke Claude Code CLI in headless mode to fill in the plan template
- **FR-005**: System MUST generate research.md with resolved technical decisions and clarifications
- **FR-006**: System MUST generate data-model.md when the spec contains data entities
- **FR-007**: System MUST generate quickstart.md with getting started guidance
- **FR-008**: System MUST update agent context by invoking `.specify/scripts/bash/update-agent-context.sh` or equivalent Rust logic
- **FR-009**: System MUST validate that Claude Code CLI is available before starting
- **FR-010**: System MUST rollback partial changes on failure (remove incomplete artifacts)
- **FR-011**: System MUST use atomic file writes (temp file + rename) for safety
- **FR-012**: System MUST preserve existing content in plan.md if manually edited sections exist

### Key Entities

- **PlanContext**: Aggregated context for plan generation including spec content, constitution, technical context
- **PlanResult**: Result of plan generation including paths to all generated artifacts
- **PlanArtifact**: Individual output file (plan.md, research.md, data-model.md, quickstart.md)
- **PlanError**: Typed errors for various failure modes (spec not found, Claude unavailable, generation failed, rollback failed)

## Success Criteria

### Measurable Outcomes

- **SC-001**: Plan generation completes in under 60 seconds for a typical spec (Claude API time is the limiting factor)
- **SC-002**: File operations complete in under 100ms
- **SC-003**: 100% of plan generation failures produce actionable error messages
- **SC-004**: 100% rollback success rate on partial failures (no orphaned files)
- **SC-005**: Unit test coverage exceeds 80% for pure functions (template loading, context extraction)
- **SC-006**: Zero platform-specific failures (works on macOS and Linux)

## Assumptions

- Claude Code CLI is installed and accessible in PATH
- The spec.md file follows the standard spec template format
- Constitution.md exists at the expected location
- Plan template exists at `.specify/templates/plan-template.md`
- Feature directory structure follows `specs/{NNN}-{name}/` pattern
- Sufficient disk space for artifact generation (typically <1MB per feature)

# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]  
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]  
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [single/web/mobile - determines source structure]  
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]  
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]  
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

**Strategy**: [Choose one based on feature complexity]

**Option 1: Single PR** (if entire feature ≤ 500 lines)
```
PR #1: Complete feature implementation
  - All user stories in one PR
  - Acceptable only if total ≤ 500 lines
```

**Option 2: PR per User Story** (RECOMMENDED for multi-story features)
```
PR #1: Foundation + Setup
  - Project structure, dependencies, core infrastructure
  - Target: ≤ 500 lines

PR #2: User Story 1 (P1 - Highest Priority)
  - Implement US1 independently
  - Tests, docs, validation
  - Target: ≤ 1,500 lines

PR #3: User Story 2 (P2)
  - Implement US2 independently
  - Tests, docs, validation
  - Target: ≤ 1,500 lines

PR #4: User Story 3 (P3)
  - Implement US3 independently
  - Tests, docs, validation
  - Target: ≤ 1,500 lines

PR #5: Polish & Integration
  - Cross-cutting improvements
  - Documentation updates
  - Target: ≤ 500 lines
```

**Option 3: PR per Component** (if single story >1,500 lines)
```
Break large user story into independently mergeable components:
  - PR for data models
  - PR for services
  - PR for API/UI
  - PR for tests and docs
```

### Selected Strategy

[Document which option above you're using and why]

**Rationale**: [e.g., "3 user stories × ~1,200 lines each = 3 PRs per Option 2"]

### Merge Sequence

1. [PR description] → Merge to main
2. [PR description] → Merge to main
3. [PR description] → Merge to main

**Branch Strategy**: [e.g., "Create `002-feature-name` base branch, then `002-US1`, `002-US2`, etc."]

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

If any PR exceeds limits, split into smaller increments.

---
description: Generate an actionable, dependency-ordered tasks.md for the feature based on available design artifacts.
---

You are a task generation expert for spec-driven development in this Rust monorepo.

---

<chain-of-thought>
Before starting ANY task generation, work through these steps IN ORDER:

<step number="1" name="CONTEXT">
  - Feature directory: ___
  - Available docs: spec.md, plan.md, data-model.md, contracts/
  - User stories in spec: ___
</step>

<step number="2" name="EXTRACTION">
  - Tech stack from plan.md: ___
  - Entities from data-model.md: ___
  - Endpoints from contracts/: ___
</step>

<step number="3" name="ORGANIZATION">
  - Phase 1 (Setup): ___
  - Phase 2 (Foundational): ___
  - Phase 3+ (User Stories by priority): ___
  - Final Phase (Polish): ___
</step>

<step number="4" name="DEPENDENCIES">
  - Sequential tasks: ___
  - Parallel tasks [P]: ___
  - Cross-story dependencies: ___
</step>

<step number="5" name="VALIDATION">
  - All user stories have tasks? YES/NO
  - Each task has file path? YES/NO
  - Format correct (checkbox, ID, labels)? YES/NO
</step>

You MUST write out these 5 steps before generating any tasks.
</chain-of-thought>

---

<decision-trees>

<tree name="Task Organization">
START: Load design documents
│
├─► Phase 1: Setup
│   ├─ Project initialization
│   ├─ Dependencies installation
│   └─ Configuration files
│
├─► Phase 2: Foundational
│   ├─ Shared infrastructure
│   ├─ Core modules used by all stories
│   └─ MUST complete before user stories
│
├─► Phase 3+: User Stories (in priority order P1, P2, P3...)
│   ├─ Map entities to story
│   ├─ Map services to story
│   ├─ Map endpoints to story
│   └─ Each story = independently testable increment
│
└─► Final Phase: Polish
    ├─ Cross-cutting concerns
    ├─ Documentation
    └─ Final integration tests
</tree>

<tree name="Task Format Decision">
START: Creating a task
│
├─► Always start with: - [ ] [TaskID]
│
├─► Is task parallelizable?
│   ├─ Different files, no dependencies → Add [P]
│   └─ Same file or depends on incomplete → No [P]
│
├─► Is task in User Story phase?
│   ├─ YES → Add [US1], [US2], etc.
│   └─ NO (Setup/Foundational/Polish) → No story label
│
├─► Add clear description with file path
│
└─► END: - [ ] T001 [P] [US1] Description in src/path/file.rs
</tree>

<tree name="Dependency Analysis">
START: Task list complete
│
├─► Identify dependencies
│   ├─ T002 needs T001 output → Sequential
│   ├─ T003 and T004 different files → Parallel [P]
│   └─ Same file edits → Sequential
│
├─► Build dependency graph
│   └─ Show story completion order
│
└─► END: Parallel execution examples per story
</tree>

</decision-trees>

---

<few-shot-examples>

<example name="Good Task Format" type="good">
## Phase 3: User Story 1 - User Registration

**Story Goal**: Users can create accounts with email/password

**Independent Test Criteria**:
- User can submit registration form
- Valid user stored in database
- Invalid input shows errors

### Tasks

- [ ] T012 [P] [US1] Create User model in src/models/user.rs
- [ ] T013 [P] [US1] Create UserRepository in src/repositories/user_repository.rs
- [ ] T014 [US1] Implement UserService in src/services/user_service.rs
- [ ] T015 [US1] Create registration endpoint in src/handlers/auth.rs
- [ ] T016 [US1] Add validation middleware in src/middleware/validation.rs

✅ Correct format: checkbox + ID + [P] marker + [US] label + description + path
</example>

<example name="Good Phase Structure" type="good">
## Phase 1: Setup
- [ ] T001 Create project structure per implementation plan
- [ ] T002 [P] Add dependencies to Cargo.toml
- [ ] T003 [P] Create configuration in src/config.rs

## Phase 2: Foundational
- [ ] T004 Create database connection pool in src/db/pool.rs
- [ ] T005 [P] Create error types in src/errors/mod.rs
- [ ] T006 [P] Create logging setup in src/logging.rs

## Phase 3: User Story 1 (P1)
- [ ] T007 [US1] Create User model in src/models/user.rs
...

✅ Clear phases, setup before foundational, foundational before stories
</example>

<example name="Bad - Missing Elements" type="bad">
- [ ] Create User model
- [ ] T001 Implement service
- [ ] [US1] Add endpoint

❌ WRONG: First missing ID
❌ WRONG: Second missing story label and path
❌ WRONG: Third missing ID and path

✅ CORRECT: - [ ] T001 [US1] Create User model in src/models/user.rs
</example>

<example name="Bad - Wrong Organization" type="bad">
## Tasks
- [ ] T001 [US1] Create User model
- [ ] T002 [US2] Create Order model
- [ ] T003 [US1] Create UserService
- [ ] T004 [US2] Create OrderService

❌ WRONG: Mixing user stories
❌ WRONG: Not organized by story phases

✅ CORRECT: Group all US1 tasks together, then US2 tasks
</example>

</few-shot-examples>

---

<grounding>

<file-locations>
rustation/
├── specs/{NNN}-{name}/
│   ├── spec.md           # INPUT: User stories, priorities
│   ├── plan.md           # INPUT: Tech stack, structure
│   ├── data-model.md     # INPUT: Entities (optional)
│   ├── contracts/        # INPUT: Endpoints (optional)
│   ├── research.md       # INPUT: Decisions (optional)
│   ├── quickstart.md     # INPUT: Test scenarios (optional)
│   └── tasks.md          # OUTPUT: Task breakdown
├── .specify/
│   ├── templates/
│   │   └── tasks-template.md  # Task structure template
│   └── scripts/bash/
│       └── check-prerequisites.sh  # Path detection
</file-locations>

<task-format>
## Required Format

```
- [ ] [TaskID] [P?] [Story?] Description with file path
```

**Components**:
1. `- [ ]` - Markdown checkbox (ALWAYS)
2. `[TaskID]` - Sequential: T001, T002, T003...
3. `[P]` - Only if parallelizable (OPTIONAL)
4. `[US1]` - Only in user story phases (OPTIONAL)
5. Description + exact file path (ALWAYS)

**Phase Labels**:
- Setup: NO story label
- Foundational: NO story label
- User Story: MUST have [US1], [US2], etc.
- Polish: NO story label
</task-format>

<commands>
# Get feature paths and available docs
.specify/scripts/bash/check-prerequisites.sh --json

# Output: { "FEATURE_DIR": "...", "AVAILABLE_DOCS": [...] }
</commands>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Missing checkbox → Not trackable → Always start with - [ ]</rule>
<rule severity="NEVER">Missing task ID → Can't reference → Always include T001, T002...</rule>
<rule severity="NEVER">Missing file path → Ambiguous location → Include exact path</rule>
<rule severity="NEVER">Mix user stories in one phase → Breaks independence → One story per phase</rule>
<rule severity="NEVER">Parallel marker on dependent tasks → Execution error → Only [P] for independent tasks</rule>
<rule severity="NEVER">Story label in Setup/Foundational → Wrong organization → No [US] in early phases</rule>

<bad-example name="Common Format Errors">
❌ Create User model (missing everything)
❌ T001 Create model (missing checkbox)
❌ - [ ] Create model (missing ID)
❌ - [ ] T001 Create model (missing path)
❌ - [ ] T001 [US1] Create model (in Setup phase - wrong label)

✅ - [ ] T001 [US1] Create User model in src/models/user.rs
</bad-example>

</negative-constraints>

---

<delimiters>
Use these markers in task output:

<marker name="PHASE HEADER">
## Phase [N]: [Name]

**Story Goal**: [what this phase accomplishes]
**Independent Test Criteria**: [how to verify]
</marker>

<marker name="TASK">
- [ ] T### [P?] [US?] Description in path/to/file.ext
</marker>

<marker name="DEPENDENCIES">
## Dependencies

```
Phase 2 (Foundational) blocks Phase 3+ (User Stories)
US1 can run in parallel with US2 if no shared entities
```
</marker>

<marker name="PARALLEL EXAMPLES">
## Parallel Execution

**Within US1**:
- T012, T013 can run in parallel (different files)
- T014 must wait for T012, T013 (depends on model + repo)
</marker>
</delimiters>

---

<output-structure>
After completing task generation, report in this format:

<report>
  <feature>
    <path>specs/062-user-auth/tasks.md</path>
  </feature>

  <summary>
    <total-tasks>25</total-tasks>
    <phases>5</phases>
    <user-stories>3</user-stories>
  </summary>

  <breakdown>
    <phase name="Setup" tasks="3"/>
    <phase name="Foundational" tasks="5"/>
    <phase name="US1" tasks="8"/>
    <phase name="US2" tasks="6"/>
    <phase name="Polish" tasks="3"/>
  </breakdown>

  <parallel>
    <opportunities>12</opportunities>
    <example>T012, T013 in US1 (different files)</example>
  </parallel>

  <validation>
    <check name="All tasks have checkbox" status="PASS"/>
    <check name="All tasks have ID" status="PASS"/>
    <check name="All tasks have file path" status="PASS"/>
    <check name="User stories organized by phase" status="PASS"/>
  </validation>

  <next-steps>
    <step>Run /speckit.analyze for consistency check</step>
    <step>Run /speckit.implement to execute tasks</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before completing task generation, verify ALL items:

<checklist name="Format Compliance">
  <item>Every task starts with - [ ]?</item>
  <item>Every task has sequential ID (T001, T002...)?</item>
  <item>Every task has exact file path?</item>
  <item>[P] only on truly parallel tasks?</item>
  <item>[US] labels only in user story phases?</item>
</checklist>

<checklist name="Organization">
  <item>Setup phase has no [US] labels?</item>
  <item>Foundational phase has no [US] labels?</item>
  <item>Each user story is its own phase?</item>
  <item>Stories in priority order (P1, P2, P3)?</item>
</checklist>

<checklist name="Coverage">
  <item>All user stories have tasks?</item>
  <item>All entities mapped to stories?</item>
  <item>All endpoints mapped to stories?</item>
  <item>Each story independently testable?</item>
</checklist>

If ANY item is NO, fix it before proceeding.
</self-correction>

---

<quick-reference>
TASKS WORKFLOW:
  1. Run check-prerequisites.sh --json
  2. Load spec.md (user stories), plan.md (tech stack)
  3. Load optional: data-model.md, contracts/, research.md
  4. Organize by phase:
     - Phase 1: Setup (no [US] label)
     - Phase 2: Foundational (no [US] label)
     - Phase 3+: User Stories (P1, P2, P3...)
     - Final: Polish (no [US] label)
  5. Generate tasks with correct format
  6. Add dependencies and parallel examples
  7. Report completion → suggest /speckit.implement

TASK FORMAT:
  - [ ] T001 [P] [US1] Description in src/path/file.rs
  │     │    │   │     │
  │     │    │   │     └── Exact file path
  │     │    │   └── Story label (only in story phases)
  │     │    └── Parallel marker (only if independent)
  │     └── Sequential task ID
  └── Markdown checkbox

PHASE RULES:
  - Setup/Foundational: NO [US] labels
  - User Stories: MUST have [US] labels
  - Polish: NO [US] labels
</quick-reference>

---
description: Execute the implementation planning workflow using the plan template to generate design artifacts.
---

You are an architecture planning expert for spec-driven development in this Rust monorepo.

---

<chain-of-thought>
Before starting ANY planning work, work through these steps IN ORDER:

<step number="1" name="CONTEXT">
  - Feature directory: ___
  - Spec file path: ___
  - Constitution loaded? YES/NO
</step>

<step number="2" name="TECHNICAL CONTEXT">
  - Tech stack from spec: ___
  - Dependencies identified: ___
  - Unknowns (NEEDS CLARIFICATION): ___
</step>

<step number="3" name="CONSTITUTION CHECK">
  - Principles loaded: ___
  - Violations found: ___
  - Gates passed? YES/NO
</step>

<step number="4" name="ARTIFACTS">
  - Phase 0: research.md needed? YES/NO
  - Phase 1: data-model.md content: ___
  - Phase 1: contracts/ needed? YES/NO
</step>

<step number="5" name="VALIDATION">
  - All unknowns resolved? YES/NO
  - Constitution check passed? YES/NO
  - Ready for /speckit.tasks? YES/NO
</step>

You MUST write out these 5 steps before generating any plan artifacts.
</chain-of-thought>

---

<decision-trees>

<tree name="Planning Phases">
START: Load spec and constitution
│
├─► Phase 0: Research (if unknowns exist)
│   ├─ Extract NEEDS CLARIFICATION from tech context
│   ├─ Research each unknown
│   ├─ Consolidate in research.md
│   └─ All unknowns resolved? → Continue
│
├─► Phase 1: Design
│   ├─ Extract entities → data-model.md
│   ├─ Generate contracts → contracts/
│   └─ Create quickstart.md
│
├─► Constitution Check (post-design)
│   ├─ Re-validate against principles
│   └─ Violations? → ERROR (fix before proceeding)
│
└─► END: Report artifacts, suggest /speckit.tasks
</tree>

<tree name="Data Model Extraction">
START: Read spec functional requirements
│
├─► Identify entities
│   ├─ Nouns that persist (User, Order, Product)
│   ├─ State transitions (Draft → Published)
│   └─ Relationships (User has many Orders)
│
├─► For each entity:
│   ├─ Fields (name, type, constraints)
│   ├─ Validation rules from requirements
│   └─ Unique identifiers
│
└─► END: Write data-model.md
</tree>

<tree name="Contract Generation">
START: Read spec user stories
│
├─► For each user action:
│   ├─ Identify endpoint needed
│   ├─ HTTP method (GET/POST/PUT/DELETE)
│   ├─ Request/response schema
│   └─ Error responses
│
├─► Choose format:
│   ├─ REST → OpenAPI spec
│   └─ GraphQL → Schema definition
│
└─► END: Write to contracts/
</tree>

</decision-trees>

---

<few-shot-examples>

<example name="Good Technical Context" type="good">
## Technical Context

**Tech Stack**:
- Language: Rust
- Framework: Axum
- Database: PostgreSQL

**Dependencies**:
- tokio (async runtime)
- sqlx (database driver)
- serde (serialization)

**Unknowns**:
- NEEDS CLARIFICATION: Caching strategy (Redis vs in-memory)

✅ Clear tech stack, specific dependencies, explicit unknowns
</example>

<example name="Good Data Model" type="good">
## Entity: User

**Fields**:
| Field | Type | Constraints |
|-------|------|-------------|
| id | UUID | Primary key |
| email | String | Unique, validated |
| password_hash | String | bcrypt, not null |
| created_at | DateTime | Auto-generated |

**Relationships**:
- User has many Sessions
- User has many Orders

**State Transitions**:
- Pending → Active → Suspended → Deleted
</example>

<example name="Bad - Missing Constitution Check" type="bad">
## Plan

1. Create user module
2. Add database tables
3. Implement API

❌ WRONG: No constitution check
❌ WRONG: No research phase for unknowns
❌ WRONG: No data model extraction

✅ CORRECT: Always run constitution check before and after design
</example>

<example name="Bad - Vague Technical Context" type="bad">
## Technical Context

- Use a database
- Need some kind of caching
- Will use a web framework

❌ WRONG: Vague tech choices
❌ WRONG: "Some kind of" = NEEDS CLARIFICATION

✅ CORRECT: Specific technologies or explicit unknowns
</example>

</few-shot-examples>

---

<grounding>

<file-locations>
rustation/
├── specs/{NNN}-{name}/
│   ├── spec.md           # INPUT: Feature specification
│   ├── plan.md           # OUTPUT: Implementation plan
│   ├── research.md       # OUTPUT: Research findings (Phase 0)
│   ├── data-model.md     # OUTPUT: Entity definitions (Phase 1)
│   ├── quickstart.md     # OUTPUT: Integration scenarios
│   └── contracts/        # OUTPUT: API contracts
│       └── openapi.yaml
├── .specify/
│   ├── memory/
│   │   └── constitution.md  # Project principles
│   ├── templates/
│   │   └── plan-template.md # Plan structure template
│   └── scripts/bash/
│       ├── setup-plan.sh           # Initialize plan
│       └── update-agent-context.sh # Update AI context
</file-locations>

<key-artifacts>
## research.md Structure

- **Decision**: What was chosen
- **Rationale**: Why chosen
- **Alternatives**: What else evaluated

## data-model.md Structure

- **Entity name**: Fields, relationships, state transitions
- **Validation rules**: From functional requirements
- **Identity**: Primary keys, unique constraints

## contracts/ Structure

- **OpenAPI**: REST API specifications
- **GraphQL**: Schema definitions (if applicable)
</key-artifacts>

<commands>
# Initialize plan
.specify/scripts/bash/setup-plan.sh --json

# Output: { "FEATURE_SPEC": "...", "IMPL_PLAN": "...", "SPECS_DIR": "...", "BRANCH": "..." }

# Update agent context after design
.specify/scripts/bash/update-agent-context.sh claude
</commands>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Skip constitution check → Violates project principles → Check before AND after design</rule>
<rule severity="NEVER">Leave NEEDS CLARIFICATION in plan → Blocks implementation → Resolve in Phase 0</rule>
<rule severity="NEVER">Design without spec → No requirements to implement → Load spec first</rule>
<rule severity="NEVER">Ignore gate failures → Constitution violations → ERROR and fix</rule>
<rule severity="NEVER">Generate contracts without user stories → No user actions → Extract from spec</rule>
<rule severity="NEVER">Skip agent context update → AI loses context → Run update script</rule>

<bad-example name="Unresolved Unknown">
## Technical Context
- Caching: NEEDS CLARIFICATION

## Data Model
(proceeding without resolving caching...)

❌ WRONG: Proceeding with unresolved unknown
✅ CORRECT: Research caching in Phase 0, then proceed
</bad-example>

</negative-constraints>

---

<delimiters>
Use these markers in planning output:

<marker name="PHASE 0">
## Phase 0: Research

| Unknown | Decision | Rationale |
|---------|----------|-----------|
</marker>

<marker name="PHASE 1">
## Phase 1: Design

### Data Model
### Contracts
### Quickstart
</marker>

<marker name="CONSTITUTION CHECK">
## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| [name] | PASS/FAIL | [details] |
</marker>

<marker name="GATE ERROR">
❌ GATE FAILURE: [principle violated]
Must fix before proceeding to /speckit.tasks
</marker>
</delimiters>

---

<output-structure>
After completing planning, report in this format:

<report>
  <feature>
    <branch>062-user-auth</branch>
    <spec>specs/062-user-auth/spec.md</spec>
  </feature>

  <artifacts>
    <artifact name="plan.md" status="CREATED"/>
    <artifact name="research.md" status="CREATED"/>
    <artifact name="data-model.md" status="CREATED"/>
    <artifact name="contracts/openapi.yaml" status="CREATED"/>
    <artifact name="quickstart.md" status="CREATED"/>
  </artifacts>

  <constitution>
    <check principle="No unwrap() in production" status="PASS"/>
    <check principle="All public APIs documented" status="PASS"/>
  </constitution>

  <validation>
    <check name="All unknowns resolved" status="PASS"/>
    <check name="Constitution gates passed" status="PASS"/>
    <check name="Agent context updated" status="PASS"/>
  </validation>

  <next-steps>
    <step>Run /speckit.tasks to generate task breakdown</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before completing planning, verify ALL items:

<checklist name="Research Phase">
  <item>All NEEDS CLARIFICATION resolved?</item>
  <item>Decisions documented with rationale?</item>
  <item>Alternatives considered?</item>
</checklist>

<checklist name="Design Phase">
  <item>Data model extracted from requirements?</item>
  <item>Contracts generated from user stories?</item>
  <item>Quickstart scenarios defined?</item>
</checklist>

<checklist name="Constitution">
  <item>Constitution check run before design?</item>
  <item>Constitution check run after design?</item>
  <item>All gates passed?</item>
</checklist>

<checklist name="Process">
  <item>Agent context updated?</item>
  <item>All artifacts saved?</item>
  <item>Paths are absolute?</item>
</checklist>

If ANY item is NO, fix it before proceeding.
</self-correction>

---

<quick-reference>
PLAN WORKFLOW:
  1. Run setup-plan.sh --json
  2. Load spec.md + constitution.md
  3. Extract technical context → identify unknowns
  4. Phase 0: Research unknowns → research.md
  5. Phase 1: Design artifacts
     - data-model.md (entities from requirements)
     - contracts/ (APIs from user stories)
     - quickstart.md (integration scenarios)
  6. Constitution check (post-design)
  7. Update agent context
  8. Report completion → suggest /speckit.tasks

KEY ARTIFACTS:
  - research.md: Decision + Rationale + Alternatives
  - data-model.md: Entities + Fields + Relationships
  - contracts/: OpenAPI or GraphQL schemas
  - quickstart.md: Integration test scenarios

CONSTITUTION GATES:
  - Must pass ALL principles
  - ERROR on any violation
  - Fix violations before /speckit.tasks
</quick-reference>

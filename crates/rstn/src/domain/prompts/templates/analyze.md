---
description: Perform a non-destructive cross-artifact consistency and quality analysis across spec.md, plan.md, and tasks.md after task generation.
---

You are a consistency analysis expert for spec-driven development in this Rust monorepo.

---

<chain-of-thought>
Before starting ANY analysis, work through these steps IN ORDER:

<step number="1" name="CONTEXT">
  - Feature directory: ___
  - Spec path: ___
  - Plan path: ___
  - Tasks path: ___
  - Constitution loaded? YES/NO
</step>

<step number="2" name="EXTRACT">
  - Requirements inventory: ___
  - User story inventory: ___
  - Task coverage mapping: ___
  - Constitution rules: ___
</step>

<step number="3" name="DETECT">
  - Duplications found: ___
  - Ambiguities found: ___
  - Coverage gaps found: ___
  - Constitution violations: ___
</step>

<step number="4" name="SEVERITY">
  - CRITICAL issues: ___
  - HIGH issues: ___
  - MEDIUM issues: ___
  - LOW issues: ___
</step>

<step number="5" name="REPORT">
  - Total findings: ___
  - Coverage %: ___
  - Ready for /speckit.implement? YES/NO
</step>

You MUST write out these 5 steps before generating the analysis report.
</chain-of-thought>

---

<decision-trees>

<tree name="Detection Passes">
START: Artifacts loaded
│
├─► A. Duplication Detection
│   ├─ Find near-duplicate requirements
│   └─ Mark lower-quality for consolidation
│
├─► B. Ambiguity Detection
│   ├─ Vague adjectives (fast, scalable, robust)
│   └─ Unresolved placeholders (TODO, ???)
│
├─► C. Underspecification
│   ├─ Requirements missing outcomes
│   ├─ User stories missing acceptance criteria
│   └─ Tasks referencing undefined components
│
├─► D. Constitution Alignment
│   ├─ MUST principles violated?
│   └─ Missing mandated sections?
│
├─► E. Coverage Gaps
│   ├─ Requirements with zero tasks
│   ├─ Tasks with no mapped requirement
│   └─ Non-functional requirements not in tasks
│
├─► F. Inconsistency
│   ├─ Terminology drift
│   ├─ Conflicting requirements
│   └─ Task ordering contradictions
│
└─► END: Prioritized findings list (max 50)
</tree>

<tree name="Severity Assignment">
START: Finding identified
│
├─► Violates constitution MUST?
│   └─ YES → CRITICAL
│
├─► Duplicate/conflicting requirement?
│   └─ YES → HIGH
│
├─► Ambiguous security/performance?
│   └─ YES → HIGH
│
├─► Terminology drift?
│   └─ YES → MEDIUM
│
├─► Style/wording improvement?
│   └─ YES → LOW
│
└─► END: Assigned severity
</tree>

</decision-trees>

---

<few-shot-examples>

<example name="Good Findings Table" type="good">
## Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| A1 | Duplication | HIGH | spec.md:L120-134 | Two similar login requirements | Merge into single requirement |
| B1 | Ambiguity | MEDIUM | spec.md:L45 | "Fast response" not quantified | Add latency target (e.g., <200ms) |
| D1 | Constitution | CRITICAL | plan.md:L78 | Uses unwrap() in production code | Replace with proper error handling |
| E1 | Coverage | HIGH | spec.md:L89 | "Password reset" has no tasks | Add T025-T027 for password reset flow |

✅ Clear IDs, specific locations, actionable recommendations
</example>

<example name="Good Coverage Summary" type="good">
## Coverage Summary

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| user-can-login | YES | T012-T015 | Complete |
| user-can-register | YES | T016-T020 | Complete |
| user-can-reset-password | NO | - | GAP: Add tasks |
| admin-can-view-users | YES | T021-T023 | Complete |

**Coverage**: 75% (3/4 requirements have tasks)
</example>

<example name="Bad - Vague Finding" type="bad">
| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| A1 | Issue | Medium | Somewhere | Something wrong | Fix it |

❌ WRONG: No specific location
❌ WRONG: Vague summary
❌ WRONG: Non-actionable recommendation

✅ CORRECT: Exact file:line, specific issue, clear action
</example>

<example name="Bad - Missing Constitution Check" type="bad">
## Analysis Report

- Found 3 duplicates
- Found 2 ambiguities
- Coverage: 80%

Recommendation: Proceed to implementation

❌ WRONG: No constitution alignment check
❌ WRONG: No severity assignment
❌ WRONG: No specific locations

✅ CORRECT: Always check constitution, assign severities, cite locations
</example>

</few-shot-examples>

---

<grounding>

<file-locations>
rustation/
├── specs/{NNN}-{name}/
│   ├── spec.md      # INPUT: Requirements, user stories
│   ├── plan.md      # INPUT: Architecture, tech choices
│   └── tasks.md     # INPUT: Task breakdown
├── .specify/
│   └── memory/
│       └── constitution.md  # INPUT: Project principles
</file-locations>

<detection-categories>
## A. Duplication Detection
- Near-duplicate requirements
- Redundant user stories

## B. Ambiguity Detection
- Vague adjectives: fast, scalable, secure, intuitive, robust
- Placeholders: TODO, TKTK, ???, <placeholder>

## C. Underspecification
- Requirements missing measurable outcome
- User stories missing acceptance criteria
- Tasks referencing undefined components

## D. Constitution Alignment
- MUST principle violations (CRITICAL)
- Missing mandated sections

## E. Coverage Gaps
- Requirements with zero tasks
- Tasks with no mapped requirement
- Non-functional requirements not in tasks

## F. Inconsistency
- Terminology drift (same concept, different names)
- Conflicting requirements
- Task ordering contradictions
</detection-categories>

<severity-levels>
| Level | Definition | Examples |
|-------|------------|----------|
| CRITICAL | Violates constitution MUST, blocks functionality | unwrap() in production, missing auth |
| HIGH | Duplicate/conflict, untestable, security gap | Same req twice, vague security |
| MEDIUM | Terminology drift, missing edge cases | "User" vs "Account", no error handling |
| LOW | Style improvements, minor redundancy | Wording, formatting |
</severity-levels>

<commands>
# Get paths and verify tasks exist
.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks

# Output: { "FEATURE_DIR": "...", "AVAILABLE_DOCS": [...] }
</commands>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Modify any files → READ-ONLY analysis → Output report only</rule>
<rule severity="NEVER">Skip constitution check → Critical violations missed → Always check principles</rule>
<rule severity="NEVER">Exceed 50 findings → Overwhelms user → Aggregate remainder</rule>
<rule severity="NEVER">Vague locations → Not actionable → Always cite file:line</rule>
<rule severity="NEVER">Proceed with CRITICAL issues → Blocks implementation → Resolve first</rule>
<rule severity="NEVER">Hallucinate missing sections → False positives → Report accurately</rule>

<bad-example name="File Modification Attempt">
❌ "Let me fix this issue in spec.md..."
❌ "Updating tasks.md to add coverage..."

✅ CORRECT: "Issue found. Recommendation: Update spec.md L45 to..."
</bad-example>

</negative-constraints>

---

<delimiters>
Use these markers in analysis output:

<marker name="FINDINGS TABLE">
## Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
</marker>

<marker name="COVERAGE TABLE">
## Coverage Summary

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
</marker>

<marker name="CONSTITUTION ISSUES">
## Constitution Alignment Issues

| Principle | Violation | Location | Recommendation |
|-----------|-----------|----------|----------------|
</marker>

<marker name="METRICS">
## Metrics

- Total Requirements: X
- Total Tasks: Y
- Coverage %: Z
- Ambiguity Count: N
- Critical Issues: N
</marker>
</delimiters>

---

<output-structure>
After completing analysis, report in this format:

<report>
  <feature>
    <path>specs/062-user-auth/</path>
  </feature>

  <findings>
    <category name="Duplication" count="2"/>
    <category name="Ambiguity" count="3"/>
    <category name="Coverage" count="1"/>
    <category name="Constitution" count="0"/>
  </findings>

  <severity>
    <level name="CRITICAL" count="0"/>
    <level name="HIGH" count="2"/>
    <level name="MEDIUM" count="3"/>
    <level name="LOW" count="1"/>
  </severity>

  <coverage>
    <total-requirements>10</total-requirements>
    <covered-requirements>9</covered-requirements>
    <coverage-percent>90</coverage-percent>
  </coverage>

  <recommendation>
    PROCEED / RESOLVE ISSUES FIRST
  </recommendation>

  <next-steps>
    <step>Fix HIGH issues before /speckit.implement</step>
    <step>MEDIUM/LOW can be addressed during implementation</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before completing analysis, verify ALL items:

<checklist name="Detection Completeness">
  <item>Duplication detection run?</item>
  <item>Ambiguity detection run?</item>
  <item>Coverage gaps identified?</item>
  <item>Constitution alignment checked?</item>
  <item>Terminology consistency checked?</item>
</checklist>

<checklist name="Report Quality">
  <item>All findings have specific locations?</item>
  <item>All findings have severity assigned?</item>
  <item>All findings have recommendations?</item>
  <item>Findings limited to 50?</item>
</checklist>

<checklist name="Process">
  <item>No files modified?</item>
  <item>Coverage % calculated?</item>
  <item>Next actions provided?</item>
</checklist>

If ANY item is NO, fix it before completing.
</self-correction>

---

<quick-reference>
ANALYZE WORKFLOW:
  1. Run check-prerequisites.sh --json --require-tasks
  2. Load spec.md, plan.md, tasks.md, constitution.md
  3. Build semantic models:
     - Requirements inventory
     - Task coverage mapping
     - Constitution rules
  4. Run detection passes (A-F)
  5. Assign severity (CRITICAL > HIGH > MEDIUM > LOW)
  6. Generate findings table (max 50)
  7. Calculate coverage %
  8. Provide recommendation

SEVERITY RULES:
  - CRITICAL: Constitution MUST violations, blocking issues
  - HIGH: Duplicates, conflicts, untestable criteria
  - MEDIUM: Terminology drift, missing edge cases
  - LOW: Style, minor improvements

KEY CONSTRAINTS:
  - READ-ONLY (no file modifications)
  - Max 50 findings (aggregate rest)
  - Always check constitution
  - Specific locations required
</quick-reference>

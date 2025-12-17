---
description: Create or update the feature specification from a natural language feature description.
---

You are a specification expert for spec-driven development in this Rust monorepo.

---

<chain-of-thought>
Before starting ANY specification work, work through these steps IN ORDER:

<step number="1" name="INPUT">
  - Feature description: ___
  - Keywords extracted: ___
  - Short name (2-4 words): ___
</step>

<step number="2" name="CONTEXT">
  - Existing branches with similar name? YES/NO
  - Next feature number: ___
  - Spec directory: specs/{NNN}-{name}/
</step>

<step number="3" name="REQUIREMENTS">
  - User goals identified: ___
  - Actors: ___
  - Success criteria: ___
</step>

<step number="4" name="CLARITY CHECK">
  - Needs clarification (max 3)? YES/NO
  - If YES, list questions: ___
  - Assumptions made: ___
</step>

<step number="5" name="OUTPUT">
  - spec.md complete? YES/NO
  - Checklist generated? YES/NO
  - Ready for /speckit.clarify or /speckit.plan? YES/NO
</step>

You MUST write out these 5 steps before generating any specification.
</chain-of-thought>

---

<decision-trees>

<tree name="Feature Naming">
START: Parse feature description
│
├─► Extract meaningful keywords
│   ├─ Action verb (add, fix, implement, create)
│   └─ Noun/object (auth, dashboard, API, button)
│
├─► Generate short name (2-4 words)
│   ├─ Format: action-noun (e.g., "add-user-auth")
│   ├─ Preserve technical terms (OAuth2, API, JWT)
│   └─ Keep concise but descriptive
│
└─► END: Return hyphenated short name
</tree>

<tree name="Feature Number Allocation">
START: Need new feature number
│
├─► Fetch remote branches
│   └─ git fetch --all --prune
│
├─► Search all sources for short-name
│   ├─ Remote: git ls-remote --heads origin
│   ├─ Local: git branch
│   └─ Specs: ls specs/
│
├─► Find highest N matching pattern {NNN}-{short-name}
│   ├─ Found? → Use N+1
│   └─ Not found? → Start at 001
│
└─► END: Return next available number
</tree>

<tree name="Clarification Handling">
START: Writing specification
│
├─► Is requirement clear?
│   ├─ YES → Document with reasonable defaults
│   └─ NO → Continue
│
├─► Can we make informed guess?
│   ├─ YES → Use default, document in Assumptions
│   └─ NO → Continue
│
├─► Does it significantly impact scope/UX/security?
│   ├─ NO → Use industry standard default
│   └─ YES → Add [NEEDS CLARIFICATION] (max 3 total)
│
└─► END: Proceed with specification
</tree>

</decision-trees>

---

<few-shot-examples>

<example name="Good Feature Description" type="good">
User: "Add user authentication with email/password login"

STEP 1: INPUT
- Feature description: "Add user authentication with email/password login"
- Keywords: authentication, email, password, login
- Short name: user-auth

STEP 2: CONTEXT
- Existing branches: NO
- Next feature number: 062
- Spec directory: specs/062-user-auth/

STEP 3: REQUIREMENTS
- User goals: Secure login to application
- Actors: End users, administrators
- Success criteria: Users can register, login, logout securely

STEP 4: CLARITY CHECK
- Needs clarification: NO (standard auth pattern)
- Assumptions: Session-based auth, bcrypt hashing, standard email validation

STEP 5: OUTPUT
- spec.md complete: YES
- Ready for next phase: YES
</example>

<example name="Vague Description Handled Well" type="good">
User: "Make the app faster"

STEP 1: INPUT
- Feature description: "Make the app faster"
- Keywords: performance, speed, optimization
- Short name: performance-optimization

STEP 4: CLARITY CHECK
- Needs clarification: YES (1 question)
- Q1: Which user flows are slowest? (startup, navigation, data loading)
- Assumptions: Focus on perceived performance, not just metrics
</example>

<example name="Missing Short Name" type="bad">
User: "Add feature"

❌ WRONG: Proceeding without extracting meaningful short name
❌ WRONG: Creating spec without clear user goals
❌ WRONG: Not identifying actors or success criteria

✅ CORRECT: Ask user to provide more detail about the feature
</example>

<example name="Too Many Clarifications" type="bad">
User: "Add payment processing"

❌ WRONG: Asking 10 clarification questions upfront
❌ WRONG: [NEEDS CLARIFICATION] for every minor detail

✅ CORRECT: Maximum 3 clarifications for critical decisions
✅ CORRECT: Use industry standards for payment (PCI compliance, etc.)
</example>

</few-shot-examples>

---

<grounding>

<file-locations>
rustation/
├── specs/{NNN}-{name}/
│   ├── spec.md                    # Feature specification (OUTPUT)
│   └── checklists/
│       └── requirements.md        # Spec quality checklist (OUTPUT)
├── .specify/
│   ├── templates/
│   │   └── spec-template.md       # Specification template
│   └── scripts/bash/
│       └── create-new-feature.sh  # Branch/spec creation script
└── features.json                  # Feature catalog
</file-locations>

<key-structures>
## Spec Template Sections (Mandatory)

1. **Overview** - Feature summary and context
2. **User Stories** - Who does what and why
3. **Functional Requirements** - What the system must do
4. **Success Criteria** - Measurable outcomes
5. **Assumptions** - Documented defaults

## Spec Template Sections (Optional)

6. **Data Model** - Entities if data involved
7. **Edge Cases** - Boundary conditions
8. **Non-Functional Requirements** - Performance, security
9. **Out of Scope** - Explicit exclusions
</key-structures>

<commands>
# Create new feature branch and spec
.specify/scripts/bash/create-new-feature.sh --json "description"

# With explicit number and name
.specify/scripts/bash/create-new-feature.sh --json --number 062 --short-name "user-auth" "Add user authentication"

# Check existing branches
git ls-remote --heads origin | grep -E 'refs/heads/[0-9]+-'
git branch | grep -E '^[* ]*[0-9]+-'
</commands>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Include implementation details → Spec is WHAT not HOW → No frameworks, APIs, code</rule>
<rule severity="NEVER">More than 3 [NEEDS CLARIFICATION] → Causes analysis paralysis → Make informed defaults</rule>
<rule severity="NEVER">Skip feature number check → Causes branch conflicts → Always check all sources</rule>
<rule severity="NEVER">Run create-new-feature.sh twice → Duplicate branches → Run once per feature</rule>
<rule severity="NEVER">Vague success criteria → Not testable → Use measurable outcomes</rule>
<rule severity="NEVER">Tech-specific success criteria → Implementation leak → Keep technology-agnostic</rule>

<bad-example name="Implementation in Spec">
❌ "API response time under 200ms"
❌ "Use Redis for caching"
❌ "React components render efficiently"

✅ "Users see results instantly"
✅ "System supports 10,000 concurrent users"
✅ "Task completion rate improves by 40%"
</bad-example>

</negative-constraints>

---

<delimiters>
Use these markers in specification output:

<marker name="FEATURE INFO">
Feature: 062-user-auth
Branch: 062-user-auth
Spec: specs/062-user-auth/spec.md
</marker>

<marker name="CLARIFICATION NEEDED">
[NEEDS CLARIFICATION: specific question here]
</marker>

<marker name="ASSUMPTION">
Assumption: [what was assumed and why]
</marker>

<marker name="READY FOR NEXT PHASE">
Spec complete. Run /speckit.clarify or /speckit.plan
</marker>
</delimiters>

---

<output-structure>
After completing specification, report in this format:

<report>
  <feature>
    <number>062</number>
    <name>user-auth</name>
    <branch>062-user-auth</branch>
  </feature>

  <artifacts>
    <spec path="specs/062-user-auth/spec.md" status="CREATED"/>
    <checklist path="specs/062-user-auth/checklists/requirements.md" status="CREATED"/>
  </artifacts>

  <clarity>
    <clarifications-needed>0</clarifications-needed>
    <assumptions-made>3</assumptions-made>
  </clarity>

  <validation>
    <check name="No implementation details" status="PASS"/>
    <check name="Measurable success criteria" status="PASS"/>
    <check name="All sections complete" status="PASS"/>
  </validation>

  <next-steps>
    <step>Run /speckit.clarify if questions remain</step>
    <step>Run /speckit.plan to generate architecture</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before completing specification, verify ALL items:

<checklist name="Content Quality">
  <item>No implementation details (languages, frameworks, APIs)?</item>
  <item>Focused on user value and business needs?</item>
  <item>Written for non-technical stakeholders?</item>
  <item>All mandatory sections completed?</item>
</checklist>

<checklist name="Requirement Completeness">
  <item>Maximum 3 [NEEDS CLARIFICATION] markers?</item>
  <item>Requirements are testable and unambiguous?</item>
  <item>Success criteria are measurable?</item>
  <item>Success criteria are technology-agnostic?</item>
</checklist>

<checklist name="Process">
  <item>Feature number checked against all sources?</item>
  <item>Short name is 2-4 words, hyphenated?</item>
  <item>create-new-feature.sh run exactly once?</item>
  <item>Assumptions documented?</item>
</checklist>

If ANY item is NO, fix it before proceeding.
</self-correction>

---

<quick-reference>
SPECIFY WORKFLOW:
  1. Parse description → extract keywords → generate short name
  2. Check branches/specs → allocate next number
  3. Run: .specify/scripts/bash/create-new-feature.sh --json "description"
  4. Load spec-template.md → fill sections
  5. Max 3 [NEEDS CLARIFICATION] → use defaults for rest
  6. Generate requirements checklist
  7. Report completion → suggest next phase

SUCCESS CRITERIA RULES:
  - Measurable (time, percentage, count)
  - Technology-agnostic (no frameworks, APIs)
  - User-focused (outcomes, not internals)
  - Verifiable (can be tested)

CLARIFICATION PRIORITY:
  scope > security/privacy > user experience > technical details
</quick-reference>

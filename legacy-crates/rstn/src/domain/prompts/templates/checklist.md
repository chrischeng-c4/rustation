---
description: Generate a custom checklist for the current feature based on user requirements.
---

You are a requirements quality expert for spec-driven development in this Rust monorepo.

---

<core-concept>
## Checklists are "Unit Tests for Requirements"

**CRITICAL**: Checklists validate the QUALITY of requirements, NOT the implementation.

**Test the English, not the code**:
- ❌ "Verify button clicks correctly" (tests implementation)
- ✅ "Are click handler requirements defined for all buttons?" (tests requirements)

If your spec is code written in English, the checklist is its unit test suite.
</core-concept>

---

<chain-of-thought>
Before generating ANY checklist, work through these steps IN ORDER:

<step number="1" name="CONTEXT">
  - Feature directory: ___
  - Spec file: ___
  - Plan file (if exists): ___
  - User input focus: ___
</step>

<step number="2" name="CLARIFY">
  - Checklist theme: ___
  - Focus areas (max 4): ___
  - Audience: Author / Reviewer / QA
  - Depth: Lightweight / Standard / Formal
</step>

<step number="3" name="EXTRACT">
  - Signals from spec: ___
  - Keywords identified: ___
  - Risk areas: ___
</step>

<step number="4" name="GENERATE">
  - Quality dimensions to check: ___
  - Completeness items: ___
  - Clarity items: ___
  - Consistency items: ___
</step>

<step number="5" name="VALIDATE">
  - All items test requirements (not implementation)? YES/NO
  - Traceability references included (≥80%)? YES/NO
  - Ready to save? YES/NO
</step>

You MUST write out these 5 steps before generating the checklist.
</chain-of-thought>

---

<decision-trees>

<tree name="Checklist Focus">
START: Analyze user input
│
├─► Extract signals from $ARGUMENTS
│   ├─ Domain keywords (auth, UX, API, perf)
│   ├─ Risk indicators (critical, must, compliance)
│   └─ Stakeholder hints (QA, review, security)
│
├─► Cluster into focus areas (max 4)
│   └─ Rank by relevance to user input
│
├─► Determine checklist type:
│   ├─ ux.md → UI/interaction requirements
│   ├─ api.md → API contract requirements
│   ├─ security.md → Security requirements
│   ├─ performance.md → Performance requirements
│   └─ requirements.md → General requirements
│
└─► END: Focus and filename determined
</tree>

<tree name="Item Generation">
START: For each focus area
│
├─► Check Completeness
│   └─ "Are [requirement type] defined for [scenario]?"
│
├─► Check Clarity
│   └─ "Is [vague term] quantified with specific criteria?"
│
├─► Check Consistency
│   └─ "Are requirements consistent between [A] and [B]?"
│
├─► Check Measurability
│   └─ "Can [requirement] be objectively measured?"
│
├─► Check Coverage
│   └─ "Are [edge cases/scenarios] addressed?"
│
└─► END: Add [Completeness/Clarity/Consistency] tags
</tree>

<tree name="Traceability">
START: Each checklist item
│
├─► Does it reference spec section?
│   ├─ YES → Add [Spec §X.Y]
│   └─ NO → Continue
│
├─► Is it checking for a gap?
│   └─ YES → Add [Gap]
│
├─► Is it checking for ambiguity?
│   └─ YES → Add [Ambiguity]
│
├─► Is it checking for conflict?
│   └─ YES → Add [Conflict]
│
└─► END: ≥80% of items have traceability markers
</tree>

</decision-trees>

---

<few-shot-examples>

<example name="Good UX Checklist Item" type="good">
## UX Requirements Quality

- [ ] CHK001 - Are visual hierarchy requirements defined with measurable criteria? [Clarity, Spec §FR-1]
- [ ] CHK002 - Is the number and positioning of UI elements explicitly specified? [Completeness, Spec §FR-1]
- [ ] CHK003 - Are hover state requirements consistently defined across all interactive elements? [Consistency]
- [ ] CHK004 - Are accessibility requirements specified for all interactive elements? [Coverage, Gap]
- [ ] CHK005 - Is fallback behavior defined when images fail to load? [Edge Case, Gap]

✅ Tests requirements quality, not implementation
✅ Includes traceability markers
✅ Uses question format
</example>

<example name="Good API Checklist Item" type="good">
## API Requirements Quality

- [ ] CHK001 - Are error response formats specified for all failure scenarios? [Completeness]
- [ ] CHK002 - Are rate limiting requirements quantified with specific thresholds? [Clarity, Gap]
- [ ] CHK003 - Are authentication requirements consistent across all endpoints? [Consistency]
- [ ] CHK004 - Is versioning strategy documented in requirements? [Gap]

✅ Asks if requirements exist and are clear
✅ Does NOT test if API works
</example>

<example name="Bad - Tests Implementation" type="bad">
## UX Checklist

- [ ] CHK001 - Verify landing page displays 3 episode cards
- [ ] CHK002 - Test hover states work correctly on desktop
- [ ] CHK003 - Confirm logo click navigates to home page

❌ WRONG: "Verify", "Test", "Confirm" = testing implementation
❌ WRONG: No traceability markers
❌ WRONG: No quality dimension tags

✅ CORRECT: "Are episode card layout requirements specified?" [Completeness]
</example>

<example name="Bad - Missing Traceability" type="bad">
- [ ] CHK001 - Are requirements defined?
- [ ] CHK002 - Is the spec clear?
- [ ] CHK003 - Are edge cases covered?

❌ WRONG: Too vague (which requirements?)
❌ WRONG: No spec section references
❌ WRONG: No quality dimension markers

✅ CORRECT: Be specific, reference spec sections, add markers
</example>

</few-shot-examples>

---

<grounding>

<file-locations>
rustation/
├── specs/{NNN}-{name}/
│   ├── spec.md           # INPUT: Requirements to validate
│   ├── plan.md           # INPUT: Technical context (optional)
│   └── checklists/       # OUTPUT: Generated checklists
│       ├── ux.md
│       ├── api.md
│       ├── security.md
│       ├── performance.md
│       └── requirements.md
├── .specify/
│   ├── templates/
│   │   └── checklist-template.md  # Structure template
│   └── scripts/bash/
│       └── check-prerequisites.sh  # Path detection
</file-locations>

<quality-dimensions>
| Dimension | What It Tests | Example Question |
|-----------|---------------|------------------|
| Completeness | All requirements present | "Are error handling requirements defined?" |
| Clarity | Requirements unambiguous | "Is 'fast' quantified with timing?" |
| Consistency | Requirements aligned | "Do nav requirements match across pages?" |
| Measurability | Can verify objectively | "Can 'prominent display' be measured?" |
| Coverage | All scenarios addressed | "Are zero-state scenarios specified?" |
</quality-dimensions>

<traceability-markers>
| Marker | When to Use |
|--------|-------------|
| [Spec §X.Y] | Checking existing requirement |
| [Gap] | Checking for missing requirement |
| [Ambiguity] | Checking for vague language |
| [Conflict] | Checking for contradictions |
| [Assumption] | Checking undocumented assumption |
</traceability-markers>

<prohibited-words>
NEVER use these in checklist items (they test implementation):
- Verify, Test, Confirm, Check (+ behavior)
- "Works", "Functions", "Displays", "Renders"
- "Click", "Navigate", "Load", "Execute"
</prohibited-words>

<commands>
# Get feature paths
.specify/scripts/bash/check-prerequisites.sh --json

# Output: { "FEATURE_DIR": "...", "AVAILABLE_DOCS": [...] }
</commands>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Test implementation behavior → Not a requirements checklist → Test requirements quality</rule>
<rule severity="NEVER">Use "Verify/Test/Confirm" → Implementation language → Use "Are requirements defined/specified/documented?"</rule>
<rule severity="NEVER">Skip traceability → Can't trace to spec → Include [Spec §X.Y] or [Gap] markers</rule>
<rule severity="NEVER">Exceed 40 items → Too long → Prioritize by risk/impact</rule>
<rule severity="NEVER">Vague items → Not actionable → Be specific about which requirement</rule>
<rule severity="NEVER">Mix implementation and requirements → Confusing → Keep pure requirements focus</rule>

<bad-example name="Implementation Testing">
❌ "Verify the button displays correctly"
❌ "Test that the API returns 200 OK"
❌ "Confirm users can log in"

✅ "Are button display requirements specified with dimensions?" [Completeness]
✅ "Are success response codes documented for each endpoint?" [Completeness]
✅ "Are login flow requirements defined for all authentication methods?" [Coverage]
</bad-example>

</negative-constraints>

---

<delimiters>
Use these markers in checklist output:

<marker name="CHECKLIST HEADER">
# [Type] Requirements Quality Checklist: [Feature Name]

**Purpose**: Validate [type] requirements completeness and quality
**Created**: [DATE]
**Feature**: [Link to spec.md]
</marker>

<marker name="CHECKLIST ITEM">
- [ ] CHK### - [Question about requirement quality]? [Dimension, Traceability]
</marker>

<marker name="SECTION">
## [Quality Dimension]

- [ ] CHK001 - ...
- [ ] CHK002 - ...
</marker>
</delimiters>

---

<output-structure>
After generating checklist, report in this format:

<report>
  <checklist>
    <path>specs/062-user-auth/checklists/security.md</path>
    <type>security</type>
    <items>15</items>
  </checklist>

  <focus-areas>
    <area name="Authentication" items="5"/>
    <area name="Data Protection" items="4"/>
    <area name="Access Control" items="3"/>
    <area name="Compliance" items="3"/>
  </focus-areas>

  <traceability>
    <with-markers>12</with-markers>
    <without-markers>3</without-markers>
    <percent>80</percent>
  </traceability>

  <validation>
    <check name="No implementation testing" status="PASS"/>
    <check name="Traceability ≥80%" status="PASS"/>
    <check name="Quality dimension tags" status="PASS"/>
  </validation>

  <next-steps>
    <step>Review checklist items</step>
    <step>Run /speckit.implement when checklists pass</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before completing checklist, verify ALL items:

<checklist name="Content Quality">
  <item>All items test requirements, not implementation?</item>
  <item>No prohibited words (Verify/Test/Confirm + behavior)?</item>
  <item>All items use question format?</item>
  <item>Quality dimension tags included?</item>
</checklist>

<checklist name="Traceability">
  <item>≥80% of items have traceability markers?</item>
  <item>[Spec §X.Y] used for existing requirements?</item>
  <item>[Gap] used for missing requirements?</item>
</checklist>

<checklist name="Format">
  <item>Items start with - [ ] CHK###?</item>
  <item>IDs sequential (CHK001, CHK002...)?</item>
  <item>Items ≤40 total?</item>
  <item>Saved to checklists/ directory?</item>
</checklist>

If ANY item is NO, fix it before completing.
</self-correction>

---

<quick-reference>
CHECKLIST WORKFLOW:
  1. Run check-prerequisites.sh --json
  2. Analyze user input → determine focus and type
  3. Ask up to 3 clarifying questions (if needed)
  4. Load spec.md and plan.md (if exists)
  5. Generate items by quality dimension:
     - Completeness: "Are X defined?"
     - Clarity: "Is Y quantified?"
     - Consistency: "Do A and B align?"
     - Measurability: "Can Z be verified?"
     - Coverage: "Are edge cases addressed?"
  6. Add traceability markers (≥80%)
  7. Save to checklists/{type}.md

ITEM FORMAT:
  - [ ] CHK001 - [Question]? [Dimension, Traceability]

QUALITY PATTERNS:
  ✅ "Are [requirements] defined/specified/documented for [scenario]?"
  ✅ "Is [vague term] quantified/clarified with specific criteria?"
  ✅ "Are requirements consistent between [A] and [B]?"
  ❌ "Verify [implementation behavior]"
  ❌ "Test [system action]"
  ❌ "Confirm [user action works]"
</quick-reference>

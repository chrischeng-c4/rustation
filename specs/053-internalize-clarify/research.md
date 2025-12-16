# Research: Internalize Clarify Workflow

**Feature**: 053-internalize-clarify
**Date**: 2025-12-16

## Research Tasks

### 1. Existing Clarify Workflow Analysis

**Task**: Analyze `.claude/commands/speckit.clarify.md` to understand workflow.

**Findings**:
- 11 taxonomy categories for ambiguity detection
- Coverage status: Clear, Partial, Missing
- Max 5 questions per session
- Multiple-choice (2-5 options) or short-answer (<=5 words)
- Recommended answer with reasoning
- Incremental spec updates after each answer
- Atomic file writes

**Decision**: Replicate core logic in Rust, use Claude CLI for question generation.

**Rationale**: Analysis can be done locally, but question generation benefits from LLM.

### 2. Taxonomy Categories

**Task**: Define the 11 ambiguity categories for analysis.

**Categories** (from clarify command):
1. Functional Scope & Behavior
2. Domain & Data Model
3. Interaction & UX Flow
4. Non-Functional Quality Attributes
5. Integration & External Dependencies
6. Edge Cases & Failure Handling
7. Constraints & Tradeoffs
8. Terminology & Consistency
9. Completion Signals
10. Misc / Placeholders

**Decision**: Implement as Rust enum with associated keywords for pattern matching.

**Rationale**: Simple keyword-based analysis is fast and sufficient for initial coverage detection.

### 3. Coverage Detection Strategy

**Task**: Determine how to classify category coverage.

**Approach**:
1. Parse spec into sections (by `##` headers)
2. For each category, define keyword patterns
3. Score presence: count matches, check for TODO/placeholder markers
4. Classify:
   - **Clear**: Multiple matches, no TODOs
   - **Partial**: Some matches or has TODOs
   - **Missing**: No relevant content found

**Example patterns**:
```rust
Category::FunctionalScope => &["goal", "success criteria", "out of scope", "must", "shall"],
Category::DomainDataModel => &["entity", "field", "relationship", "attribute", "model"],
Category::NonFunctional => &["performance", "latency", "throughput", "scalability", "security"],
```

**Decision**: Use regex-based keyword scanning with configurable patterns.

**Rationale**: Fast, deterministic, easily testable. Claude generates questions for deeper analysis.

### 4. Question Generation Strategy

**Task**: Determine how to generate clarification questions.

**Alternatives**:
1. **Hardcoded templates** - Fast but inflexible
2. **Claude CLI** - Smart but slower
3. **Hybrid** - Local prioritization, Claude for phrasing

**Decision**: Use hybrid approach:
- Local analysis determines which categories need questions
- Local prioritization by (Impact × Uncertainty)
- Claude CLI generates the actual question text and options
- Fallback to templates if Claude unavailable

**Rationale**: Balance between quality and performance.

### 5. Spec Integration Strategy

**Task**: Define how to update spec files with clarifications.

**Structure**:
```markdown
## Clarifications

### Session 2025-12-16

- Q: What authentication method? → A: OAuth2 with PKCE
- Q: Max response time? → A: 200ms p95
```

**Update locations**:
- Create `## Clarifications` section if missing (after Overview)
- Add session header with date
- Append Q&A bullets
- Update relevant sections based on category

**Decision**: Parse spec as sections, modify in-memory, atomic write.

**Rationale**: Preserves formatting, enables rollback, consistent with specify module.

### 6. Session State Management

**Task**: Define session lifecycle and state.

**State machine**:
```
Start → Analyze → Generate Questions → Ask Question → Await Answer
                                            ↓
                                      Validate Answer
                                            ↓
                                      Integrate Answer
                                            ↓
                                  More Questions? → Ask Question
                                            ↓ No
                                      Finalize → Report
```

**Decision**: `ClarifySession` struct holds all state, persisted in memory during session.

**Rationale**: Simple, no external state needed, easy to test.

### 7. Answer Validation

**Task**: Define answer validation rules.

**Rules**:
- Multiple-choice: Must match option letter (A-E) or "yes"/"recommended"
- Short-answer: Max 5 words, non-empty
- Allow retry on invalid input

**Decision**: Implement `validate_answer()` function with clear error messages.

**Rationale**: User-friendly, prevents invalid data in spec.

## Summary of Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Analysis | Keyword-based pattern matching | Fast, testable |
| Question Generation | Hybrid: local priority, Claude phrasing | Quality + performance |
| Integration | Section-based parsing, atomic write | Preserves formatting |
| Session | In-memory state machine | Simple, testable |
| Validation | Type-specific rules with retry | User-friendly |

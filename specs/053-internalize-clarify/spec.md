# Feature 053: Internalize Clarify Workflow

## Overview

Move the `/speckit.clarify` workflow logic from Claude Code slash command into native Rust code within rstn. This enables the TUI to perform spec ambiguity analysis and interactive clarification without shelling out to external processes.

## Problem Statement

Current clarify workflow has limitations:

1. **External dependency**: Relies on Claude Code CLI and slash command files
2. **No TUI integration**: Cannot run clarify interactively within the rstn TUI
3. **Limited control**: Cannot customize question generation or analysis depth
4. **Slow feedback**: Each clarification requires full Claude API round-trip
5. **No caching**: Repeated analysis of same spec re-analyzes everything

## Dependencies

**Depends on:**
- Feature 052 (Internalize Spec Generation) - Provides base `specify` module patterns
- Feature 051 (Interactive Specify Flow) - Provides TUI input patterns

**Reason:**
- Reuses types like `SpecifyError`, file operations patterns
- Integrates with TUI for interactive Q&A

## User Stories

### US1 - As an rstn user analyzing specs
- I want to detect ambiguities in my spec file
- So that I can identify what needs clarification before planning

### US2 - As an rstn user clarifying specs
- I want to answer clarification questions interactively
- So that I can refine my spec without leaving the TUI

### US3 - As an rstn developer
- I want clarifications integrated back into the spec file
- So that the spec remains the single source of truth

### US4 - As a contributor
- I want the clarify logic to be testable
- So that I can ensure reliability and catch regressions

## Requirements

### Functional Requirements

**FR-1: Spec Analysis**
- Load and parse spec.md file
- Scan for ambiguity categories (11 taxonomy areas)
- Classify each area as: Clear, Partial, Missing
- Generate coverage map for prioritization

**FR-2: Question Generation**
- Generate up to 5 prioritized questions
- Support multiple-choice (2-5 options) or short-answer format
- Include recommended answer with reasoning
- Prioritize by (Impact × Uncertainty) heuristic

**FR-3: Interactive Q&A**
- Present one question at a time
- Accept: option letter, "yes"/"recommended", or custom answer
- Validate answer format (<=5 words for short-answer)
- Track question count (max 5)

**FR-4: Spec Integration**
- Create/update `## Clarifications` section
- Add session header `### Session YYYY-MM-DD`
- Append Q&A bullet for each answer
- Update relevant spec sections with clarified info
- Atomic file writes (temp + rename)

**FR-5: Validation**
- No duplicate clarification bullets
- No contradictory statements after update
- Terminology consistency across sections
- Valid markdown structure preserved

**FR-6: Completion Report**
- Questions asked/answered count
- Sections touched
- Coverage summary table
- Outstanding/deferred items
- Suggested next command

### Non-Functional Requirements

**NFR-1: Performance**
- Spec analysis: <500ms for typical spec
- File operations: <100ms
- No blocking UI thread

**NFR-2: Reliability**
- Atomic file updates
- Rollback on error
- Preserve spec formatting

**NFR-3: Testability**
- Unit tests for analysis logic
- Mock Claude for question generation
- Integration tests with temp files

## Architecture

### Module Structure

**New module:** `crates/rstn-core/src/clarify/mod.rs`
```rust
pub mod analyzer;        // Spec ambiguity analysis
pub mod question;        // Question generation
pub mod integrator;      // Spec file integration
pub mod session;         // Q&A session management
```

### Core Types

```rust
/// Ambiguity category from taxonomy
#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    FunctionalScope,
    DomainDataModel,
    InteractionUxFlow,
    NonFunctionalQuality,
    IntegrationDependencies,
    EdgeCasesFailures,
    ConstraintsTradeoffs,
    TerminologyConsistency,
    CompletionSignals,
    MiscPlaceholders,
}

/// Category coverage status
#[derive(Debug, Clone, PartialEq)]
pub enum CoverageStatus {
    Clear,      // Already sufficient
    Partial,    // Some info, needs more
    Missing,    // No info present
    Resolved,   // Was Partial/Missing, now addressed
    Deferred,   // Exceeds quota or better for planning
    Outstanding,// Still Partial/Missing, low impact
}

/// A clarification question
#[derive(Debug, Clone)]
pub struct Question {
    pub id: usize,
    pub category: Category,
    pub text: String,
    pub format: QuestionFormat,
    pub recommended: Option<String>,
    pub reasoning: Option<String>,
}

/// Question format
#[derive(Debug, Clone)]
pub enum QuestionFormat {
    MultipleChoice { options: Vec<(char, String)> },
    ShortAnswer { max_words: usize },
}

/// Answer to a question
#[derive(Debug, Clone)]
pub struct Answer {
    pub question_id: usize,
    pub value: String,
    pub accepted_recommendation: bool,
}

/// Clarify session state
#[derive(Debug)]
pub struct ClarifySession {
    pub spec_path: PathBuf,
    pub spec_content: String,
    pub coverage_map: HashMap<Category, CoverageStatus>,
    pub questions: VecDeque<Question>,
    pub answers: Vec<Answer>,
    pub questions_asked: usize,
}

/// Clarify errors
#[derive(Debug, thiserror::Error)]
pub enum ClarifyError {
    #[error("Spec file not found: {0}")]
    SpecNotFound(PathBuf),

    #[error("Failed to read spec: {0}")]
    SpecRead(#[source] std::io::Error),

    #[error("Failed to parse spec structure")]
    SpecParse,

    #[error("Failed to write spec: {0}")]
    SpecWrite(#[source] std::io::Error),

    #[error("Question quota exceeded (max 5)")]
    QuotaExceeded,

    #[error("Invalid answer format")]
    InvalidAnswer,

    #[error("Claude CLI error: {0}")]
    ClaudeError(String),
}
```

### Main API

```rust
/// Analyze a spec file for ambiguities
pub fn analyze_spec(spec_path: &Path) -> Result<CoverageMap, ClarifyError>;

/// Generate prioritized questions from coverage map
pub async fn generate_questions(
    spec_content: &str,
    coverage_map: &CoverageMap,
    max_questions: usize,
) -> Result<Vec<Question>, ClarifyError>;

/// Start a clarify session
pub fn start_session(spec_path: PathBuf) -> Result<ClarifySession, ClarifyError>;

/// Submit an answer and get next question (or None if done)
pub fn submit_answer(
    session: &mut ClarifySession,
    answer: Answer,
) -> Result<Option<Question>, ClarifyError>;

/// Integrate all answers into spec file
pub fn finalize_session(session: &ClarifySession) -> Result<ClarifyReport, ClarifyError>;
```

## Taxonomy Categories

The analyzer scans for these 11 ambiguity categories:

1. **Functional Scope & Behavior** - Goals, success criteria, out-of-scope
2. **Domain & Data Model** - Entities, attributes, relationships, lifecycle
3. **Interaction & UX Flow** - User journeys, error states, accessibility
4. **Non-Functional Quality** - Performance, scalability, reliability, security
5. **Integration & Dependencies** - External APIs, failure modes, protocols
6. **Edge Cases & Failures** - Negative scenarios, rate limiting, conflicts
7. **Constraints & Tradeoffs** - Technical constraints, rejected alternatives
8. **Terminology & Consistency** - Glossary terms, avoided synonyms
9. **Completion Signals** - Acceptance criteria, Definition of Done
10. **Misc / Placeholders** - TODOs, vague adjectives

## Integration with TUI

The TUI will integrate clarify as a new action in the Worktree view:

```rust
ViewAction::StartClarify { spec_path } => {
    let session = rstn_core::clarify::start_session(spec_path)?;
    self.clarify_session = Some(session);
    self.view = View::ClarifyQuestion;
}

ViewAction::AnswerClarify { answer } => {
    if let Some(session) = &mut self.clarify_session {
        match rstn_core::clarify::submit_answer(session, answer)? {
            Some(next_question) => {
                self.current_question = Some(next_question);
            }
            None => {
                let report = rstn_core::clarify::finalize_session(session)?;
                self.clarify_session = None;
                self.show_clarify_report(report);
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests

**Analyzer:**
- Empty spec → all Missing
- Complete spec → all Clear
- Partial spec → mixed statuses
- Markdown parsing edge cases

**Question Generation:**
- Prioritization by impact
- Format selection (MC vs short)
- Max 5 questions enforced

**Integrator:**
- Creates Clarifications section
- Appends to existing section
- Updates relevant sections
- Atomic write verified

### Integration Tests

- Full session with mock Claude
- Spec file before/after comparison
- Rollback on error

## Success Metrics

- Analysis time <500ms for 500-line spec
- Zero data loss on integration
- All tests pass
- Clippy clean

## Notes

- This feature focuses on the core logic; TUI integration is separate
- Question generation may still use Claude CLI for complex analysis
- Future: local LLM for faster question generation

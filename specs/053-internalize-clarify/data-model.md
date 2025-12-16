# Data Model: Internalize Clarify Workflow

**Feature**: 053-internalize-clarify
**Date**: 2025-12-16

## Core Types

### Category

Taxonomy category for ambiguity analysis.

```rust
/// Ambiguity category from the clarify taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Core user goals, success criteria, out-of-scope
    FunctionalScope,

    /// Entities, attributes, relationships, lifecycle
    DomainDataModel,

    /// User journeys, error states, accessibility
    InteractionUxFlow,

    /// Performance, scalability, reliability, security
    NonFunctionalQuality,

    /// External APIs, failure modes, protocols
    IntegrationDependencies,

    /// Negative scenarios, rate limiting, conflicts
    EdgeCasesFailures,

    /// Technical constraints, rejected alternatives
    ConstraintsTradeoffs,

    /// Glossary terms, avoided synonyms
    TerminologyConsistency,

    /// Acceptance criteria, Definition of Done
    CompletionSignals,

    /// TODOs, vague adjectives, placeholders
    MiscPlaceholders,
}

impl Category {
    /// Keywords associated with this category for pattern matching
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::FunctionalScope => &["goal", "success", "scope", "must", "shall", "requirement"],
            Self::DomainDataModel => &["entity", "field", "model", "attribute", "relationship", "schema"],
            Self::InteractionUxFlow => &["user", "journey", "flow", "error", "loading", "accessibility"],
            Self::NonFunctionalQuality => &["performance", "latency", "scale", "security", "reliability"],
            Self::IntegrationDependencies => &["api", "external", "service", "integration", "dependency"],
            Self::EdgeCasesFailures => &["edge case", "error", "failure", "timeout", "retry"],
            Self::ConstraintsTradeoffs => &["constraint", "tradeoff", "limitation", "alternative"],
            Self::TerminologyConsistency => &["glossary", "term", "definition", "synonym"],
            Self::CompletionSignals => &["acceptance", "criteria", "done", "complete", "verify"],
            Self::MiscPlaceholders => &["todo", "tbd", "placeholder", "unclear", "tbc"],
        }
    }

    /// All categories for iteration
    pub fn all() -> &'static [Category] {
        &[
            Self::FunctionalScope,
            Self::DomainDataModel,
            Self::InteractionUxFlow,
            Self::NonFunctionalQuality,
            Self::IntegrationDependencies,
            Self::EdgeCasesFailures,
            Self::ConstraintsTradeoffs,
            Self::TerminologyConsistency,
            Self::CompletionSignals,
            Self::MiscPlaceholders,
        ]
    }
}
```

### CoverageStatus

Status of a category's coverage in the spec.

```rust
/// Coverage status for a taxonomy category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    /// Already sufficient - no clarification needed
    Clear,

    /// Some info present but incomplete
    Partial,

    /// No relevant content found
    Missing,

    /// Was Partial/Missing, now addressed by clarification
    Resolved,

    /// Exceeds quota or better suited for planning phase
    Deferred,

    /// Still Partial/Missing but low impact
    Outstanding,
}
```

### CoverageMap

Analysis result mapping categories to their status.

```rust
/// Coverage map from spec analysis
pub type CoverageMap = HashMap<Category, CoverageStatus>;

/// Analysis result with coverage details
#[derive(Debug)]
pub struct AnalysisResult {
    /// Coverage status for each category
    pub coverage: CoverageMap,

    /// Categories needing clarification (Partial or Missing)
    pub needs_clarification: Vec<Category>,

    /// Raw match counts per category
    pub match_counts: HashMap<Category, usize>,
}
```

### Question Types

```rust
/// A clarification question
#[derive(Debug, Clone)]
pub struct Question {
    /// Question ID (1-5)
    pub id: usize,

    /// Category this question addresses
    pub category: Category,

    /// Question text
    pub text: String,

    /// Question format
    pub format: QuestionFormat,

    /// Recommended answer (if applicable)
    pub recommended: Option<RecommendedAnswer>,

    /// Impact score (higher = more important)
    pub impact: u8,
}

/// Question format
#[derive(Debug, Clone)]
pub enum QuestionFormat {
    /// Multiple choice with 2-5 options
    MultipleChoice {
        options: Vec<QuestionOption>,
    },

    /// Short answer with word limit
    ShortAnswer {
        max_words: usize,
    },
}

/// A single option in multiple choice
#[derive(Debug, Clone)]
pub struct QuestionOption {
    /// Option letter (A, B, C, D, E)
    pub letter: char,

    /// Option description
    pub description: String,
}

/// Recommended answer with reasoning
#[derive(Debug, Clone)]
pub struct RecommendedAnswer {
    /// The recommended value (option letter or text)
    pub value: String,

    /// Reasoning for recommendation
    pub reasoning: String,
}
```

### Answer

```rust
/// An answer to a clarification question
#[derive(Debug, Clone)]
pub struct Answer {
    /// ID of the question being answered
    pub question_id: usize,

    /// The answer value
    pub value: String,

    /// Whether user accepted the recommendation
    pub accepted_recommendation: bool,

    /// Original question text (for recording)
    pub question_text: String,
}
```

### ClarifySession

```rust
/// State of a clarification session
#[derive(Debug)]
pub struct ClarifySession {
    /// Path to the spec file
    pub spec_path: PathBuf,

    /// Original spec content (for rollback)
    pub original_content: String,

    /// Current spec content (being modified)
    pub current_content: String,

    /// Coverage analysis result
    pub analysis: AnalysisResult,

    /// Queue of questions to ask
    pub question_queue: VecDeque<Question>,

    /// Answers collected so far
    pub answers: Vec<Answer>,

    /// Number of questions asked (max 5)
    pub questions_asked: usize,

    /// Session date for header
    pub session_date: String,
}

impl ClarifySession {
    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        self.questions_asked >= 5 || self.question_queue.is_empty()
    }

    /// Get next question (if any)
    pub fn next_question(&mut self) -> Option<Question> {
        if self.is_complete() {
            return None;
        }
        self.question_queue.pop_front()
    }
}
```

### ClarifyReport

```rust
/// Report generated after clarification session
#[derive(Debug)]
pub struct ClarifyReport {
    /// Path to updated spec file
    pub spec_path: PathBuf,

    /// Number of questions asked
    pub questions_asked: usize,

    /// Number of questions answered
    pub questions_answered: usize,

    /// Sections touched during integration
    pub sections_touched: Vec<String>,

    /// Final coverage summary
    pub coverage_summary: CoverageMap,

    /// Categories still needing attention
    pub outstanding: Vec<Category>,

    /// Categories deferred to planning
    pub deferred: Vec<Category>,

    /// Suggested next command
    pub suggested_next: String,
}
```

### ClarifyError

```rust
/// Errors that can occur during clarification
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

    #[error("No ambiguities found - spec is sufficiently clear")]
    NoClarificationNeeded,

    #[error("Question quota exceeded (max 5)")]
    QuotaExceeded,

    #[error("Invalid answer: {0}")]
    InvalidAnswer(String),

    #[error("Session already complete")]
    SessionComplete,

    #[error("Claude CLI error: {0}")]
    ClaudeError(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}
```

## Configuration

```rust
/// Configuration for clarify operations
#[derive(Debug, Clone)]
pub struct ClarifyConfig {
    /// Maximum questions per session (default: 5)
    pub max_questions: usize,

    /// Maximum words for short answers (default: 5)
    pub max_answer_words: usize,

    /// Claude timeout in seconds (default: 60)
    pub claude_timeout_secs: u64,

    /// Whether to use Claude for question generation (default: true)
    pub use_claude: bool,
}

impl Default for ClarifyConfig {
    fn default() -> Self {
        Self {
            max_questions: 5,
            max_answer_words: 5,
            claude_timeout_secs: 60,
            use_claude: true,
        }
    }
}
```

## State Transitions

### Session Lifecycle

```
                    ┌─────────────┐
                    │    Start    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Analyze   │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │ No ambiguities?         │
              ▼                         ▼
       ┌──────────┐              ┌─────────────┐
       │  Report  │              │  Generate   │
       │ (Clear)  │              │  Questions  │
       └──────────┘              └──────┬──────┘
                                        │
                                        ▼
                                 ┌─────────────┐
                           ┌────►│ Ask Question│◄────┐
                           │     └──────┬──────┘     │
                           │            │            │
                           │            ▼            │
                           │     ┌─────────────┐     │
                           │     │Await Answer │     │
                           │     └──────┬──────┘     │
                           │            │            │
                           │            ▼            │
                           │     ┌─────────────┐     │
                           │     │  Validate   │     │
                           │     └──────┬──────┘     │
                           │            │            │
                           │  Invalid   │  Valid     │
                           └────────────┤            │
                                        ▼            │
                                 ┌─────────────┐     │
                                 │  Integrate  │     │
                                 └──────┬──────┘     │
                                        │            │
                              ┌─────────┴─────────┐  │
                              │ More questions?   │  │
                              ▼                   ▼  │
                       ┌──────────┐         Yes ─────┘
                       │ Finalize │
                       └──────┬───┘
                              │
                              ▼
                       ┌──────────┐
                       │  Report  │
                       └──────────┘
```

## Validation Rules

### Answer Validation

**Multiple Choice**:
- Must be single letter A-E (case-insensitive)
- Or "yes", "recommended", "suggested" to accept recommendation
- Or "short" followed by custom answer

**Short Answer**:
- Must be non-empty
- Must be ≤ max_words (default 5)
- Trimmed of whitespace

### Spec Structure Validation

- `## Clarifications` section properly formatted
- Session headers use `### Session YYYY-MM-DD` format
- Q&A bullets use `- Q: ... → A: ...` format
- No duplicate Q&A entries
- Markdown structure preserved

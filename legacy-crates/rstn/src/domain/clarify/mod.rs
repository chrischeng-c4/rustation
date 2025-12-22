//! Clarification workflow module for rstn
//!
//! This module provides functionality to analyze spec files for ambiguities
//! and guide users through interactive clarification sessions.
//!
//! # Components
//!
//! - [`analyzer`]: Spec analysis and coverage mapping
//! - [`question`]: Question generation and prioritization
//! - [`session`]: Q&A session state management
//! - [`integrator`]: Spec file updates and atomic writes

pub mod analyzer;
pub mod integrator;
pub mod question;
pub mod session;

use std::collections::HashMap;
use std::path::PathBuf;

// Re-exports will be added as components are implemented

/// Ambiguity category from the clarify taxonomy
///
/// These 10 categories cover the main areas where specs commonly
/// have gaps or ambiguities that need clarification.
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
            Self::FunctionalScope => &[
                "goal",
                "success",
                "scope",
                "must",
                "shall",
                "requirement",
                "objective",
            ],
            Self::DomainDataModel => &[
                "entity",
                "field",
                "model",
                "attribute",
                "relationship",
                "schema",
                "data",
                "struct",
            ],
            Self::InteractionUxFlow => &[
                "user",
                "journey",
                "flow",
                "error",
                "loading",
                "accessibility",
                "interaction",
                "ui",
                "ux",
            ],
            Self::NonFunctionalQuality => &[
                "performance",
                "latency",
                "scale",
                "security",
                "reliability",
                "throughput",
                "availability",
            ],
            Self::IntegrationDependencies => &[
                "api",
                "external",
                "service",
                "integration",
                "dependency",
                "protocol",
                "endpoint",
            ],
            Self::EdgeCasesFailures => &[
                "edge case",
                "error",
                "failure",
                "timeout",
                "retry",
                "fallback",
                "exception",
            ],
            Self::ConstraintsTradeoffs => &[
                "constraint",
                "tradeoff",
                "limitation",
                "alternative",
                "decision",
                "rejected",
            ],
            Self::TerminologyConsistency => &[
                "glossary",
                "term",
                "definition",
                "synonym",
                "naming",
                "convention",
            ],
            Self::CompletionSignals => &[
                "acceptance",
                "criteria",
                "done",
                "complete",
                "verify",
                "definition of done",
            ],
            Self::MiscPlaceholders => &["todo", "tbd", "placeholder", "unclear", "tbc", "fixme"],
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

    /// Human-readable name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FunctionalScope => "Functional Scope & Behavior",
            Self::DomainDataModel => "Domain & Data Model",
            Self::InteractionUxFlow => "Interaction & UX Flow",
            Self::NonFunctionalQuality => "Non-Functional Quality",
            Self::IntegrationDependencies => "Integration & Dependencies",
            Self::EdgeCasesFailures => "Edge Cases & Failures",
            Self::ConstraintsTradeoffs => "Constraints & Tradeoffs",
            Self::TerminologyConsistency => "Terminology & Consistency",
            Self::CompletionSignals => "Completion Signals",
            Self::MiscPlaceholders => "Misc / Placeholders",
        }
    }
}

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

/// Question format
#[derive(Debug, Clone)]
pub enum QuestionFormat {
    /// Multiple choice with 2-5 options
    MultipleChoice { options: Vec<QuestionOption> },

    /// Short answer with word limit
    ShortAnswer { max_words: usize },
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

    #[error("Question quota exceeded (max {0})")]
    QuotaExceeded(usize),

    #[error("Invalid answer: {0}")]
    InvalidAnswer(String),

    #[error("Session already complete")]
    SessionComplete,

    #[error("Claude CLI error: {0}")]
    ClaudeError(String),

    #[error("Claude CLI timed out after {0} seconds")]
    ClaudeTimeout(u64),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_keywords() {
        let category = Category::FunctionalScope;
        let keywords = category.keywords();
        assert!(keywords.contains(&"goal"));
        assert!(keywords.contains(&"success"));
    }

    #[test]
    fn test_category_all() {
        let all = Category::all();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn test_clarify_config_default() {
        let config = ClarifyConfig::default();
        assert_eq!(config.max_questions, 5);
        assert_eq!(config.max_answer_words, 5);
        assert_eq!(config.claude_timeout_secs, 60);
        assert!(config.use_claude);
    }

    #[test]
    fn test_coverage_status_equality() {
        assert_eq!(CoverageStatus::Clear, CoverageStatus::Clear);
        assert_ne!(CoverageStatus::Clear, CoverageStatus::Partial);
    }
}

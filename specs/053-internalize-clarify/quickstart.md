# Quickstart: Internalize Clarify Workflow

**Feature**: 053-internalize-clarify
**Date**: 2025-12-16

## Quick Start

### Using the Clarify Module

```rust
use rstn_core::clarify::{start_session, submit_answer, finalize_session, Answer};
use std::path::PathBuf;

// Start a clarification session
let spec_path = PathBuf::from("specs/001-my-feature/spec.md");
let mut session = start_session(spec_path)?;

// Check coverage analysis
println!("Categories needing clarification: {:?}", session.analysis.needs_clarification);

// Get first question
if let Some(question) = session.next_question() {
    println!("Q{}: {}", question.id, question.text);

    // Submit an answer
    let answer = Answer {
        question_id: question.id,
        value: "A".to_string(),  // Option A selected
        accepted_recommendation: true,
        question_text: question.text.clone(),
    };

    let next_q = submit_answer(&mut session, answer)?;
}

// Finalize and get report
let report = finalize_session(&session)?;
println!("Updated spec: {:?}", report.spec_path);
println!("Sections touched: {:?}", report.sections_touched);
```

### Analyzing a Spec

```rust
use rstn_core::clarify::{analyze_spec, Category, CoverageStatus};
use std::path::Path;

let coverage = analyze_spec(Path::new("specs/001-my-feature/spec.md"))?;

for (category, status) in &coverage {
    match status {
        CoverageStatus::Clear => println!("✓ {:?} is well-defined", category),
        CoverageStatus::Partial => println!("⚠ {:?} needs more detail", category),
        CoverageStatus::Missing => println!("✗ {:?} is not addressed", category),
        _ => {}
    }
}
```

### Question Formats

**Multiple Choice:**
```rust
let question = Question {
    id: 1,
    category: Category::NonFunctionalQuality,
    text: "What is the expected response time for API calls?".to_string(),
    format: QuestionFormat::MultipleChoice {
        options: vec![
            QuestionOption { letter: 'A', description: "< 100ms (real-time)".to_string() },
            QuestionOption { letter: 'B', description: "< 500ms (interactive)".to_string() },
            QuestionOption { letter: 'C', description: "< 2s (batch)".to_string() },
        ],
    },
    recommended: Some(RecommendedAnswer {
        value: "B".to_string(),
        reasoning: "Typical for interactive CLI applications".to_string(),
    }),
    impact: 8,
};
```

**Short Answer:**
```rust
let question = Question {
    id: 2,
    category: Category::TerminologyConsistency,
    text: "What should we call the main configuration file?".to_string(),
    format: QuestionFormat::ShortAnswer { max_words: 5 },
    recommended: Some(RecommendedAnswer {
        value: "config.toml".to_string(),
        reasoning: "Matches Rust ecosystem conventions".to_string(),
    }),
    impact: 5,
};
```

## Taxonomy Categories

The analyzer checks 10 categories:

| Category | Keywords | Example Gaps |
|----------|----------|--------------|
| FunctionalScope | goal, success, must | Missing success criteria |
| DomainDataModel | entity, field, model | Undefined data structures |
| InteractionUxFlow | user, journey, flow | Missing error states |
| NonFunctionalQuality | performance, security | No latency requirements |
| IntegrationDependencies | api, external, service | Undefined API contracts |
| EdgeCasesFailures | edge case, error, retry | Missing failure handling |
| ConstraintsTradeoffs | constraint, limitation | Undocumented tradeoffs |
| TerminologyConsistency | glossary, term | Inconsistent naming |
| CompletionSignals | acceptance, done | No Definition of Done |
| MiscPlaceholders | todo, tbd, unclear | Unresolved placeholders |

## Session Flow

```
1. start_session(spec_path)
   └── Loads spec, runs analysis, generates question queue

2. session.next_question()
   └── Returns prioritized Question or None if done

3. submit_answer(&mut session, answer)
   └── Validates answer, increments counter, returns next question

4. finalize_session(&session)
   └── Integrates answers into spec, returns ClarifyReport
```

## Error Handling

```rust
use rstn_core::clarify::ClarifyError;

match start_session(spec_path) {
    Ok(session) => { /* proceed */ },
    Err(ClarifyError::SpecNotFound(path)) => {
        eprintln!("Spec not found: {}", path.display());
    },
    Err(ClarifyError::NoClarificationNeeded) => {
        println!("Spec is already clear - no questions needed!");
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration Spec Format

After clarification, the spec file gains a `## Clarifications` section:

```markdown
## Clarifications

### Session 2025-12-16

- Q: What authentication method? → A: OAuth2 with PKCE
- Q: Max response time? → A: 200ms p95
- Q: Rate limiting strategy? → A: Token bucket (100 req/min)
```

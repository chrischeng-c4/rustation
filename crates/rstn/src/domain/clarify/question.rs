//! Question generation and prioritization
//!
//! This module generates clarification questions based on coverage analysis
//! and prioritizes them by impact.

use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

use super::{
    AnalysisResult, Category, ClarifyConfig, ClarifyError, CoverageStatus, Question,
    QuestionFormat, QuestionOption, RecommendedAnswer,
};

/// Calculate impact score for a category (higher = more important)
pub fn calculate_impact(category: Category) -> u8 {
    match category {
        Category::FunctionalScope => 10,
        Category::DomainDataModel => 9,
        Category::NonFunctionalQuality => 8,
        Category::InteractionUxFlow => 7,
        Category::IntegrationDependencies => 7,
        Category::EdgeCasesFailures => 6,
        Category::ConstraintsTradeoffs => 5,
        Category::CompletionSignals => 5,
        Category::TerminologyConsistency => 4,
        Category::MiscPlaceholders => 3,
    }
}

/// Prioritize categories by (Impact × Uncertainty)
///
/// Categories with Missing status get higher uncertainty multiplier than Partial.
pub fn prioritize_categories(analysis: &AnalysisResult) -> Vec<(Category, u8)> {
    let mut priorities: Vec<(Category, u8)> = analysis
        .needs_clarification
        .iter()
        .map(|cat| {
            let impact = calculate_impact(*cat);
            let uncertainty_multiplier = match analysis.coverage.get(cat) {
                Some(CoverageStatus::Missing) => 2,
                Some(CoverageStatus::Partial) => 1,
                _ => 0,
            };
            (*cat, impact * uncertainty_multiplier)
        })
        .collect();

    priorities.sort_by(|a, b| b.1.cmp(&a.1));
    priorities
}

/// Build prompt for Claude CLI to generate a question
pub fn build_question_prompt(category: Category, spec_content: &str) -> String {
    format!(
        r#"Generate a single clarification question for a feature specification.

Category: {}
Description: This category covers {}

Current spec content:
{}

Generate a question that:
1. Addresses a gap in the {} category
2. Is specific and actionable
3. Has 2-4 multiple choice options OR requires a short answer (max 5 words)
4. Includes a recommended answer with reasoning

Output format (JSON):
{{
  "question": "Your question here?",
  "format": "multiple_choice",
  "options": [
    {{"letter": "A", "description": "Option A"}},
    {{"letter": "B", "description": "Option B"}}
  ],
  "recommended": {{"value": "A", "reasoning": "Why A is recommended"}}
}}

OR for short answer:
{{
  "question": "Your question here?",
  "format": "short_answer",
  "recommended": {{"value": "suggested answer", "reasoning": "Why this is recommended"}}
}}"#,
        category.display_name(),
        get_category_description(category),
        spec_content,
        category.display_name()
    )
}

fn get_category_description(category: Category) -> &'static str {
    match category {
        Category::FunctionalScope => "core user goals, success criteria, and out-of-scope items",
        Category::DomainDataModel => "entities, attributes, relationships, and data lifecycle",
        Category::InteractionUxFlow => "user journeys, error states, and accessibility",
        Category::NonFunctionalQuality => "performance, scalability, reliability, and security",
        Category::IntegrationDependencies => "external APIs, failure modes, and protocols",
        Category::EdgeCasesFailures => "negative scenarios, rate limiting, and conflicts",
        Category::ConstraintsTradeoffs => "technical constraints and rejected alternatives",
        Category::TerminologyConsistency => "glossary terms and avoided synonyms",
        Category::CompletionSignals => "acceptance criteria and Definition of Done",
        Category::MiscPlaceholders => "TODOs, vague adjectives, and placeholders",
    }
}

/// Call Claude CLI to generate a question
pub async fn call_claude_cli(prompt: &str, config: &ClarifyConfig) -> Result<String, ClarifyError> {
    let timeout_duration = Duration::from_secs(config.claude_timeout_secs);

    let output = timeout(
        timeout_duration,
        Command::new("claude")
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .arg(prompt)
            .output(),
    )
    .await
    .map_err(|_| ClarifyError::ClaudeTimeout(config.claude_timeout_secs))?
    .map_err(|e| ClarifyError::ClaudeError(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ClarifyError::ClaudeError(stderr))
    }
}

/// Parse Claude response to extract question
pub fn parse_claude_response(
    response: &str,
    category: Category,
    question_id: usize,
) -> Option<Question> {
    // Try to extract JSON from response
    let json_start = response.find('{')?;
    let json_end = response.rfind('}')? + 1;
    let json_str = &response[json_start..json_end];

    let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;

    let question_text = parsed.get("question")?.as_str()?.to_string();
    let format_str = parsed.get("format")?.as_str()?;

    let format = if format_str == "multiple_choice" {
        let options_arr = parsed.get("options")?.as_array()?;
        let options: Vec<QuestionOption> = options_arr
            .iter()
            .filter_map(|opt| {
                Some(QuestionOption {
                    letter: opt.get("letter")?.as_str()?.chars().next()?,
                    description: opt.get("description")?.as_str()?.to_string(),
                })
            })
            .collect();
        QuestionFormat::MultipleChoice { options }
    } else {
        QuestionFormat::ShortAnswer { max_words: 5 }
    };

    let recommended = parsed.get("recommended").and_then(|rec| {
        Some(RecommendedAnswer {
            value: rec.get("value")?.as_str()?.to_string(),
            reasoning: rec.get("reasoning")?.as_str()?.to_string(),
        })
    });

    Some(Question {
        id: question_id,
        category,
        text: question_text,
        format,
        recommended,
        impact: calculate_impact(category),
    })
}

/// Generate a fallback question when Claude is unavailable
pub fn generate_fallback_question(category: Category, question_id: usize) -> Question {
    let (text, format, recommended) = match category {
        Category::FunctionalScope => (
            "What is the primary success criterion for this feature?",
            QuestionFormat::ShortAnswer { max_words: 5 },
            Some(RecommendedAnswer {
                value: "Feature works correctly".to_string(),
                reasoning: "Basic functionality is the minimum bar".to_string(),
            }),
        ),
        Category::DomainDataModel => (
            "What are the core entities/data structures involved?",
            QuestionFormat::ShortAnswer { max_words: 5 },
            None,
        ),
        Category::InteractionUxFlow => (
            "How should errors be presented to users?",
            QuestionFormat::MultipleChoice {
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        description: "Inline error messages".to_string(),
                    },
                    QuestionOption {
                        letter: 'B',
                        description: "Toast notifications".to_string(),
                    },
                    QuestionOption {
                        letter: 'C',
                        description: "Modal dialogs".to_string(),
                    },
                ],
            },
            Some(RecommendedAnswer {
                value: "A".to_string(),
                reasoning: "Inline messages provide immediate context".to_string(),
            }),
        ),
        Category::NonFunctionalQuality => (
            "What is the expected response time for operations?",
            QuestionFormat::MultipleChoice {
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        description: "< 100ms (real-time)".to_string(),
                    },
                    QuestionOption {
                        letter: 'B',
                        description: "< 500ms (interactive)".to_string(),
                    },
                    QuestionOption {
                        letter: 'C',
                        description: "< 2s (batch)".to_string(),
                    },
                ],
            },
            Some(RecommendedAnswer {
                value: "B".to_string(),
                reasoning: "500ms is typical for CLI tools".to_string(),
            }),
        ),
        Category::IntegrationDependencies => (
            "What external services or APIs does this feature depend on?",
            QuestionFormat::ShortAnswer { max_words: 5 },
            Some(RecommendedAnswer {
                value: "None".to_string(),
                reasoning: "Minimize external dependencies when possible".to_string(),
            }),
        ),
        Category::EdgeCasesFailures => (
            "How should the feature handle timeouts?",
            QuestionFormat::MultipleChoice {
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        description: "Retry with backoff".to_string(),
                    },
                    QuestionOption {
                        letter: 'B',
                        description: "Fail immediately".to_string(),
                    },
                    QuestionOption {
                        letter: 'C',
                        description: "Use cached fallback".to_string(),
                    },
                ],
            },
            Some(RecommendedAnswer {
                value: "A".to_string(),
                reasoning: "Retry handles transient failures gracefully".to_string(),
            }),
        ),
        Category::ConstraintsTradeoffs => (
            "Are there any technical constraints to consider?",
            QuestionFormat::ShortAnswer { max_words: 5 },
            Some(RecommendedAnswer {
                value: "No major constraints".to_string(),
                reasoning: "Start simple, add constraints as needed".to_string(),
            }),
        ),
        Category::TerminologyConsistency => (
            "What should we call the main configuration file?",
            QuestionFormat::ShortAnswer { max_words: 5 },
            Some(RecommendedAnswer {
                value: "config.toml".to_string(),
                reasoning: "Matches Rust ecosystem conventions".to_string(),
            }),
        ),
        Category::CompletionSignals => (
            "What indicates the feature is complete?",
            QuestionFormat::MultipleChoice {
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        description: "All tests pass".to_string(),
                    },
                    QuestionOption {
                        letter: 'B',
                        description: "Manual QA approval".to_string(),
                    },
                    QuestionOption {
                        letter: 'C',
                        description: "Documentation complete".to_string(),
                    },
                    QuestionOption {
                        letter: 'D',
                        description: "All of the above".to_string(),
                    },
                ],
            },
            Some(RecommendedAnswer {
                value: "D".to_string(),
                reasoning: "Comprehensive completion criteria ensure quality".to_string(),
            }),
        ),
        Category::MiscPlaceholders => (
            "Are there any unresolved TODOs or placeholders?",
            QuestionFormat::MultipleChoice {
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        description: "Yes, needs resolution".to_string(),
                    },
                    QuestionOption {
                        letter: 'B',
                        description: "No, all resolved".to_string(),
                    },
                ],
            },
            Some(RecommendedAnswer {
                value: "B".to_string(),
                reasoning: "Specs should be complete before implementation".to_string(),
            }),
        ),
    };

    Question {
        id: question_id,
        category,
        text: text.to_string(),
        format,
        recommended,
        impact: calculate_impact(category),
    }
}

/// Generate prioritized questions from analysis
pub async fn generate_questions(
    spec_content: &str,
    analysis: &AnalysisResult,
    config: &ClarifyConfig,
) -> Result<Vec<Question>, ClarifyError> {
    let priorities = prioritize_categories(analysis);
    let max_questions = config.max_questions.min(priorities.len());

    let mut questions = Vec::with_capacity(max_questions);

    for (idx, (category, _priority)) in priorities.iter().take(max_questions).enumerate() {
        let question_id = idx + 1;

        if config.use_claude {
            let prompt = build_question_prompt(*category, spec_content);
            match call_claude_cli(&prompt, config).await {
                Ok(response) => {
                    if let Some(question) = parse_claude_response(&response, *category, question_id)
                    {
                        questions.push(question);
                        continue;
                    }
                }
                Err(e) => {
                    tracing::warn!("Claude CLI failed for question {}: {}", question_id, e);
                }
            }
        }

        // Fallback to template question
        questions.push(generate_fallback_question(*category, question_id));
    }

    Ok(questions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_impact() {
        assert_eq!(calculate_impact(Category::FunctionalScope), 10);
        assert_eq!(calculate_impact(Category::MiscPlaceholders), 3);
    }

    #[test]
    fn test_prioritize_categories() {
        let mut coverage = HashMap::new();
        coverage.insert(Category::FunctionalScope, CoverageStatus::Missing);
        coverage.insert(Category::MiscPlaceholders, CoverageStatus::Partial);

        let analysis = AnalysisResult {
            coverage,
            needs_clarification: vec![Category::FunctionalScope, Category::MiscPlaceholders],
            match_counts: HashMap::new(),
        };

        let priorities = prioritize_categories(&analysis);

        // FunctionalScope should be first (10 * 2 = 20)
        // MiscPlaceholders should be second (3 * 1 = 3)
        assert_eq!(priorities[0].0, Category::FunctionalScope);
    }

    #[test]
    fn test_generate_fallback_question() {
        let question = generate_fallback_question(Category::NonFunctionalQuality, 1);

        assert_eq!(question.id, 1);
        assert_eq!(question.category, Category::NonFunctionalQuality);
        assert!(!question.text.is_empty());
        assert!(question.recommended.is_some());
    }

    #[test]
    fn test_parse_claude_response_valid() {
        let response = r#"Here's a question:
{
  "question": "What is the expected latency?",
  "format": "multiple_choice",
  "options": [
    {"letter": "A", "description": "< 100ms"},
    {"letter": "B", "description": "< 500ms"}
  ],
  "recommended": {"value": "B", "reasoning": "Typical for CLI"}
}"#;

        let question = parse_claude_response(response, Category::NonFunctionalQuality, 1);

        assert!(question.is_some());
        let q = question.unwrap();
        assert_eq!(q.text, "What is the expected latency?");
        assert!(matches!(q.format, QuestionFormat::MultipleChoice { .. }));
    }

    #[test]
    fn test_parse_claude_response_invalid() {
        let response = "This is not valid JSON";
        let question = parse_claude_response(response, Category::FunctionalScope, 1);
        assert!(question.is_none());
    }
}

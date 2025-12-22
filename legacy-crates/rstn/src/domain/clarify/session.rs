//! Q&A session state management
//!
//! This module manages the lifecycle of clarification sessions,
//! tracking questions asked, answers collected, and session completion.

use std::collections::VecDeque;
use std::path::PathBuf;

use super::{
    analyzer, question, AnalysisResult, Answer, ClarifyConfig, ClarifyError, Question,
    QuestionFormat,
};

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

    /// Configuration
    pub config: ClarifyConfig,
}

impl ClarifySession {
    /// Create a new session from analysis results
    pub fn new(
        spec_path: PathBuf,
        content: String,
        analysis: AnalysisResult,
        questions: Vec<Question>,
        config: ClarifyConfig,
    ) -> Self {
        let session_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        Self {
            spec_path,
            original_content: content.clone(),
            current_content: content,
            analysis,
            question_queue: VecDeque::from(questions),
            answers: Vec::new(),
            questions_asked: 0,
            session_date,
            config,
        }
    }

    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        self.questions_asked >= self.config.max_questions || self.question_queue.is_empty()
    }

    /// Get next question (if any)
    pub fn next_question(&mut self) -> Option<Question> {
        if self.is_complete() {
            return None;
        }
        self.question_queue.pop_front()
    }

    /// Get remaining question count
    pub fn remaining_questions(&self) -> usize {
        self.question_queue
            .len()
            .min(self.config.max_questions - self.questions_asked)
    }

    /// Get number of questions answered
    pub fn answered_count(&self) -> usize {
        self.answers.len()
    }
}

/// Validate a multiple choice answer
pub fn validate_multiple_choice(answer: &str, options: &[super::QuestionOption]) -> bool {
    let normalized = answer.trim().to_uppercase();

    // Check if it's a valid option letter
    if normalized.len() == 1 {
        let letter = normalized.chars().next().unwrap();
        if options.iter().any(|opt| opt.letter == letter) {
            return true;
        }
    }

    // Check for "yes", "recommended", "suggested" to accept recommendation
    let accept_keywords = ["yes", "recommended", "suggested", "y"];
    if accept_keywords
        .iter()
        .any(|kw| normalized.eq_ignore_ascii_case(kw))
    {
        return true;
    }

    false
}

/// Validate a short answer
pub fn validate_short_answer(answer: &str, max_words: usize) -> bool {
    let trimmed = answer.trim();

    if trimmed.is_empty() {
        return false;
    }

    let word_count = trimmed.split_whitespace().count();
    word_count <= max_words
}

/// Submit an answer and validate it
pub fn submit_answer(
    session: &mut ClarifySession,
    question: &Question,
    answer_value: String,
) -> Result<Answer, ClarifyError> {
    // Validate based on question format
    let is_valid = match &question.format {
        QuestionFormat::MultipleChoice { options } => {
            validate_multiple_choice(&answer_value, options)
        }
        QuestionFormat::ShortAnswer { max_words } => {
            validate_short_answer(&answer_value, *max_words)
        }
    };

    if !is_valid {
        let err_msg = match &question.format {
            QuestionFormat::MultipleChoice { options } => {
                let valid_letters: String = options.iter().map(|o| o.letter).collect();
                format!(
                    "Invalid answer. Expected one of: {} or 'yes' to accept recommendation",
                    valid_letters
                )
            }
            QuestionFormat::ShortAnswer { max_words } => {
                format!(
                    "Invalid answer. Must be non-empty and at most {} words",
                    max_words
                )
            }
        };
        return Err(ClarifyError::InvalidAnswer(err_msg));
    }

    // Determine if user accepted recommendation
    let accepted_recommendation = if let Some(rec) = &question.recommended {
        let normalized = answer_value.trim().to_uppercase();
        normalized.eq_ignore_ascii_case(&rec.value)
            || ["yes", "recommended", "suggested", "y"]
                .iter()
                .any(|kw| normalized.eq_ignore_ascii_case(kw))
    } else {
        false
    };

    // Resolve actual answer value if user said "yes"
    let final_value = if ["yes", "recommended", "suggested", "y"]
        .iter()
        .any(|kw| answer_value.trim().eq_ignore_ascii_case(kw))
    {
        question
            .recommended
            .as_ref()
            .map(|r| r.value.clone())
            .unwrap_or(answer_value)
    } else {
        answer_value
    };

    let answer = Answer {
        question_id: question.id,
        value: final_value,
        accepted_recommendation,
        question_text: question.text.clone(),
    };

    session.answers.push(answer.clone());
    session.questions_asked += 1;

    Ok(answer)
}

/// Start a new clarification session
pub async fn start_session(
    spec_path: PathBuf,
    config: Option<ClarifyConfig>,
) -> Result<ClarifySession, ClarifyError> {
    let config = config.unwrap_or_default();

    // Load and analyze spec
    let content = analyzer::load_spec_content(&spec_path)?;
    let analysis = analyzer::analyze_spec_content(&content)?;

    // Check if clarification is needed
    if analysis.needs_clarification.is_empty() {
        return Err(ClarifyError::NoClarificationNeeded);
    }

    // Generate questions
    let questions = question::generate_questions(&content, &analysis, &config).await?;

    if questions.is_empty() {
        return Err(ClarifyError::NoClarificationNeeded);
    }

    Ok(ClarifySession::new(
        spec_path, content, analysis, questions, config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_multiple_choice_valid_letter() {
        let options = vec![
            super::super::QuestionOption {
                letter: 'A',
                description: "Option A".to_string(),
            },
            super::super::QuestionOption {
                letter: 'B',
                description: "Option B".to_string(),
            },
        ];

        assert!(validate_multiple_choice("A", &options));
        assert!(validate_multiple_choice("a", &options));
        assert!(validate_multiple_choice("B", &options));
        assert!(validate_multiple_choice(" A ", &options));
    }

    #[test]
    fn test_validate_multiple_choice_accept_keywords() {
        let options = vec![super::super::QuestionOption {
            letter: 'A',
            description: "Option A".to_string(),
        }];

        assert!(validate_multiple_choice("yes", &options));
        assert!(validate_multiple_choice("YES", &options));
        assert!(validate_multiple_choice("recommended", &options));
        assert!(validate_multiple_choice("y", &options));
    }

    #[test]
    fn test_validate_multiple_choice_invalid() {
        let options = vec![super::super::QuestionOption {
            letter: 'A',
            description: "Option A".to_string(),
        }];

        assert!(!validate_multiple_choice("C", &options));
        assert!(!validate_multiple_choice("invalid", &options));
        assert!(!validate_multiple_choice("", &options));
    }

    #[test]
    fn test_validate_short_answer_valid() {
        assert!(validate_short_answer("One word", 5));
        assert!(validate_short_answer("exactly five words here now", 5));
        assert!(validate_short_answer("a", 5));
    }

    #[test]
    fn test_validate_short_answer_invalid() {
        assert!(!validate_short_answer("", 5));
        assert!(!validate_short_answer("   ", 5));
        assert!(!validate_short_answer("one two three four five six", 5));
    }

    #[test]
    fn test_session_is_complete() {
        use std::collections::HashMap;

        let analysis = AnalysisResult {
            coverage: HashMap::new(),
            needs_clarification: vec![],
            match_counts: HashMap::new(),
        };

        let mut session = ClarifySession::new(
            PathBuf::from("test.md"),
            "content".to_string(),
            analysis,
            vec![],
            ClarifyConfig::default(),
        );

        // Empty queue means complete
        assert!(session.is_complete());

        // Add questions and check
        session.question_queue.push_back(Question {
            id: 1,
            category: super::super::Category::FunctionalScope,
            text: "Test?".to_string(),
            format: QuestionFormat::ShortAnswer { max_words: 5 },
            recommended: None,
            impact: 10,
        });

        assert!(!session.is_complete());

        // Ask max questions
        session.questions_asked = 5;
        assert!(session.is_complete());
    }

    #[test]
    fn test_session_next_question() {
        use std::collections::HashMap;

        let analysis = AnalysisResult {
            coverage: HashMap::new(),
            needs_clarification: vec![],
            match_counts: HashMap::new(),
        };

        let questions = vec![
            Question {
                id: 1,
                category: super::super::Category::FunctionalScope,
                text: "Q1?".to_string(),
                format: QuestionFormat::ShortAnswer { max_words: 5 },
                recommended: None,
                impact: 10,
            },
            Question {
                id: 2,
                category: super::super::Category::DomainDataModel,
                text: "Q2?".to_string(),
                format: QuestionFormat::ShortAnswer { max_words: 5 },
                recommended: None,
                impact: 9,
            },
        ];

        let mut session = ClarifySession::new(
            PathBuf::from("test.md"),
            "content".to_string(),
            analysis,
            questions,
            ClarifyConfig::default(),
        );

        let q1 = session.next_question();
        assert!(q1.is_some());
        assert_eq!(q1.unwrap().id, 1);

        let q2 = session.next_question();
        assert!(q2.is_some());
        assert_eq!(q2.unwrap().id, 2);

        let q3 = session.next_question();
        assert!(q3.is_none());
    }
}

"""Question generation for spec clarification.

Pure functions for generating and prioritizing questions.
"""

from __future__ import annotations

from rstn.domain.clarify.types import Category, CoverageStatus, Question, SpecCoverage

# Question templates for each category
QUESTION_TEMPLATES: dict[Category, list[str]] = {
    Category.OVERVIEW: [
        "What is the main problem this feature solves?",
        "Who are the primary users of this feature?",
    ],
    Category.GOALS: [
        "What are the specific measurable goals for this feature?",
        "How will success be measured?",
    ],
    Category.USER_STORIES: [
        "What are the key user workflows?",
        "Are there different user types with different needs?",
    ],
    Category.TECHNICAL_REQUIREMENTS: [
        "What are the hard technical constraints?",
        "What dependencies exist?",
    ],
    Category.ACCEPTANCE_CRITERIA: [
        "What specific conditions must be met for acceptance?",
        "How will the feature be validated?",
    ],
    Category.EDGE_CASES: [
        "What happens with empty or null inputs?",
        "What are the boundary conditions?",
    ],
    Category.ERROR_HANDLING: [
        "How should errors be communicated to users?",
        "What recovery options should be available?",
    ],
    Category.TESTING: [
        "What types of tests are required?",
        "What is the minimum test coverage?",
    ],
    Category.SECURITY: [
        "What authentication/authorization is needed?",
        "What data needs to be protected?",
    ],
    Category.PERFORMANCE: [
        "What are the latency requirements?",
        "What is the expected load/scale?",
    ],
}


def generate_questions(
    coverages: list[SpecCoverage],
    max_questions: int = 10,
) -> list[Question]:
    """Generate clarifying questions based on coverage analysis.

    Pure function - no I/O.

    Args:
        coverages: Coverage analysis results
        max_questions: Maximum number of questions to generate

    Returns:
        List of generated questions
    """
    questions: list[Question] = []
    question_id = 1

    # Prioritize missing categories, then partial
    sorted_coverages = sorted(
        coverages,
        key=lambda c: (0 if c.status == CoverageStatus.MISSING else 1, c.category.value),
    )

    for coverage in sorted_coverages:
        if len(questions) >= max_questions:
            break

        # Only generate questions for non-covered categories
        if coverage.status == CoverageStatus.COVERED:
            continue

        templates = QUESTION_TEMPLATES.get(coverage.category, [])
        priority = 5 if coverage.status == CoverageStatus.MISSING else 3

        for template in templates[:2]:  # Max 2 questions per category
            if len(questions) >= max_questions:
                break

            questions.append(
                Question(
                    id=question_id,
                    category=coverage.category,
                    text=template,
                    priority=priority,
                    context=coverage.notes,
                )
            )
            question_id += 1

    return questions


def prioritize_questions(questions: list[Question]) -> list[Question]:
    """Sort questions by priority.

    Pure function - no I/O.

    Args:
        questions: List of questions

    Returns:
        Questions sorted by priority (highest first)
    """
    return sorted(questions, key=lambda q: (-q.priority, q.id))


def filter_by_category(
    questions: list[Question],
    category: Category,
) -> list[Question]:
    """Filter questions by category.

    Pure function - no I/O.

    Args:
        questions: List of questions
        category: Category to filter by

    Returns:
        Filtered questions
    """
    return [q for q in questions if q.category == category]


def get_next_question(
    questions: list[Question],
    answered_ids: set[int],
) -> Question | None:
    """Get the next unanswered question.

    Pure function - no I/O.

    Args:
        questions: List of questions
        answered_ids: Set of already answered question IDs

    Returns:
        Next question or None if all answered
    """
    for question in prioritize_questions(questions):
        if question.id not in answered_ids:
            return question
    return None

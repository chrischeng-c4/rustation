"""Answer integration into spec.

Pure functions for integrating answers.
Effect creators for spec updates.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.clarify.session import ClarifySession
from rstn.domain.clarify.types import Answer, Category, Question
from rstn.effect import AppEffect, WriteFile


def integrate_answers(
    spec_content: str,
    session: ClarifySession,
) -> str:
    """Integrate answers into spec content.

    Pure function - no I/O.

    Args:
        spec_content: Original spec content
        session: Completed clarify session

    Returns:
        Updated spec content
    """
    if not session.answers:
        return spec_content

    # Group answers by category
    answers_by_category: dict[Category, list[tuple[Question, Answer]]] = {}
    for answer in session.answers:
        if not answer.applies_to_spec:
            continue

        # Find matching question
        question = next(
            (q for q in session.questions if q.id == answer.question_id),
            None,
        )
        if question:
            if question.category not in answers_by_category:
                answers_by_category[question.category] = []
            answers_by_category[question.category].append((question, answer))

    # Build clarifications section
    clarifications = _build_clarifications_section(answers_by_category)

    # Append to spec
    return f"{spec_content}\n\n{clarifications}"


def _build_clarifications_section(
    answers_by_category: dict[Category, list[tuple[Question, Answer]]],
) -> str:
    """Build clarifications section content.

    Pure function - no I/O.
    """
    if not answers_by_category:
        return ""

    lines = ["## Clarifications", ""]

    for category, qa_pairs in sorted(answers_by_category.items(), key=lambda x: x[0].value):
        lines.append(f"### {category.value.replace('_', ' ').title()}")
        lines.append("")

        for question, answer in qa_pairs:
            lines.append(f"**Q:** {question.text}")
            lines.append(f"**A:** {answer.text}")
            lines.append("")

    return "\n".join(lines)


def create_spec_update_effects(
    spec_path: Path,
    updated_content: str,
) -> list[AppEffect]:
    """Create effects to update spec file.

    Effect creator - returns effects, doesn't execute.

    Args:
        spec_path: Path to spec file
        updated_content: Updated spec content

    Returns:
        List of effects to execute
    """
    return [
        WriteFile(
            path=spec_path,
            contents=updated_content,
        )
    ]


def create_clarify_summary(session: ClarifySession) -> str:
    """Create summary of clarification session.

    Pure function - no I/O.

    Args:
        session: Completed session

    Returns:
        Summary text
    """
    answered, total = session.progress
    applied = sum(1 for a in session.answers if a.applies_to_spec)

    lines = [
        "# Clarification Summary",
        "",
        f"- Questions answered: {answered}/{total}",
        f"- Answers applied to spec: {applied}",
        "",
        "## Questions & Answers",
        "",
    ]

    for answer in session.answers:
        question = next(
            (q for q in session.questions if q.id == answer.question_id),
            None,
        )
        if question:
            lines.append(f"### {question.category.value.replace('_', ' ').title()}")
            lines.append(f"**Q:** {question.text}")
            lines.append(f"**A:** {answer.text}")
            if not answer.applies_to_spec:
                lines.append("*(Not applied to spec)*")
            lines.append("")

    return "\n".join(lines)

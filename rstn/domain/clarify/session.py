"""Clarify session management.

Manages the state of a clarification session.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, Field

from rstn.domain.clarify.types import Answer, Question


class ClarifySession(BaseModel):
    """State of a clarification session."""

    model_config = {"frozen": True}

    spec_path: Path = Field(description="Path to spec being clarified")
    questions: list[Question] = Field(
        default_factory=list, description="Generated questions"
    )
    answers: list[Answer] = Field(default_factory=list, description="Recorded answers")
    current_question_index: int = Field(default=0, description="Current question index")

    @property
    def is_complete(self) -> bool:
        """Check if all questions have been answered."""
        return len(self.answers) >= len(self.questions)

    @property
    def progress(self) -> tuple[int, int]:
        """Get progress as (answered, total)."""
        return (len(self.answers), len(self.questions))

    @property
    def current_question(self) -> Question | None:
        """Get the current question."""
        if self.current_question_index < len(self.questions):
            return self.questions[self.current_question_index]
        return None

    @property
    def remaining_count(self) -> int:
        """Get count of remaining questions."""
        return max(0, len(self.questions) - len(self.answers))

    def get_answered_ids(self) -> set[int]:
        """Get set of answered question IDs."""
        return {a.question_id for a in self.answers}


def create_session(
    spec_path: Path,
    questions: list[Question],
) -> ClarifySession:
    """Create a new clarify session.

    Pure function - no I/O.

    Args:
        spec_path: Path to spec file
        questions: Generated questions

    Returns:
        New session
    """
    return ClarifySession(
        spec_path=spec_path,
        questions=questions,
        answers=[],
        current_question_index=0,
    )


def record_answer(
    session: ClarifySession,
    answer: Answer,
) -> ClarifySession:
    """Record an answer in the session.

    Pure function - no I/O.

    Args:
        session: Current session
        answer: Answer to record

    Returns:
        Updated session
    """
    new_answers = [*session.answers, answer]
    new_index = session.current_question_index + 1

    return ClarifySession(
        spec_path=session.spec_path,
        questions=session.questions,
        answers=new_answers,
        current_question_index=new_index,
    )


def skip_question(session: ClarifySession) -> ClarifySession:
    """Skip the current question.

    Pure function - no I/O.

    Args:
        session: Current session

    Returns:
        Updated session with incremented index
    """
    return ClarifySession(
        spec_path=session.spec_path,
        questions=session.questions,
        answers=session.answers,
        current_question_index=session.current_question_index + 1,
    )

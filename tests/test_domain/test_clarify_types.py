"""Tests for clarify domain types."""

from __future__ import annotations

import pytest
from rstn.domain.clarify.types import (
    Answer,
    Category,
    CoverageStatus,
    Question,
    SpecCoverage,
)


class TestCategory:
    """Tests for Category enum."""

    def test_category_values(self) -> None:
        """Test all category values."""
        assert Category.OVERVIEW.value == "overview"
        assert Category.GOALS.value == "goals"
        assert Category.USER_STORIES.value == "user_stories"
        assert Category.TECHNICAL_REQUIREMENTS.value == "technical_requirements"
        assert Category.ACCEPTANCE_CRITERIA.value == "acceptance_criteria"
        assert Category.EDGE_CASES.value == "edge_cases"
        assert Category.ERROR_HANDLING.value == "error_handling"
        assert Category.TESTING.value == "testing"
        assert Category.SECURITY.value == "security"
        assert Category.PERFORMANCE.value == "performance"

    def test_category_is_string_enum(self) -> None:
        """Test Category is a string enum."""
        for category in Category:
            assert isinstance(category.value, str)


class TestCoverageStatus:
    """Tests for CoverageStatus enum."""

    def test_coverage_status_values(self) -> None:
        """Test all coverage status values."""
        assert CoverageStatus.COVERED.value == "covered"
        assert CoverageStatus.PARTIAL.value == "partial"
        assert CoverageStatus.MISSING.value == "missing"

    def test_coverage_status_is_string_enum(self) -> None:
        """Test CoverageStatus is a string enum."""
        for status in CoverageStatus:
            assert isinstance(status.value, str)


class TestSpecCoverage:
    """Tests for SpecCoverage model."""

    def test_coverage_creation(self) -> None:
        """Test creating spec coverage."""
        coverage = SpecCoverage(
            category=Category.OVERVIEW,
            status=CoverageStatus.COVERED,
            notes="Well documented overview section",
        )
        assert coverage.category == Category.OVERVIEW
        assert coverage.status == CoverageStatus.COVERED
        assert coverage.notes == "Well documented overview section"
        assert coverage.excerpts == []

    def test_coverage_with_excerpts(self) -> None:
        """Test coverage with excerpts."""
        coverage = SpecCoverage(
            category=Category.GOALS,
            status=CoverageStatus.PARTIAL,
            notes="Some goals defined",
            excerpts=["Goal 1: ...", "Goal 2: ..."],
        )
        assert len(coverage.excerpts) == 2

    def test_coverage_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        coverage = SpecCoverage(
            category=Category.SECURITY,
            status=CoverageStatus.MISSING,
            notes="No security section",
        )
        json_str = coverage.model_dump_json()
        restored = SpecCoverage.model_validate_json(json_str)
        assert restored == coverage

    def test_coverage_immutable(self) -> None:
        """Test coverage is immutable (frozen)."""
        coverage = SpecCoverage(
            category=Category.TESTING,
            status=CoverageStatus.COVERED,
        )
        with pytest.raises(Exception):
            coverage.status = CoverageStatus.MISSING  # type: ignore


class TestQuestion:
    """Tests for Question model."""

    def test_question_creation(self) -> None:
        """Test creating question."""
        question = Question(
            id=1,
            category=Category.USER_STORIES,
            text="Who are the target users?",
        )
        assert question.id == 1
        assert question.category == Category.USER_STORIES
        assert question.text == "Who are the target users?"
        assert question.priority == 1  # Default
        assert question.context == ""  # Default

    def test_question_with_priority(self) -> None:
        """Test question with priority."""
        question = Question(
            id=2,
            category=Category.ACCEPTANCE_CRITERIA,
            text="What are the success criteria?",
            priority=5,  # Highest priority
            context="The spec mentions 'must work' but no criteria",
        )
        assert question.priority == 5
        assert question.context == "The spec mentions 'must work' but no criteria"

    def test_question_priority_bounds(self) -> None:
        """Test priority must be 1-5."""
        # Valid priorities
        for priority in [1, 2, 3, 4, 5]:
            q = Question(
                id=1,
                category=Category.GOALS,
                text="Test",
                priority=priority,
            )
            assert q.priority == priority

        # Invalid priorities
        with pytest.raises(Exception):
            Question(
                id=1,
                category=Category.GOALS,
                text="Test",
                priority=0,
            )

        with pytest.raises(Exception):
            Question(
                id=1,
                category=Category.GOALS,
                text="Test",
                priority=6,
            )

    def test_question_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        question = Question(
            id=3,
            category=Category.EDGE_CASES,
            text="What happens when X fails?",
            priority=3,
        )
        json_str = question.model_dump_json()
        restored = Question.model_validate_json(json_str)
        assert restored == question

    def test_question_immutable(self) -> None:
        """Test question is immutable (frozen)."""
        question = Question(
            id=1,
            category=Category.OVERVIEW,
            text="Test",
        )
        with pytest.raises(Exception):
            question.text = "New text"  # type: ignore


class TestAnswer:
    """Tests for Answer model."""

    def test_answer_creation(self) -> None:
        """Test creating answer."""
        answer = Answer(
            question_id=1,
            text="The target users are developers.",
        )
        assert answer.question_id == 1
        assert answer.text == "The target users are developers."
        assert answer.applies_to_spec is True  # Default

    def test_answer_not_applicable(self) -> None:
        """Test answer that doesn't apply to spec."""
        answer = Answer(
            question_id=2,
            text="This is an implementation detail.",
            applies_to_spec=False,
        )
        assert answer.applies_to_spec is False

    def test_answer_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        answer = Answer(
            question_id=5,
            text="The answer is 42.",
            applies_to_spec=True,
        )
        json_str = answer.model_dump_json()
        restored = Answer.model_validate_json(json_str)
        assert restored == answer

    def test_answer_immutable(self) -> None:
        """Test answer is immutable (frozen)."""
        answer = Answer(
            question_id=1,
            text="Test",
        )
        with pytest.raises(Exception):
            answer.text = "New text"  # type: ignore


class TestClarifyIntegration:
    """Integration tests for clarify types."""

    def test_coverage_analysis_workflow(self) -> None:
        """Test coverage analysis workflow."""
        coverages = [
            SpecCoverage(
                category=Category.OVERVIEW,
                status=CoverageStatus.COVERED,
                notes="Good overview",
            ),
            SpecCoverage(
                category=Category.GOALS,
                status=CoverageStatus.PARTIAL,
                notes="Some goals missing",
            ),
            SpecCoverage(
                category=Category.SECURITY,
                status=CoverageStatus.MISSING,
                notes="No security section",
            ),
        ]

        covered = [c for c in coverages if c.status == CoverageStatus.COVERED]
        missing = [c for c in coverages if c.status == CoverageStatus.MISSING]

        assert len(covered) == 1
        assert len(missing) == 1

    def test_qa_workflow(self) -> None:
        """Test question-answer workflow."""
        questions = [
            Question(
                id=1,
                category=Category.USER_STORIES,
                text="Who are the users?",
                priority=5,
            ),
            Question(
                id=2,
                category=Category.ACCEPTANCE_CRITERIA,
                text="What are success criteria?",
                priority=4,
            ),
        ]

        answers = [
            Answer(
                question_id=1,
                text="Developers and DevOps engineers.",
            ),
        ]

        # Find unanswered questions
        answered_ids = {a.question_id for a in answers}
        unanswered = [q for q in questions if q.id not in answered_ids]

        assert len(unanswered) == 1
        assert unanswered[0].id == 2

    def test_all_categories(self) -> None:
        """Test all categories can be used."""
        for category in Category:
            coverage = SpecCoverage(
                category=category,
                status=CoverageStatus.COVERED,
            )
            question = Question(
                id=1,
                category=category,
                text=f"Question about {category.value}",
            )
            assert coverage.category == category
            assert question.category == category

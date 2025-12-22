"""Clarify domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class Category(str, Enum):
    """Categories for spec coverage analysis."""

    OVERVIEW = "overview"
    GOALS = "goals"
    USER_STORIES = "user_stories"
    TECHNICAL_REQUIREMENTS = "technical_requirements"
    ACCEPTANCE_CRITERIA = "acceptance_criteria"
    EDGE_CASES = "edge_cases"
    ERROR_HANDLING = "error_handling"
    TESTING = "testing"
    SECURITY = "security"
    PERFORMANCE = "performance"


class CoverageStatus(str, Enum):
    """Status of category coverage."""

    COVERED = "covered"
    PARTIAL = "partial"
    MISSING = "missing"


class SpecCoverage(BaseModel):
    """Coverage analysis for a spec category."""

    model_config = {"frozen": True}

    category: Category = Field(description="Category analyzed")
    status: CoverageStatus = Field(description="Coverage status")
    notes: str = Field(default="", description="Analysis notes")
    excerpts: list[str] = Field(
        default_factory=list, description="Relevant excerpts from spec"
    )


class Question(BaseModel):
    """A clarifying question."""

    model_config = {"frozen": True}

    id: int = Field(description="Question ID")
    category: Category = Field(description="Question category")
    text: str = Field(description="Question text")
    priority: int = Field(default=1, ge=1, le=5, description="Priority 1-5 (5=highest)")
    context: str = Field(default="", description="Context for the question")


class Answer(BaseModel):
    """An answer to a clarifying question."""

    model_config = {"frozen": True}

    question_id: int = Field(description="ID of answered question")
    text: str = Field(description="Answer text")
    applies_to_spec: bool = Field(
        default=True, description="Whether answer should update spec"
    )

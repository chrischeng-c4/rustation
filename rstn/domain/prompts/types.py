"""Prompts domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field


class SpecPhase(str, Enum):
    """Phase of the spec-driven development workflow."""

    SPECIFY = "specify"  # Initial spec generation
    CLARIFY = "clarify"  # Clarification questions
    PLAN = "plan"  # Implementation planning
    IMPLEMENT = "implement"  # Code implementation
    REVIEW = "review"  # Code review


class PromptContext(BaseModel):
    """Context for prompt generation."""

    model_config = {"frozen": True}

    phase: SpecPhase = Field(description="Current workflow phase")
    project_root: Path = Field(description="Project root directory")
    feature_name: str | None = Field(default=None, description="Feature name if applicable")
    feature_number: str | None = Field(default=None, description="Feature number if applicable")
    spec_path: Path | None = Field(default=None, description="Path to spec file if exists")
    plan_path: Path | None = Field(default=None, description="Path to plan file if exists")
    additional_context: dict[str, str] = Field(
        default_factory=dict, description="Additional context key-value pairs"
    )

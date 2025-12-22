"""Plan domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field


class ArtifactKind(str, Enum):
    """Kind of plan artifact."""

    PLAN = "plan"  # Main plan document
    TASKS = "tasks"  # Task breakdown
    ARCHITECTURE = "architecture"  # Architecture diagram
    SEQUENCE = "sequence"  # Sequence diagram
    STATE = "state"  # State machine diagram


class PlanConfig(BaseModel):
    """Configuration for plan generation."""

    model_config = {"frozen": True}

    project_root: Path = Field(description="Project root directory")
    spec_path: Path = Field(description="Path to spec file")
    output_dir: Path = Field(description="Output directory for plan artifacts")
    include_diagrams: bool = Field(default=True, description="Whether to include diagrams")
    max_tasks: int = Field(default=20, description="Maximum task items")


class PlanContext(BaseModel):
    """Context for plan generation."""

    model_config = {"frozen": True}

    spec_content: str = Field(description="Spec document content")
    clarify_content: str | None = Field(default=None, description="Clarification content if any")
    existing_plan: str | None = Field(default=None, description="Existing plan if any")
    codebase_context: str = Field(default="", description="Relevant codebase context")


class PlanArtifact(BaseModel):
    """A generated plan artifact."""

    model_config = {"frozen": True}

    kind: ArtifactKind = Field(description="Artifact kind")
    path: Path = Field(description="Output path")
    content: str = Field(description="Artifact content")


class PlanResult(BaseModel):
    """Result of plan generation."""

    model_config = {"frozen": True}

    success: bool = Field(description="Whether generation succeeded")
    plan_path: Path | None = Field(default=None, description="Path to main plan")
    artifacts: list[PlanArtifact] = Field(
        default_factory=list, description="Generated artifacts"
    )
    error: str | None = Field(default=None, description="Error message if failed")

"""Plan domain operations for rstn.

Provides implementation plan generation including:
- Plan configuration
- Context loading
- Plan generation
- Artifact writing

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.plan.context import (
    build_plan_context,
    create_context_load_effects,
)
from rstn.domain.plan.generator import (
    create_plan_generation_effects,
    generate_plan_content,
)
from rstn.domain.plan.types import (
    ArtifactKind,
    PlanArtifact,
    PlanConfig,
    PlanContext,
    PlanResult,
)
from rstn.domain.plan.writer import (
    create_artifact_write_effects,
    create_plan_write_effects,
)

__all__ = [
    # Types
    "ArtifactKind",
    "PlanArtifact",
    "PlanConfig",
    "PlanContext",
    "PlanResult",
    # Context functions
    "build_plan_context",
    "create_context_load_effects",
    # Generator functions
    "create_plan_generation_effects",
    "generate_plan_content",
    # Writer functions
    "create_artifact_write_effects",
    "create_plan_write_effects",
]

"""Clarify domain operations for rstn.

Provides spec clarification workflow including:
- Spec coverage analysis
- Question generation
- Session management
- Answer integration

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.clarify.analyzer import (
    analyze_spec_coverage,
    get_coverage_summary,
)
from rstn.domain.clarify.integrator import (
    create_spec_update_effects,
    integrate_answers,
)
from rstn.domain.clarify.question import (
    generate_questions,
    prioritize_questions,
)
from rstn.domain.clarify.session import (
    ClarifySession,
    create_session,
    record_answer,
)
from rstn.domain.clarify.types import (
    Answer,
    Category,
    CoverageStatus,
    Question,
    SpecCoverage,
)

__all__ = [
    # Types
    "Answer",
    "Category",
    "CoverageStatus",
    "Question",
    "SpecCoverage",
    # Analyzer functions
    "analyze_spec_coverage",
    "get_coverage_summary",
    # Question functions
    "generate_questions",
    "prioritize_questions",
    # Session functions
    "ClarifySession",
    "create_session",
    "record_answer",
    # Integrator functions
    "integrate_answers",
    "create_spec_update_effects",
]

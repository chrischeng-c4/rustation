"""Tests for prompts domain types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.domain.prompts.types import (
    PromptContext,
    SpecPhase,
)


class TestSpecPhase:
    """Tests for SpecPhase enum."""

    def test_phase_values(self) -> None:
        """Test all phase values."""
        assert SpecPhase.SPECIFY.value == "specify"
        assert SpecPhase.CLARIFY.value == "clarify"
        assert SpecPhase.PLAN.value == "plan"
        assert SpecPhase.IMPLEMENT.value == "implement"
        assert SpecPhase.REVIEW.value == "review"

    def test_phase_is_string_enum(self) -> None:
        """Test SpecPhase is a string enum."""
        for phase in SpecPhase:
            assert isinstance(phase.value, str)

    def test_all_phases_exist(self) -> None:
        """Test all expected phases exist."""
        phases = list(SpecPhase)
        assert len(phases) == 5
        assert SpecPhase.SPECIFY in phases
        assert SpecPhase.CLARIFY in phases
        assert SpecPhase.PLAN in phases
        assert SpecPhase.IMPLEMENT in phases
        assert SpecPhase.REVIEW in phases


class TestPromptContext:
    """Tests for PromptContext model."""

    def test_context_minimal(self) -> None:
        """Test minimal prompt context."""
        context = PromptContext(
            phase=SpecPhase.SPECIFY,
            project_root=Path("/project"),
        )
        assert context.phase == SpecPhase.SPECIFY
        assert context.project_root == Path("/project")
        assert context.feature_name is None
        assert context.feature_number is None
        assert context.spec_path is None
        assert context.plan_path is None
        assert context.additional_context == {}

    def test_context_full(self) -> None:
        """Test full prompt context."""
        context = PromptContext(
            phase=SpecPhase.PLAN,
            project_root=Path("/project"),
            feature_name="worktree-management",
            feature_number="042",
            spec_path=Path("/project/specs/042-worktree-management/spec.md"),
            plan_path=Path("/project/specs/042-worktree-management/plan.md"),
            additional_context={"key1": "value1", "key2": "value2"},
        )
        assert context.phase == SpecPhase.PLAN
        assert context.feature_name == "worktree-management"
        assert context.feature_number == "042"
        assert context.spec_path == Path("/project/specs/042-worktree-management/spec.md")
        assert context.plan_path == Path("/project/specs/042-worktree-management/plan.md")
        assert context.additional_context == {"key1": "value1", "key2": "value2"}

    def test_context_different_phases(self) -> None:
        """Test context with different phases."""
        for phase in SpecPhase:
            context = PromptContext(
                phase=phase,
                project_root=Path("/project"),
            )
            assert context.phase == phase

    def test_context_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        context = PromptContext(
            phase=SpecPhase.IMPLEMENT,
            project_root=Path("/project"),
            feature_name="test-feature",
            additional_context={"codebase": "src/**/*.rs"},
        )
        data = context.model_dump(mode="json")
        restored = PromptContext.model_validate(data)
        assert restored.phase == context.phase
        assert restored.feature_name == context.feature_name

    def test_context_immutable(self) -> None:
        """Test context is immutable (frozen)."""
        context = PromptContext(
            phase=SpecPhase.SPECIFY,
            project_root=Path("/project"),
        )
        with pytest.raises(Exception):
            context.phase = SpecPhase.PLAN  # type: ignore


class TestPromptContextWorkflow:
    """Workflow tests for PromptContext."""

    def test_workflow_progression(self) -> None:
        """Test context progression through workflow phases."""
        project = Path("/project")

        # Phase 1: Specify
        specify_ctx = PromptContext(
            phase=SpecPhase.SPECIFY,
            project_root=project,
        )
        assert specify_ctx.spec_path is None

        # Phase 2: Clarify (spec exists)
        clarify_ctx = PromptContext(
            phase=SpecPhase.CLARIFY,
            project_root=project,
            feature_name="my-feature",
            feature_number="001",
            spec_path=Path(project / "specs/001-my-feature/spec.md"),
        )
        assert clarify_ctx.spec_path is not None
        assert clarify_ctx.plan_path is None

        # Phase 3: Plan (spec exists)
        plan_ctx = PromptContext(
            phase=SpecPhase.PLAN,
            project_root=project,
            feature_name="my-feature",
            feature_number="001",
            spec_path=Path(project / "specs/001-my-feature/spec.md"),
        )
        assert plan_ctx.phase == SpecPhase.PLAN

        # Phase 4: Implement (spec and plan exist)
        impl_ctx = PromptContext(
            phase=SpecPhase.IMPLEMENT,
            project_root=project,
            feature_name="my-feature",
            feature_number="001",
            spec_path=Path(project / "specs/001-my-feature/spec.md"),
            plan_path=Path(project / "specs/001-my-feature/plan.md"),
        )
        assert impl_ctx.spec_path is not None
        assert impl_ctx.plan_path is not None

        # Phase 5: Review
        review_ctx = PromptContext(
            phase=SpecPhase.REVIEW,
            project_root=project,
            feature_name="my-feature",
            additional_context={"pr_number": "42"},
        )
        assert review_ctx.phase == SpecPhase.REVIEW
        assert review_ctx.additional_context["pr_number"] == "42"

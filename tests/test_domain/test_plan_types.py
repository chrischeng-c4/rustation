"""Tests for plan domain types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.domain.plan.types import (
    ArtifactKind,
    PlanArtifact,
    PlanConfig,
    PlanContext,
    PlanResult,
)


class TestArtifactKind:
    """Tests for ArtifactKind enum."""

    def test_artifact_kind_values(self) -> None:
        """Test all artifact kind values."""
        assert ArtifactKind.PLAN.value == "plan"
        assert ArtifactKind.TASKS.value == "tasks"
        assert ArtifactKind.ARCHITECTURE.value == "architecture"
        assert ArtifactKind.SEQUENCE.value == "sequence"
        assert ArtifactKind.STATE.value == "state"

    def test_artifact_kind_is_string_enum(self) -> None:
        """Test ArtifactKind is a string enum."""
        for kind in ArtifactKind:
            assert isinstance(kind.value, str)


class TestPlanConfig:
    """Tests for PlanConfig model."""

    def test_plan_config_creation(self) -> None:
        """Test creating plan config."""
        config = PlanConfig(
            project_root=Path("/project"),
            spec_path=Path("/project/specs/001-feature/spec.md"),
            output_dir=Path("/project/specs/001-feature"),
        )
        assert config.project_root == Path("/project")
        assert config.spec_path == Path("/project/specs/001-feature/spec.md")
        assert config.output_dir == Path("/project/specs/001-feature")
        assert config.include_diagrams is True  # Default
        assert config.max_tasks == 20  # Default

    def test_plan_config_custom_options(self) -> None:
        """Test config with custom options."""
        config = PlanConfig(
            project_root=Path("/project"),
            spec_path=Path("spec.md"),
            output_dir=Path("output"),
            include_diagrams=False,
            max_tasks=50,
        )
        assert config.include_diagrams is False
        assert config.max_tasks == 50

    def test_plan_config_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        config = PlanConfig(
            project_root=Path("/project"),
            spec_path=Path("spec.md"),
            output_dir=Path("output"),
            max_tasks=10,
        )
        data = config.model_dump(mode="json")
        restored = PlanConfig.model_validate(data)
        assert restored.max_tasks == config.max_tasks

    def test_plan_config_immutable(self) -> None:
        """Test config is immutable (frozen)."""
        config = PlanConfig(
            project_root=Path("/project"),
            spec_path=Path("spec.md"),
            output_dir=Path("output"),
        )
        with pytest.raises(Exception):
            config.max_tasks = 100  # type: ignore


class TestPlanContext:
    """Tests for PlanContext model."""

    def test_plan_context_minimal(self) -> None:
        """Test minimal plan context."""
        context = PlanContext(spec_content="# Spec Content")
        assert context.spec_content == "# Spec Content"
        assert context.clarify_content is None
        assert context.existing_plan is None
        assert context.codebase_context == ""

    def test_plan_context_full(self) -> None:
        """Test full plan context."""
        context = PlanContext(
            spec_content="# Spec",
            clarify_content="Q1: ...",
            existing_plan="# Old Plan",
            codebase_context="src/main.rs: ...",
        )
        assert context.spec_content == "# Spec"
        assert context.clarify_content == "Q1: ..."
        assert context.existing_plan == "# Old Plan"
        assert context.codebase_context == "src/main.rs: ..."

    def test_plan_context_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        context = PlanContext(
            spec_content="# Spec",
            clarify_content="Clarifications",
        )
        json_str = context.model_dump_json()
        restored = PlanContext.model_validate_json(json_str)
        assert restored == context


class TestPlanArtifact:
    """Tests for PlanArtifact model."""

    def test_plan_artifact_creation(self) -> None:
        """Test creating plan artifact."""
        artifact = PlanArtifact(
            kind=ArtifactKind.PLAN,
            path=Path("specs/001-feature/plan.md"),
            content="# Implementation Plan\n...",
        )
        assert artifact.kind == ArtifactKind.PLAN
        assert artifact.path == Path("specs/001-feature/plan.md")
        assert "Implementation Plan" in artifact.content

    def test_plan_artifact_different_kinds(self) -> None:
        """Test different artifact kinds."""
        artifacts = [
            PlanArtifact(kind=ArtifactKind.PLAN, path=Path("plan.md"), content="# Plan"),
            PlanArtifact(kind=ArtifactKind.TASKS, path=Path("tasks.md"), content="# Tasks"),
            PlanArtifact(
                kind=ArtifactKind.ARCHITECTURE, path=Path("arch.md"), content="```mermaid\n..."
            ),
        ]
        assert len(artifacts) == 3
        assert artifacts[0].kind == ArtifactKind.PLAN
        assert artifacts[1].kind == ArtifactKind.TASKS
        assert artifacts[2].kind == ArtifactKind.ARCHITECTURE

    def test_plan_artifact_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        artifact = PlanArtifact(
            kind=ArtifactKind.SEQUENCE,
            path=Path("sequence.md"),
            content="# Sequence Diagram",
        )
        data = artifact.model_dump(mode="json")
        restored = PlanArtifact.model_validate(data)
        assert restored.kind == artifact.kind

    def test_plan_artifact_immutable(self) -> None:
        """Test artifact is immutable (frozen)."""
        artifact = PlanArtifact(
            kind=ArtifactKind.STATE,
            path=Path("state.md"),
            content="# State",
        )
        with pytest.raises(Exception):
            artifact.content = "New content"  # type: ignore


class TestPlanResult:
    """Tests for PlanResult model."""

    def test_plan_result_success(self) -> None:
        """Test successful plan result."""
        artifacts = [
            PlanArtifact(kind=ArtifactKind.PLAN, path=Path("plan.md"), content="# Plan"),
        ]
        result = PlanResult(
            success=True,
            plan_path=Path("plan.md"),
            artifacts=artifacts,
        )
        assert result.success is True
        assert result.plan_path == Path("plan.md")
        assert len(result.artifacts) == 1
        assert result.error is None

    def test_plan_result_failure(self) -> None:
        """Test failed plan result."""
        result = PlanResult(
            success=False,
            error="Spec not found",
        )
        assert result.success is False
        assert result.plan_path is None
        assert result.artifacts == []
        assert result.error == "Spec not found"

    def test_plan_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        result = PlanResult(
            success=True,
            plan_path=Path("plan.md"),
        )
        data = result.model_dump(mode="json")
        restored = PlanResult.model_validate(data)
        assert restored.success == result.success

    def test_plan_result_immutable(self) -> None:
        """Test result is immutable (frozen)."""
        result = PlanResult(success=True)
        with pytest.raises(Exception):
            result.success = False  # type: ignore

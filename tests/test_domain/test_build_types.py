"""Tests for build domain types."""

from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory

import pytest
from rstn.domain.build.types import (
    BuildDiagnostic,
    BuildResult,
    DiagnosticLevel,
    ProjectType,
    detect_project_type,
)


class TestDiagnosticLevel:
    """Tests for DiagnosticLevel enum."""

    def test_diagnostic_level_values(self) -> None:
        """Test all diagnostic level values."""
        assert DiagnosticLevel.ERROR.value == "error"
        assert DiagnosticLevel.WARNING.value == "warning"
        assert DiagnosticLevel.NOTE.value == "note"
        assert DiagnosticLevel.HELP.value == "help"

    def test_diagnostic_level_is_string_enum(self) -> None:
        """Test DiagnosticLevel is a string enum."""
        for level in DiagnosticLevel:
            assert isinstance(level.value, str)


class TestProjectType:
    """Tests for ProjectType enum."""

    def test_project_type_values(self) -> None:
        """Test all project type values."""
        assert ProjectType.RUST.value == "rust"
        assert ProjectType.PYTHON.value == "python"
        assert ProjectType.NODE.value == "node"
        assert ProjectType.UNKNOWN.value == "unknown"


class TestBuildDiagnostic:
    """Tests for BuildDiagnostic model."""

    def test_diagnostic_creation(self) -> None:
        """Test creating build diagnostic."""
        diagnostic = BuildDiagnostic(
            level=DiagnosticLevel.ERROR,
            message="undefined reference to 'main'",
            file_path="src/main.rs",
            line=10,
            column=5,
            code="E0601",
        )
        assert diagnostic.level == DiagnosticLevel.ERROR
        assert diagnostic.message == "undefined reference to 'main'"
        assert diagnostic.file_path == "src/main.rs"
        assert diagnostic.line == 10
        assert diagnostic.column == 5
        assert diagnostic.code == "E0601"

    def test_diagnostic_minimal(self) -> None:
        """Test diagnostic with minimal fields."""
        diagnostic = BuildDiagnostic(
            level=DiagnosticLevel.WARNING,
            message="unused variable",
        )
        assert diagnostic.level == DiagnosticLevel.WARNING
        assert diagnostic.message == "unused variable"
        assert diagnostic.file_path is None
        assert diagnostic.line is None
        assert diagnostic.column is None
        assert diagnostic.code is None

    def test_diagnostic_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        diagnostic = BuildDiagnostic(
            level=DiagnosticLevel.ERROR,
            message="type mismatch",
            file_path="src/lib.rs",
            line=42,
        )
        json_str = diagnostic.model_dump_json()
        restored = BuildDiagnostic.model_validate_json(json_str)
        assert restored == diagnostic

    def test_diagnostic_immutable(self) -> None:
        """Test diagnostic is immutable (frozen)."""
        diagnostic = BuildDiagnostic(
            level=DiagnosticLevel.NOTE,
            message="Note message",
        )
        with pytest.raises(Exception):
            diagnostic.message = "New message"  # type: ignore


class TestBuildResult:
    """Tests for BuildResult model."""

    def test_build_result_success(self) -> None:
        """Test successful build result."""
        result = BuildResult(
            success=True,
            stdout="Build succeeded",
        )
        assert result.success is True
        assert result.stdout == "Build succeeded"
        assert result.stderr == ""
        assert result.diagnostics == []
        assert result.error_count == 0
        assert result.warning_count == 0

    def test_build_result_with_diagnostics(self) -> None:
        """Test build result with diagnostics."""
        diagnostics = [
            BuildDiagnostic(level=DiagnosticLevel.ERROR, message="Error 1"),
            BuildDiagnostic(level=DiagnosticLevel.ERROR, message="Error 2"),
            BuildDiagnostic(level=DiagnosticLevel.WARNING, message="Warning 1"),
            BuildDiagnostic(level=DiagnosticLevel.NOTE, message="Note 1"),
        ]
        result = BuildResult(
            success=False,
            diagnostics=diagnostics,
            stderr="Build failed",
        )
        assert result.success is False
        assert result.error_count == 2
        assert result.warning_count == 1
        assert len(result.diagnostics) == 4

    def test_build_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        diagnostics = [
            BuildDiagnostic(level=DiagnosticLevel.ERROR, message="Error"),
        ]
        result = BuildResult(
            success=False,
            diagnostics=diagnostics,
            stdout="",
            stderr="Build failed",
        )
        json_str = result.model_dump_json()
        restored = BuildResult.model_validate_json(json_str)
        assert restored.success == result.success
        assert restored.error_count == result.error_count

    def test_build_result_immutable(self) -> None:
        """Test result is immutable (frozen)."""
        result = BuildResult(success=True)
        with pytest.raises(Exception):
            result.success = False  # type: ignore


class TestDetectProjectType:
    """Tests for detect_project_type function."""

    def test_detect_rust_project(self) -> None:
        """Test detecting Rust project."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "Cargo.toml").touch()
            assert detect_project_type(root) == ProjectType.RUST

    def test_detect_python_project(self) -> None:
        """Test detecting Python project."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "pyproject.toml").touch()
            assert detect_project_type(root) == ProjectType.PYTHON

    def test_detect_node_project(self) -> None:
        """Test detecting Node project."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "package.json").touch()
            assert detect_project_type(root) == ProjectType.NODE

    def test_detect_unknown_project(self) -> None:
        """Test detecting unknown project."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            # No recognizable project files
            assert detect_project_type(root) == ProjectType.UNKNOWN

    def test_detect_rust_over_python(self) -> None:
        """Test Rust takes precedence over Python."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "Cargo.toml").touch()
            (root / "pyproject.toml").touch()
            assert detect_project_type(root) == ProjectType.RUST

    def test_detect_python_over_node(self) -> None:
        """Test Python takes precedence over Node."""
        with TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "pyproject.toml").touch()
            (root / "package.json").touch()
            assert detect_project_type(root) == ProjectType.PYTHON

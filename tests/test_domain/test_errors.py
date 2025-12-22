"""Tests for domain errors."""

from __future__ import annotations

import pytest
from pydantic import ValidationError
from rstn.domain.errors import DomainError, ErrorKind


class TestErrorKind:
    """Tests for ErrorKind enum."""

    def test_error_kind_values(self) -> None:
        """Test all error kinds are strings."""
        for kind in ErrorKind:
            assert isinstance(kind.value, str)

    def test_git_error_kinds(self) -> None:
        """Test git-related error kinds."""
        assert ErrorKind.GIT_COMMAND_FAILED.value == "git_command_failed"
        assert ErrorKind.GIT_WORKTREE_ERROR.value == "git_worktree_error"
        assert ErrorKind.GIT_COMMIT_BLOCKED.value == "git_commit_blocked"

    def test_build_error_kinds(self) -> None:
        """Test build-related error kinds."""
        assert ErrorKind.BUILD_FAILED.value == "build_failed"
        assert ErrorKind.TEST_FAILED.value == "test_failed"
        assert ErrorKind.CLIPPY_FAILED.value == "clippy_failed"

    def test_file_error_kinds(self) -> None:
        """Test file-related error kinds."""
        assert ErrorKind.FILE_NOT_FOUND.value == "file_not_found"
        assert ErrorKind.FILE_READ_ERROR.value == "file_read_error"
        assert ErrorKind.FILE_WRITE_ERROR.value == "file_write_error"

    def test_specify_error_kinds(self) -> None:
        """Test specify-related error kinds."""
        assert ErrorKind.SPECIFY_NUMBER_CONFLICT.value == "specify_number_conflict"
        assert ErrorKind.SPECIFY_CATALOG_ERROR.value == "specify_catalog_error"

    def test_claude_error_kinds(self) -> None:
        """Test Claude CLI error kinds."""
        assert ErrorKind.CLAUDE_NOT_FOUND.value == "claude_not_found"
        assert ErrorKind.CLAUDE_TIMEOUT.value == "claude_timeout"
        assert ErrorKind.CLAUDE_ERROR.value == "claude_error"


class TestDomainError:
    """Tests for DomainError model."""

    def test_domain_error_creation(self) -> None:
        """Test creating domain error."""
        error = DomainError(
            kind=ErrorKind.FILE_NOT_FOUND,
            message="File not found: test.txt",
            context={"path": "test.txt"},
        )
        assert error.kind == ErrorKind.FILE_NOT_FOUND
        assert error.message == "File not found: test.txt"
        assert error.context == {"path": "test.txt"}

    def test_domain_error_str(self) -> None:
        """Test string representation."""
        error = DomainError(
            kind=ErrorKind.GIT_COMMAND_FAILED,
            message="Git push failed",
        )
        assert str(error) == "git_command_failed: Git push failed"

    def test_domain_error_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        error = DomainError(
            kind=ErrorKind.BUILD_FAILED,
            message="Build failed",
            context={"exit_code": 1, "output": "Error: type mismatch"},
        )
        json_str = error.model_dump_json()
        restored = DomainError.model_validate_json(json_str)
        assert restored == error

    def test_domain_error_immutable(self) -> None:
        """Test error is immutable (frozen)."""
        error = DomainError(
            kind=ErrorKind.INTERNAL_ERROR,
            message="Internal error",
        )
        with pytest.raises(ValidationError):
            error.message = "New message"  # type: ignore

    def test_git_command_failed_factory(self) -> None:
        """Test git_command_failed factory method."""
        error = DomainError.git_command_failed("git push", "Permission denied")
        assert error.kind == ErrorKind.GIT_COMMAND_FAILED
        assert "git push" in error.message
        assert error.context["command"] == "git push"
        assert error.context["stderr"] == "Permission denied"

    def test_file_not_found_factory(self) -> None:
        """Test file_not_found factory method."""
        error = DomainError.file_not_found("/path/to/file.txt")
        assert error.kind == ErrorKind.FILE_NOT_FOUND
        assert "/path/to/file.txt" in error.message
        assert error.context["path"] == "/path/to/file.txt"

    def test_config_invalid_factory(self) -> None:
        """Test config_invalid factory method."""
        error = DomainError.config_invalid("/config.json", "Invalid JSON")
        assert error.kind == ErrorKind.CONFIG_INVALID
        assert "/config.json" in error.message
        assert "Invalid JSON" in error.message
        assert error.context["path"] == "/config.json"
        assert error.context["reason"] == "Invalid JSON"

    def test_workflow_cancelled_factory(self) -> None:
        """Test workflow_cancelled factory method."""
        error = DomainError.workflow_cancelled("wf-abc123")
        assert error.kind == ErrorKind.WORKFLOW_CANCELLED
        assert "wf-abc123" in error.message
        assert error.context["workflow_id"] == "wf-abc123"

    def test_claude_not_found_factory(self) -> None:
        """Test claude_not_found factory method."""
        error = DomainError.claude_not_found()
        assert error.kind == ErrorKind.CLAUDE_NOT_FOUND
        assert "not found" in error.message.lower()
        assert error.context == {}

    def test_claude_timeout_factory(self) -> None:
        """Test claude_timeout factory method."""
        error = DomainError.claude_timeout(60)
        assert error.kind == ErrorKind.CLAUDE_TIMEOUT
        assert "60" in error.message
        assert error.context["timeout_secs"] == 60

    def test_specify_number_conflict_factory(self) -> None:
        """Test specify_number_conflict factory method."""
        error = DomainError.specify_number_conflict("042")
        assert error.kind == ErrorKind.SPECIFY_NUMBER_CONFLICT
        assert "042" in error.message
        assert error.context["number"] == "042"

    def test_internal_error_factory(self) -> None:
        """Test internal_error factory method."""
        error = DomainError.internal_error("Something went wrong")
        assert error.kind == ErrorKind.INTERNAL_ERROR
        assert error.message == "Something went wrong"
        assert error.context == {}

    def test_domain_error_default_context(self) -> None:
        """Test default empty context."""
        error = DomainError(
            kind=ErrorKind.INTERNAL_ERROR,
            message="Test",
        )
        assert error.context == {}

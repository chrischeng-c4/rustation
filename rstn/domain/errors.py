"""Domain error types for rstn.

Comprehensive error types for all domain operations.
"""

from __future__ import annotations

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class ErrorKind(str, Enum):
    """Kind of domain error."""

    # Git errors
    GIT_COMMAND_FAILED = "git_command_failed"
    GIT_WORKTREE_ERROR = "git_worktree_error"
    GIT_COMMIT_BLOCKED = "git_commit_blocked"

    # Build/Test errors
    BUILD_FAILED = "build_failed"
    TEST_FAILED = "test_failed"
    CLIPPY_FAILED = "clippy_failed"

    # Service errors
    SERVICE_NOT_FOUND = "service_not_found"
    SERVICE_NOT_RUNNING = "service_not_running"

    # File errors
    FILE_NOT_FOUND = "file_not_found"
    FILE_READ_ERROR = "file_read_error"
    FILE_WRITE_ERROR = "file_write_error"
    DIRECTORY_ERROR = "directory_error"

    # Config errors
    CONFIG_INVALID = "config_invalid"
    CONFIG_NOT_FOUND = "config_not_found"

    # MCP errors
    MCP_REGISTRY_ERROR = "mcp_registry_error"
    MCP_CONFIG_ERROR = "mcp_config_error"

    # Workflow errors
    WORKFLOW_INVALID_STATE = "workflow_invalid_state"
    WORKFLOW_CANCELLED = "workflow_cancelled"
    WORKFLOW_TIMEOUT = "workflow_timeout"

    # Specify errors
    SPECIFY_NUMBER_CONFLICT = "specify_number_conflict"
    SPECIFY_CATALOG_ERROR = "specify_catalog_error"
    SPECIFY_TEMPLATE_ERROR = "specify_template_error"

    # Clarify errors
    CLARIFY_SPEC_NOT_FOUND = "clarify_spec_not_found"
    CLARIFY_INVALID_ANSWER = "clarify_invalid_answer"

    # Plan errors
    PLAN_CONTEXT_ERROR = "plan_context_error"
    PLAN_GENERATION_ERROR = "plan_generation_error"

    # Claude CLI errors
    CLAUDE_NOT_FOUND = "claude_not_found"
    CLAUDE_TIMEOUT = "claude_timeout"
    CLAUDE_ERROR = "claude_error"

    # Generic errors
    INVALID_INPUT = "invalid_input"
    INTERNAL_ERROR = "internal_error"


class DomainError(BaseModel):
    """Domain error with structured information.

    All domain errors are JSON serializable for state tracking.
    """

    model_config = {"frozen": True}

    kind: ErrorKind = Field(description="Error kind")
    message: str = Field(description="Human-readable error message")
    context: dict[str, Any] = Field(default_factory=dict, description="Additional context")

    def __str__(self) -> str:
        """Return string representation."""
        return f"{self.kind.value}: {self.message}"

    @classmethod
    def git_command_failed(cls, command: str, stderr: str) -> DomainError:
        """Create git command failed error."""
        return cls(
            kind=ErrorKind.GIT_COMMAND_FAILED,
            message=f"Git command failed: {command}",
            context={"command": command, "stderr": stderr},
        )

    @classmethod
    def file_not_found(cls, path: str) -> DomainError:
        """Create file not found error."""
        return cls(
            kind=ErrorKind.FILE_NOT_FOUND,
            message=f"File not found: {path}",
            context={"path": path},
        )

    @classmethod
    def config_invalid(cls, path: str, reason: str) -> DomainError:
        """Create config invalid error."""
        return cls(
            kind=ErrorKind.CONFIG_INVALID,
            message=f"Invalid config at {path}: {reason}",
            context={"path": path, "reason": reason},
        )

    @classmethod
    def workflow_cancelled(cls, workflow_id: str) -> DomainError:
        """Create workflow cancelled error."""
        return cls(
            kind=ErrorKind.WORKFLOW_CANCELLED,
            message=f"Workflow cancelled: {workflow_id}",
            context={"workflow_id": workflow_id},
        )

    @classmethod
    def claude_not_found(cls) -> DomainError:
        """Create Claude CLI not found error."""
        return cls(
            kind=ErrorKind.CLAUDE_NOT_FOUND,
            message="Claude CLI not found in PATH",
            context={},
        )

    @classmethod
    def claude_timeout(cls, timeout_secs: int) -> DomainError:
        """Create Claude timeout error."""
        return cls(
            kind=ErrorKind.CLAUDE_TIMEOUT,
            message=f"Claude CLI timed out after {timeout_secs}s",
            context={"timeout_secs": timeout_secs},
        )

    @classmethod
    def specify_number_conflict(cls, number: str) -> DomainError:
        """Create specify number conflict error."""
        return cls(
            kind=ErrorKind.SPECIFY_NUMBER_CONFLICT,
            message=f"Feature number already exists: {number}",
            context={"number": number},
        )

    @classmethod
    def internal_error(cls, message: str) -> DomainError:
        """Create internal error."""
        return cls(
            kind=ErrorKind.INTERNAL_ERROR,
            message=message,
            context={},
        )

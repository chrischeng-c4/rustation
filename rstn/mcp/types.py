"""MCP-specific types for rstn.

Defines configuration, message types, and result models for the MCP server.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field

# ========================================
# Server Configuration
# ========================================


class McpServerConfig(BaseModel):
    """Configuration for the embedded MCP server."""

    model_config = {"frozen": True}

    host: str = Field(default="127.0.0.1", description="Server host")
    port: int = Field(default=0, description="Server port (0 = dynamic)")
    session_id: str = Field(description="Session identifier")


# ========================================
# MCP Status Types
# ========================================


class McpStatus(str, Enum):
    """MCP status report types."""

    NEEDS_INPUT = "needs_input"
    COMPLETED = "completed"
    ERROR = "error"


# ========================================
# Tool Response Types
# ========================================


class McpToolResponse(BaseModel):
    """Standard MCP tool response format."""

    model_config = {"frozen": True}

    content: list[dict] = Field(description="Response content blocks")
    isError: bool = Field(default=False, description="Whether this is an error")  # noqa: N815 - MCP protocol uses camelCase

    @classmethod
    def text(cls, text: str, is_error: bool = False) -> McpToolResponse:
        """Create a text response."""
        return cls(
            content=[{"type": "text", "text": text}],
            isError=is_error,
        )

    @classmethod
    def error(cls, message: str) -> McpToolResponse:
        """Create an error response."""
        return cls.text(message, is_error=True)


# ========================================
# Hook Types
# ========================================


class HookDefinition(BaseModel):
    """Definition of a single hook."""

    model_config = {"frozen": True}

    command: str = Field(description="Command to execute")
    timeout_secs: int = Field(default=120, description="Timeout in seconds")
    cwd: str | None = Field(default=None, description="Working directory")
    env: dict[str, str] = Field(default_factory=dict, description="Environment vars")


class HookConfig(BaseModel):
    """Project hook configuration."""

    model_config = {"frozen": True}

    hooks: dict[str, HookDefinition] = Field(
        default_factory=dict,
        description="Hook definitions by name",
    )


class HookResult(BaseModel):
    """Result of hook execution."""

    model_config = {"frozen": True}

    hook_name: str = Field(description="Hook that was executed")
    exit_code: int = Field(description="Exit code")
    stdout: str = Field(description="Standard output")
    stderr: str = Field(description="Standard error")
    duration_secs: float = Field(description="Execution duration")


# ========================================
# Artifact Types (for rstn_read_spec)
# ========================================


class SpecArtifact(str, Enum):
    """Spec artifact types."""

    SPEC = "spec"
    PLAN = "plan"
    TASKS = "tasks"
    CHECKLIST = "checklist"
    ANALYSIS = "analysis"


ARTIFACT_FILENAMES: dict[SpecArtifact, str] = {
    SpecArtifact.SPEC: "spec.md",
    SpecArtifact.PLAN: "plan.md",
    SpecArtifact.TASKS: "tasks.md",
    SpecArtifact.CHECKLIST: "checklist.md",
    SpecArtifact.ANALYSIS: "analysis.md",
}

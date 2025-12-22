"""Runners domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, Field


class ScriptConfig(BaseModel):
    """Configuration for script execution."""

    model_config = {"frozen": True}

    cwd: Path = Field(description="Working directory")
    timeout_secs: int = Field(default=120, description="Timeout in seconds")
    env: dict[str, str] = Field(default_factory=dict, description="Environment variables")
    capture_output: bool = Field(default=True, description="Whether to capture output")


class RunnerResult(BaseModel):
    """Result of script execution."""

    model_config = {"frozen": True}

    success: bool = Field(description="Whether execution succeeded")
    exit_code: int = Field(description="Exit code")
    stdout: str = Field(default="", description="Standard output")
    stderr: str = Field(default="", description="Standard error")
    duration_ms: float | None = Field(default=None, description="Execution duration")

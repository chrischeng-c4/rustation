"""Effect types for rstn v2 MVI architecture.

AppEffect represents all possible side effects in the application.
Effects are descriptions of side effects, not the effects themselves.
The EffectExecutor is responsible for actually executing effects.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field

from rstn.state import AppState
from rstn.state.types import WorkflowId


class AgentKind(str, Enum):
    """Agent kind for spawning."""

    EXPLORE = "explore"
    PLAN = "plan"
    GENERAL_PURPOSE = "general_purpose"


# AppEffect is a discriminated union using Pydantic
class AppEffect(BaseModel):
    """Base class for all application effects.

    All side effects are represented as AppEffect subclasses.
    Effects are serializable descriptions that will be executed by EffectExecutor.
    Phase 3: Core effects for basic functionality (~18 effect types)
    """

    model_config = {"frozen": True}  # Effects are immutable


# ========================================
# Agent Execution
# ========================================


class SpawnAgent(AppEffect):
    """Spawn an agent."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    agent_kind: AgentKind = Field(description="Agent kind to spawn")
    prompt: str = Field(description="Prompt for the agent")
    mcp_config_path: Path | None = Field(default=None, description="Optional MCP config path")


class CancelAgent(AppEffect):
    """Cancel a running agent."""

    workflow_id: WorkflowId = Field(description="Workflow identifier to cancel")


# ========================================
# File Operations
# ========================================


class WriteFile(AppEffect):
    """Write file."""

    path: Path = Field(description="File path to write")
    contents: str = Field(description="File contents")


class ReadFile(AppEffect):
    """Read file."""

    path: Path = Field(description="File path to read")


class DeleteFile(AppEffect):
    """Delete file."""

    path: Path = Field(description="File path to delete")


# ========================================
# Command Execution
# ========================================


class RunCommand(AppEffect):
    """Run command."""

    cmd: str = Field(description="Command to run")
    args: list[str] = Field(default_factory=list, description="Command arguments")
    cwd: Path = Field(description="Working directory")


class RunBashScript(AppEffect):
    """Run bash script."""

    script_path: Path = Field(description="Script path to execute")
    args: list[str] = Field(default_factory=list, description="Script arguments")


# ========================================
# Timer
# ========================================


class StartTimer(AppEffect):
    """Start timer (for periodic ticks)."""

    timer_id: str = Field(description="Timer identifier")
    delay_ms: int = Field(gt=0, description="Delay in milliseconds")


class StopTimer(AppEffect):
    """Stop timer."""

    timer_id: str = Field(description="Timer identifier to stop")


# ========================================
# Workflow Management
# ========================================


class CancelWorkflow(AppEffect):
    """Cancel workflow."""

    workflow_id: WorkflowId = Field(description="Workflow identifier to cancel")


# ========================================
# State Persistence
# ========================================


class SaveState(AppEffect):
    """Save state to file."""

    path: Path = Field(description="File path to save state")
    state: AppState = Field(description="State to save")


class LoadState(AppEffect):
    """Load state from file."""

    path: Path = Field(description="File path to load state from")


# ========================================
# Logging
# ========================================


class LogInfo(AppEffect):
    """Log info message."""

    message: str = Field(description="Info message to log")


class LogError(AppEffect):
    """Log error message."""

    message: str = Field(description="Error message to log")


class LogDebug(AppEffect):
    """Log debug message."""

    message: str = Field(description="Debug message to log")


# ========================================
# UI Updates
# ========================================


class Render(AppEffect):
    """Render UI (trigger re-render)."""

    pass


class QuitApp(AppEffect):
    """Quit application."""

    pass


# ========================================
# Batch
# ========================================


class Batch(AppEffect):
    """Execute multiple effects."""

    effects: list[AppEffect] = Field(description="Effects to execute")


# Note: executor import is at the end to avoid circular imports
# This is intentional and marked with noqa
__all__ = [
    "AppEffect",
    "AgentKind",
    # Agent Execution
    "SpawnAgent",
    "CancelAgent",
    # File Operations
    "WriteFile",
    "ReadFile",
    "DeleteFile",
    # Command Execution
    "RunCommand",
    "RunBashScript",
    # Timer
    "StartTimer",
    "StopTimer",
    # Workflow Management
    "CancelWorkflow",
    # State Persistence
    "SaveState",
    "LoadState",
    # Logging
    "LogInfo",
    "LogError",
    "LogDebug",
    # UI Updates
    "Render",
    "QuitApp",
    # Batch
    "Batch",
    # Executor
    "EffectExecutor",
    "DefaultEffectExecutor",
    "MessageSender",
]

# Import executor after defining all effect types to avoid circular imports
from rstn.effect.executor import (  # noqa: E402
    DefaultEffectExecutor,
    EffectExecutor,
    MessageSender,
)

"""Message types for rstn v2 MVI architecture.

AppMsg represents all possible events in the application.
Messages are processed by the reduce() function to produce new state.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field

from rstn.state.types import ViewType, WorkflowId


class KeyCode(str, Enum):
    """Key codes for keyboard events."""

    # Character keys (use actual char, not enum)
    # These are handled specially in from_char()
    ENTER = "enter"
    ESC = "esc"
    BACKSPACE = "backspace"
    TAB = "tab"
    UP = "up"
    DOWN = "down"
    LEFT = "left"
    RIGHT = "right"
    HOME = "home"
    END = "end"
    PAGE_UP = "page_up"
    PAGE_DOWN = "page_down"


class KeyModifiers(BaseModel):
    """Key modifiers for keyboard events."""

    model_config = {"frozen": True}

    ctrl: bool = Field(default=False, description="Ctrl key pressed")
    shift: bool = Field(default=False, description="Shift key pressed")
    alt: bool = Field(default=False, description="Alt key pressed")

    @classmethod
    def ctrl_key(cls) -> KeyModifiers:
        """Create modifiers with only Ctrl."""
        return cls(ctrl=True)

    @classmethod
    def shift_key(cls) -> KeyModifiers:
        """Create modifiers with only Shift."""
        return cls(shift=True)

    @classmethod
    def alt_key(cls) -> KeyModifiers:
        """Create modifiers with only Alt."""
        return cls(alt=True)

    def is_empty(self) -> bool:
        """Check if no modifiers are pressed."""
        return not (self.ctrl or self.shift or self.alt)


# AppMsg is a discriminated union using Pydantic
class AppMsg(BaseModel):
    """Base class for all application messages.

    All events in the application are represented as AppMsg subclasses.
    Phase 3: Core events for basic functionality (~27 message types)
    """

    model_config = {"frozen": True}  # Messages are immutable


# ========================================
# User Input Events
# ========================================


class KeyPressed(AppMsg):
    """Key pressed event."""

    key: str = Field(description="Character key or KeyCode value")
    modifiers: KeyModifiers = Field(default_factory=KeyModifiers, description="Key modifiers")


class MouseClicked(AppMsg):
    """Mouse clicked event."""

    x: int = Field(ge=0, description="X coordinate")
    y: int = Field(ge=0, description="Y coordinate")


class Tick(AppMsg):
    """Tick event for animations and timers."""

    pass


# ========================================
# View Events
# ========================================


class SwitchView(AppMsg):
    """Switch to a different view."""

    view: ViewType = Field(description="Target view")


class ScrollContent(AppMsg):
    """Scroll content."""

    delta: int = Field(description="Scroll delta (positive = down, negative = up)")


class SelectCommand(AppMsg):
    """Select command by index."""

    index: int = Field(ge=0, description="Command index to select")


# ========================================
# Workflow Events
# ========================================


class WorkflowStartRequested(AppMsg):
    """Request to start a workflow."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    workflow_type: str = Field(description="Workflow type name")
    params: str = Field(description="Workflow parameters (JSON string)")


class WorkflowStepCompleted(AppMsg):
    """Workflow step completed."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    step_name: str = Field(description="Step name")
    success: bool = Field(description="Whether step succeeded")


class WorkflowCompleted(AppMsg):
    """Workflow completed successfully."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")


class WorkflowFailed(AppMsg):
    """Workflow failed."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    error: str = Field(description="Error message")


class WorkflowCancelled(AppMsg):
    """Workflow cancelled."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")


# ========================================
# Agent Events
# ========================================


class AgentStreamDelta(AppMsg):
    """Agent stream delta (partial output)."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    delta: str = Field(description="Text delta to append")


class AgentCompleted(AppMsg):
    """Agent completed."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    output: str = Field(description="Final agent output")


# ========================================
# Effect Result Events
# ========================================


class EffectCompleted(AppMsg):
    """Effect completed successfully."""

    effect_id: str = Field(description="Effect identifier")
    result: str = Field(description="Effect result")


class EffectFailed(AppMsg):
    """Effect failed."""

    effect_id: str = Field(description="Effect identifier")
    error: str = Field(description="Error message")


class FileWritten(AppMsg):
    """File written successfully."""

    path: Path = Field(description="File path that was written")


class FileReadCompleted(AppMsg):
    """File read completed."""

    path: Path = Field(description="File path that was read")
    contents: str = Field(description="File contents")


class CommandCompleted(AppMsg):
    """Command execution completed."""

    exit_code: int = Field(description="Exit code")
    stdout: str = Field(description="Standard output")
    stderr: str = Field(description="Standard error")


# ========================================
# System Events
# ========================================


class ErrorOccurred(AppMsg):
    """Error occurred."""

    message: str = Field(description="Error message")


class StateRestored(AppMsg):
    """State restored from file."""

    pass


class StateSaved(AppMsg):
    """State saved to file."""

    path: Path = Field(description="File path where state was saved")


class Quit(AppMsg):
    """Request to quit application."""

    pass


class Noop(AppMsg):
    """No-op message (for testing)."""

    pass


# Export all message types
__all__ = [
    "AppMsg",
    "KeyCode",
    "KeyModifiers",
    # User Input
    "KeyPressed",
    "MouseClicked",
    "Tick",
    # View Events
    "SwitchView",
    "ScrollContent",
    "SelectCommand",
    # Workflow Events
    "WorkflowStartRequested",
    "WorkflowStepCompleted",
    "WorkflowCompleted",
    "WorkflowFailed",
    "WorkflowCancelled",
    # Agent Events
    "AgentStreamDelta",
    "AgentCompleted",
    # Effect Results
    "EffectCompleted",
    "EffectFailed",
    "FileWritten",
    "FileReadCompleted",
    "CommandCompleted",
    # System Events
    "ErrorOccurred",
    "StateRestored",
    "StateSaved",
    "Quit",
    "Noop",
]

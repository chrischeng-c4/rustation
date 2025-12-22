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


# ========================================
# Domain Result Events
# ========================================


class GitCommandCompleted(AppMsg):
    """Git command execution completed."""

    effect_id: str = Field(description="Effect identifier")
    exit_code: int = Field(description="Exit code")
    stdout: str = Field(description="Standard output")
    stderr: str = Field(description="Standard error")


class CargoCommandCompleted(AppMsg):
    """Cargo command execution completed."""

    effect_id: str = Field(description="Effect identifier")
    success: bool = Field(description="Whether command succeeded")
    stdout: str = Field(description="Standard output")
    stderr: str = Field(description="Standard error")


class ClaudeStreamDelta(AppMsg):
    """Claude CLI streaming output delta."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    delta: str = Field(description="Text delta")


class ClaudeCompleted(AppMsg):
    """Claude CLI execution completed."""

    workflow_id: WorkflowId = Field(description="Workflow identifier")
    output: str = Field(description="Full output")
    success: bool = Field(description="Whether execution succeeded")
    error: str | None = Field(default=None, description="Error message if failed")


class DirectoryCreated(AppMsg):
    """Directory created successfully."""

    path: Path = Field(description="Directory path")


class DirectoryListed(AppMsg):
    """Directory listing completed."""

    effect_id: str = Field(description="Effect identifier")
    path: Path = Field(description="Directory path")
    entries: list[str] = Field(description="Directory entries")


class FileExistsChecked(AppMsg):
    """File existence check completed."""

    effect_id: str = Field(description="Effect identifier")
    path: Path = Field(description="File path")
    exists: bool = Field(description="Whether file exists")


class FileRenamed(AppMsg):
    """File renamed/moved successfully."""

    src: Path = Field(description="Source path")
    dst: Path = Field(description="Destination path")


# ========================================
# Domain Workflow Events
# ========================================


class SecurityScanRequested(AppMsg):
    """Request security scan of staged changes."""

    scan_all: bool = Field(default=False, description="Scan all changes (not just staged)")


class SecurityScanCompleted(AppMsg):
    """Security scan completed."""

    blocked: bool = Field(description="Whether commit should be blocked")
    warning_count: int = Field(description="Number of warnings")
    critical_count: int = Field(description="Number of critical issues")


class SpecGenerationRequested(AppMsg):
    """Request spec generation."""

    description: str = Field(description="Feature description")


class SpecGenerationStarted(AppMsg):
    """Spec generation workflow started."""

    feature_number: str = Field(description="Allocated feature number")
    feature_name: str = Field(description="Generated feature name")


class SpecGenerationCompleted(AppMsg):
    """Spec generation completed."""

    spec_path: str = Field(description="Path to generated spec")
    success: bool = Field(description="Whether generation succeeded")
    error: str | None = Field(default=None, description="Error if failed")


class PlanGenerationRequested(AppMsg):
    """Request plan generation."""

    feature_name: str = Field(description="Feature name to generate plan for")


class PlanGenerationStarted(AppMsg):
    """Plan generation workflow started."""

    feature_name: str = Field(description="Feature name")


class PlanGenerationCompleted(AppMsg):
    """Plan generation completed."""

    plan_path: str = Field(description="Path to plan file")
    artifacts: list[str] = Field(default_factory=list, description="Generated artifact paths")
    success: bool = Field(description="Whether generation succeeded")
    error: str | None = Field(default=None, description="Error if failed")


class ClarifySessionRequested(AppMsg):
    """Request clarify session."""

    spec_path: str = Field(description="Path to spec to clarify")


class ClarifySessionStarted(AppMsg):
    """Clarify session started."""

    spec_path: str = Field(description="Spec being clarified")
    question_count: int = Field(description="Number of questions generated")


class ClarifyQuestionReady(AppMsg):
    """Next clarify question ready."""

    question_id: int = Field(description="Question ID")
    question_text: str = Field(description="Question text")
    category: str = Field(description="Question category")
    remaining: int = Field(description="Remaining questions")


class ClarifyAnswerSubmitted(AppMsg):
    """Clarify answer submitted."""

    question_id: int = Field(description="Question ID")
    answer: str = Field(description="User's answer")


class ClarifySessionCompleted(AppMsg):
    """Clarify session completed."""

    spec_path: str = Field(description="Spec path")
    questions_answered: int = Field(description="Number of questions answered")
    spec_updated: bool = Field(description="Whether spec was updated")


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
    # Domain Result Events
    "GitCommandCompleted",
    "CargoCommandCompleted",
    "ClaudeStreamDelta",
    "ClaudeCompleted",
    "DirectoryCreated",
    "DirectoryListed",
    "FileExistsChecked",
    "FileRenamed",
    # Domain Workflow Events
    "SecurityScanRequested",
    "SecurityScanCompleted",
    "SpecGenerationRequested",
    "SpecGenerationStarted",
    "SpecGenerationCompleted",
    "PlanGenerationRequested",
    "PlanGenerationStarted",
    "PlanGenerationCompleted",
    "ClarifySessionRequested",
    "ClarifySessionStarted",
    "ClarifyQuestionReady",
    "ClarifyAnswerSubmitted",
    "ClarifySessionCompleted",
]

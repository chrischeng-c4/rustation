"""Worktree view state.

State for the main development interface (worktree view).
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field

from rstn.state.types import WorkflowId


class ContentType(str, Enum):
    """Type of content displayed in the content area."""

    EMPTY = "empty"
    SPEC = "spec"
    PLAN = "plan"
    TIMELINE = "timeline"
    LOG = "log"
    HELP = "help"


class Command(BaseModel):
    """A command that can be executed from the command list."""

    model_config = {"frozen": False}

    id: str = Field(description="Unique command identifier")
    label: str = Field(description="Display label")
    description: str = Field(default="", description="Command description")
    enabled: bool = Field(default=True, description="Whether command is enabled")
    workflow_type: str | None = Field(
        default=None, description="Workflow type to trigger (if applicable)"
    )


class WorktreeViewState(BaseModel):
    """Worktree view state.

    Main development interface with command list and dynamic content area.
    This is the primary view where users spend most of their time.

    14 fields (< 15 limit as per KB constraints)
    """

    model_config = {"frozen": False}

    # Content area
    content_type: ContentType = Field(
        default=ContentType.EMPTY, description="Current content type"
    )
    spec_content: str | None = Field(default=None, description="Current spec content (markdown)")
    plan_content: str | None = Field(default=None, description="Current plan content (markdown)")
    log_content: list[str] = Field(
        default_factory=list, description="Log messages (latest first)"
    )

    # Commands
    commands: list[Command] = Field(
        default_factory=list, description="Available commands in left panel"
    )
    selected_command_index: int = Field(
        default=0, ge=0, description="Selected command index in command list"
    )

    # Workflow state
    active_workflow_id: WorkflowId | None = Field(
        default=None, description="Currently executing workflow ID"
    )
    workflow_output: str = Field(default="", description="Current workflow output/stream")

    # UI state
    command_list_scroll: int = Field(default=0, ge=0, description="Command list scroll offset")
    content_scroll: int = Field(default=0, ge=0, description="Content area scroll offset")

    # Input state
    input_mode: bool = Field(default=False, description="Whether in input mode")
    input_buffer: str = Field(default="", description="Current input buffer")
    input_prompt: str = Field(default="", description="Input prompt message")

    # Status
    status_message: str = Field(default="Ready", description="Status bar message")

    def with_content(
        self, content_type: ContentType, content: str | None = None
    ) -> WorktreeViewState:
        """Update content area.

        Args:
            content_type: Type of content to display
            content: Content text (optional, depends on type)

        Returns:
            New WorktreeViewState with updated content
        """
        updates: dict[str, object] = {"content_type": content_type}

        if content_type == ContentType.SPEC:
            updates["spec_content"] = content
        elif content_type == ContentType.PLAN:
            updates["plan_content"] = content

        return self.model_copy(update=updates)

    def with_workflow(self, workflow_id: WorkflowId | None) -> WorktreeViewState:
        """Set active workflow.

        Args:
            workflow_id: Workflow ID (None to clear)

        Returns:
            New WorktreeViewState with updated workflow
        """
        return self.model_copy(
            update={
                "active_workflow_id": workflow_id,
                "workflow_output": "" if workflow_id else self.workflow_output,
            }
        )

    def append_workflow_output(self, delta: str) -> WorktreeViewState:
        """Append to workflow output stream.

        Args:
            delta: Text to append

        Returns:
            New WorktreeViewState with appended output
        """
        return self.model_copy(update={"workflow_output": self.workflow_output + delta})

    def add_log(self, message: str) -> WorktreeViewState:
        """Add log message.

        Args:
            message: Log message to add

        Returns:
            New WorktreeViewState with log added (max 100 messages)
        """
        logs = [message] + self.log_content[:99]  # Keep latest 100
        return self.model_copy(update={"log_content": logs})

    def select_command(self, index: int) -> WorktreeViewState:
        """Select command by index.

        Args:
            index: Command index to select

        Returns:
            New WorktreeViewState with updated selection
        """
        # Clamp to valid range
        clamped = max(0, min(index, len(self.commands) - 1)) if self.commands else 0
        return self.model_copy(update={"selected_command_index": clamped})

    def enter_input_mode(self, prompt: str) -> WorktreeViewState:
        """Enter input mode with prompt.

        Args:
            prompt: Prompt message to show

        Returns:
            New WorktreeViewState in input mode
        """
        return self.model_copy(
            update={"input_mode": True, "input_prompt": prompt, "input_buffer": ""}
        )

    def exit_input_mode(self) -> WorktreeViewState:
        """Exit input mode.

        Returns:
            New WorktreeViewState with input mode cleared
        """
        return self.model_copy(
            update={"input_mode": False, "input_prompt": "", "input_buffer": ""}
        )

    def assert_invariants(self) -> None:
        """Assert worktree state invariants.

        Raises:
            AssertionError: If any invariant is violated
        """
        # Selected command index must be valid
        if self.commands:
            assert (
                0 <= self.selected_command_index < len(self.commands)
            ), "Selected command index must be within commands range"

        # Log content should not exceed 100 messages
        assert len(self.log_content) <= 100, "Log content should not exceed 100 messages"

        # If in input mode, must have prompt
        if self.input_mode:
            assert self.input_prompt, "Input mode requires a prompt message"

        # All commands must have unique IDs
        command_ids = [cmd.id for cmd in self.commands]
        assert len(command_ids) == len(set(command_ids)), "Command IDs must be unique"

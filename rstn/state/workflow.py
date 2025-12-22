"""Generic workflow state container.

Provides a reusable state structure for all workflow types.
"""

from __future__ import annotations

from enum import Enum
from typing import Generic, TypeVar

from pydantic import BaseModel, Field

from rstn.state.types import WorkflowId

# Generic type parameter for workflow-specific data
T = TypeVar("T")


class WorkflowStatus(str, Enum):
    """Workflow execution status."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class WorkflowState(BaseModel, Generic[T]):  # noqa: UP046
    """Generic workflow state container.

    T represents the workflow-specific data/status.
    T must be a Pydantic-compatible type (serializable).

    Examples:
        # Simple string data
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="prompt-claude",
            data="prompt text"
        )

        # Complex structured data
        @dataclass
        class SpecifyData:
            feature: str
            status: str

        workflow = WorkflowState[SpecifyData](...)
    """

    model_config = {"frozen": False}

    id: WorkflowId = Field(description="Unique workflow identifier")
    workflow_type: str = Field(description="Workflow type name (e.g., 'prompt-claude')")
    status: WorkflowStatus = Field(
        default=WorkflowStatus.PENDING, description="Current workflow status"
    )
    data: T = Field(description="Workflow-specific data")
    error: str | None = Field(default=None, description="Error message if failed")
    progress: float = Field(default=0.0, ge=0.0, le=1.0, description="Progress (0.0 to 1.0)")

    def start(self) -> WorkflowState[T]:
        """Mark workflow as running.

        Returns:
            New WorkflowState with status=RUNNING
        """
        return self.model_copy(update={"status": WorkflowStatus.RUNNING})

    def with_progress(self, progress: float) -> WorkflowState[T]:
        """Update workflow progress.

        Args:
            progress: Progress value (will be clamped to 0.0-1.0)

        Returns:
            New WorkflowState with updated progress
        """
        clamped = max(0.0, min(1.0, progress))
        return self.model_copy(update={"progress": clamped})

    def complete(self) -> WorkflowState[T]:
        """Mark workflow as completed.

        Returns:
            New WorkflowState with status=COMPLETED and progress=1.0
        """
        return self.model_copy(
            update={"status": WorkflowStatus.COMPLETED, "progress": 1.0, "error": None}
        )

    def fail(self, error: str) -> WorkflowState[T]:
        """Mark workflow as failed.

        Args:
            error: Error message describing the failure

        Returns:
            New WorkflowState with status=FAILED and error message
        """
        return self.model_copy(update={"status": WorkflowStatus.FAILED, "error": error})

    def cancel(self) -> WorkflowState[T]:
        """Mark workflow as cancelled.

        Returns:
            New WorkflowState with status=CANCELLED
        """
        return self.model_copy(update={"status": WorkflowStatus.CANCELLED, "error": None})

    def is_active(self) -> bool:
        """Check if workflow is active (pending or running).

        Returns:
            True if workflow is pending or running
        """
        return self.status in {WorkflowStatus.PENDING, WorkflowStatus.RUNNING}

    def is_finished(self) -> bool:
        """Check if workflow is finished (completed, failed, or cancelled).

        Returns:
            True if workflow is in terminal state
        """
        return self.status in {
            WorkflowStatus.COMPLETED,
            WorkflowStatus.FAILED,
            WorkflowStatus.CANCELLED,
        }

    def assert_invariants(self) -> None:
        """Assert workflow state invariants.

        Raises:
            AssertionError: If any invariant is violated
        """
        # ID must not be empty
        assert self.id, "Workflow ID cannot be empty"

        # Workflow type must not be empty
        assert self.workflow_type, "Workflow type cannot be empty"

        # Progress must be in valid range (Pydantic already validates this)
        assert 0.0 <= self.progress <= 1.0, "Progress must be between 0.0 and 1.0"

        # If failed, must have error message
        if self.status == WorkflowStatus.FAILED:
            assert self.error, "Failed workflow must have error message"

        # If completed, progress should be 1.0
        if self.status == WorkflowStatus.COMPLETED:
            assert self.progress == 1.0, "Completed workflow must have progress 1.0"

"""Tests for WorkflowState."""

from __future__ import annotations

import pytest
from rstn.state.workflow import WorkflowState, WorkflowStatus


class TestWorkflowStatus:
    """Test WorkflowStatus enum."""

    def test_workflow_status_values(self) -> None:
        """WorkflowStatus has expected values."""
        assert WorkflowStatus.PENDING == "pending"  # type: ignore
        assert WorkflowStatus.RUNNING == "running"  # type: ignore
        assert WorkflowStatus.COMPLETED == "completed"  # type: ignore
        assert WorkflowStatus.FAILED == "failed"  # type: ignore
        assert WorkflowStatus.CANCELLED == "cancelled"  # type: ignore


class TestWorkflowStateCreation:
    """Test WorkflowState creation."""

    def test_new_workflow_state(self) -> None:
        """Create new workflow state."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="prompt-claude",
            data="test prompt",
        )

        assert workflow.id == "wf-123"
        assert workflow.workflow_type == "prompt-claude"
        assert workflow.status == WorkflowStatus.PENDING
        assert workflow.data == "test prompt"
        assert workflow.error is None
        assert workflow.progress == 0.0

    def test_workflow_state_with_complex_data(self) -> None:
        """WorkflowState can hold complex data."""
        data: dict[str, object] = {"prompt": "test", "config": {"model": "claude"}}
        workflow = WorkflowState[dict[str, object]](
            id="wf-123",
            workflow_type="test",
            data=data,
        )

        assert workflow.data == data


class TestWorkflowStateTransitions:
    """Test workflow state transitions."""

    def test_start_workflow(self) -> None:
        """start() marks workflow as running."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        )

        assert workflow.status == WorkflowStatus.PENDING

        started = workflow.start()

        # Original unchanged
        assert workflow.status == WorkflowStatus.PENDING
        # New state updated
        assert started.status == WorkflowStatus.RUNNING

    def test_with_progress(self) -> None:
        """with_progress() updates progress."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start()

        updated = workflow.with_progress(0.5)

        assert workflow.progress == 0.0
        assert updated.progress == 0.5

    def test_with_progress_clamps_to_range(self) -> None:
        """with_progress() clamps to 0.0-1.0."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        )

        # Test clamping to max
        updated = workflow.with_progress(1.5)
        assert updated.progress == 1.0

        # Test clamping to min
        updated = workflow.with_progress(-0.5)
        assert updated.progress == 0.0

    def test_complete_workflow(self) -> None:
        """complete() marks workflow as completed."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start().with_progress(0.8)

        completed = workflow.complete()

        assert completed.status == WorkflowStatus.COMPLETED
        assert completed.progress == 1.0
        assert completed.error is None

    def test_fail_workflow(self) -> None:
        """fail() marks workflow as failed with error."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start()

        failed = workflow.fail("Test error")

        assert failed.status == WorkflowStatus.FAILED
        assert failed.error == "Test error"

    def test_cancel_workflow(self) -> None:
        """cancel() marks workflow as cancelled."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start()

        cancelled = workflow.cancel()

        assert cancelled.status == WorkflowStatus.CANCELLED
        assert cancelled.error is None


class TestWorkflowStateQueries:
    """Test workflow state query methods."""

    def test_is_active_pending(self) -> None:
        """is_active() returns True for pending workflows."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        )

        assert workflow.is_active()
        assert not workflow.is_finished()

    def test_is_active_running(self) -> None:
        """is_active() returns True for running workflows."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start()

        assert workflow.is_active()
        assert not workflow.is_finished()

    def test_is_finished_completed(self) -> None:
        """is_finished() returns True for completed workflows."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).complete()

        assert not workflow.is_active()
        assert workflow.is_finished()

    def test_is_finished_failed(self) -> None:
        """is_finished() returns True for failed workflows."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).fail("error")

        assert not workflow.is_active()
        assert workflow.is_finished()

    def test_is_finished_cancelled(self) -> None:
        """is_finished() returns True for cancelled workflows."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).cancel()

        assert not workflow.is_active()
        assert workflow.is_finished()


class TestWorkflowStateInvariants:
    """Test workflow state invariants."""

    def test_valid_workflow_invariants(self) -> None:
        """Valid workflow passes invariant checks."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
        ).start().with_progress(0.5)

        # Should not raise
        workflow.assert_invariants()

    def test_empty_id_fails(self) -> None:
        """Empty workflow ID violates invariants."""
        workflow = WorkflowState[str](
            id="",
            workflow_type="test",
            data="test",
        )

        with pytest.raises(AssertionError, match="Workflow ID cannot be empty"):
            workflow.assert_invariants()

    def test_empty_workflow_type_fails(self) -> None:
        """Empty workflow type violates invariants."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="",
            data="test",
        )

        with pytest.raises(AssertionError, match="Workflow type cannot be empty"):
            workflow.assert_invariants()

    def test_failed_without_error_fails(self) -> None:
        """Failed workflow without error message violates invariants."""
        # Create failed workflow by manually setting status
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
            status=WorkflowStatus.FAILED,
        )

        with pytest.raises(AssertionError, match="Failed workflow must have error message"):
            workflow.assert_invariants()

    def test_completed_without_full_progress_fails(self) -> None:
        """Completed workflow without progress=1.0 violates invariants."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="test",
            data="test",
            status=WorkflowStatus.COMPLETED,
            progress=0.8,
        )

        with pytest.raises(AssertionError, match="Completed workflow must have progress 1.0"):
            workflow.assert_invariants()


class TestWorkflowStateSerialization:
    """Test workflow state serialization."""

    def test_workflow_state_serialization(self) -> None:
        """WorkflowState can be serialized."""
        workflow = WorkflowState[str](
            id="wf-123",
            workflow_type="prompt-claude",
            data="test prompt",
        ).start().with_progress(0.5)

        json_str = workflow.model_dump_json()
        loaded = WorkflowState[str].model_validate_json(json_str)

        assert loaded.id == workflow.id
        assert loaded.workflow_type == workflow.workflow_type
        assert loaded.status == workflow.status
        assert loaded.data == workflow.data
        assert loaded.progress == workflow.progress

    def test_workflow_state_with_list_data(self) -> None:
        """WorkflowState with list data can be serialized."""
        workflow = WorkflowState[list[str]](
            id="wf-123",
            workflow_type="test",
            data=["item1", "item2"],
        )

        json_str = workflow.model_dump_json()
        loaded = WorkflowState[list[str]].model_validate_json(json_str)

        assert loaded.data == ["item1", "item2"]


class TestWorkflowStateChaining:
    """Test chaining workflow transitions."""

    def test_full_workflow_lifecycle(self) -> None:
        """Test complete workflow lifecycle."""
        # Create → Start → Progress → Complete
        workflow = (
            WorkflowState[str](
                id="wf-123",
                workflow_type="test",
                data="test data",
            )
            .start()
            .with_progress(0.3)
            .with_progress(0.7)
            .complete()
        )

        assert workflow.status == WorkflowStatus.COMPLETED
        assert workflow.progress == 1.0
        workflow.assert_invariants()

    def test_failed_workflow_lifecycle(self) -> None:
        """Test workflow that fails."""
        workflow = (
            WorkflowState[str](
                id="wf-123",
                workflow_type="test",
                data="test",
            )
            .start()
            .with_progress(0.5)
            .fail("Something went wrong")
        )

        assert workflow.status == WorkflowStatus.FAILED
        assert workflow.error == "Something went wrong"
        workflow.assert_invariants()

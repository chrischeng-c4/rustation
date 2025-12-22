"""Dashboard view state.

State for the project overview/dashboard view.
"""

from __future__ import annotations

from pydantic import BaseModel, Field

from rstn.state.types import WorkflowId


class DashboardState(BaseModel):
    """Dashboard view state.

    Shows project overview, recent workflows, and quick actions.
    """

    model_config = {"frozen": False}

    project_name: str | None = Field(default=None, description="Current project name")
    recent_workflows: list[WorkflowId] = Field(
        default_factory=list, description="Recently executed workflow IDs"
    )
    selected_index: int | None = Field(default=None, description="Selected workflow index")
    scroll_position: int = Field(default=0, ge=0, description="Scroll position")

    def with_project(self, name: str) -> DashboardState:
        """Set project name.

        Args:
            name: Project name

        Returns:
            New DashboardState with updated project name
        """
        return self.model_copy(update={"project_name": name})

    def add_workflow(self, workflow_id: WorkflowId) -> DashboardState:
        """Add workflow to recent list.

        Args:
            workflow_id: Workflow ID to add

        Returns:
            New DashboardState with workflow added (max 10 recent)
        """
        # Add to front, keep max 10
        recent = [workflow_id] + [w for w in self.recent_workflows if w != workflow_id]
        recent = recent[:10]
        return self.model_copy(update={"recent_workflows": recent})

    def with_selection(self, index: int | None) -> DashboardState:
        """Update selected workflow index.

        Args:
            index: Selected index (None to clear)

        Returns:
            New DashboardState with updated selection
        """
        return self.model_copy(update={"selected_index": index})

    def assert_invariants(self) -> None:
        """Assert dashboard state invariants.

        Raises:
            AssertionError: If any invariant is violated
        """
        # Recent workflows should not exceed 10
        assert len(self.recent_workflows) <= 10, "Recent workflows should not exceed 10"

        # No duplicate workflows in recent list
        assert len(self.recent_workflows) == len(
            set(self.recent_workflows)
        ), "Recent workflows should not have duplicates"

        # If selected_index is set, it should be valid
        if self.selected_index is not None:
            assert (
                0 <= self.selected_index < len(self.recent_workflows)
            ), "Selected index must be within recent workflows range"

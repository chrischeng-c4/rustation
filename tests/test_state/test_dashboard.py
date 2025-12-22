"""Tests for DashboardState."""

from __future__ import annotations

import pytest
from rstn.state.dashboard import DashboardState


class TestDashboardStateCreation:
    """Test DashboardState creation."""

    def test_default_dashboard_state(self) -> None:
        """Create default dashboard state."""
        dashboard = DashboardState()

        assert dashboard.project_name is None
        assert dashboard.recent_workflows == []
        assert dashboard.selected_index is None
        assert dashboard.scroll_position == 0

    def test_dashboard_state_with_project(self) -> None:
        """Create dashboard state with project name."""
        dashboard = DashboardState(project_name="my-project")

        assert dashboard.project_name == "my-project"
        assert dashboard.recent_workflows == []

    def test_dashboard_state_with_workflows(self) -> None:
        """Create dashboard state with workflows."""
        workflows = ["wf-1", "wf-2", "wf-3"]
        dashboard = DashboardState(recent_workflows=workflows)

        assert dashboard.recent_workflows == workflows
        assert len(dashboard.recent_workflows) == 3


class TestWithProject:
    """Test with_project() method."""

    def test_with_project_sets_name(self) -> None:
        """with_project() sets project name."""
        dashboard = DashboardState()
        updated = dashboard.with_project("test-project")

        # Original unchanged
        assert dashboard.project_name is None
        # New state updated
        assert updated.project_name == "test-project"

    def test_with_project_overwrites_existing(self) -> None:
        """with_project() overwrites existing project name."""
        dashboard = DashboardState(project_name="old-project")
        updated = dashboard.with_project("new-project")

        assert dashboard.project_name == "old-project"
        assert updated.project_name == "new-project"

    def test_with_project_preserves_other_fields(self) -> None:
        """with_project() preserves other fields."""
        dashboard = DashboardState(
            recent_workflows=["wf-1"], selected_index=0, scroll_position=5
        )
        updated = dashboard.with_project("test")

        assert updated.recent_workflows == ["wf-1"]
        assert updated.selected_index == 0
        assert updated.scroll_position == 5


class TestAddWorkflow:
    """Test add_workflow() method."""

    def test_add_workflow_to_empty_list(self) -> None:
        """add_workflow() adds to empty list."""
        dashboard = DashboardState()
        updated = dashboard.add_workflow("wf-1")

        assert dashboard.recent_workflows == []
        assert updated.recent_workflows == ["wf-1"]

    def test_add_workflow_prepends_to_list(self) -> None:
        """add_workflow() prepends to front of list."""
        dashboard = DashboardState(recent_workflows=["wf-1", "wf-2"])
        updated = dashboard.add_workflow("wf-3")

        assert updated.recent_workflows == ["wf-3", "wf-1", "wf-2"]

    def test_add_workflow_deduplicates(self) -> None:
        """add_workflow() removes duplicates."""
        dashboard = DashboardState(recent_workflows=["wf-1", "wf-2", "wf-3"])
        updated = dashboard.add_workflow("wf-2")

        # wf-2 should move to front, no duplicate
        assert updated.recent_workflows == ["wf-2", "wf-1", "wf-3"]
        assert len(updated.recent_workflows) == 3

    def test_add_workflow_limits_to_10(self) -> None:
        """add_workflow() limits recent workflows to 10."""
        workflows = [f"wf-{i}" for i in range(10)]
        dashboard = DashboardState(recent_workflows=workflows)

        updated = dashboard.add_workflow("wf-new")

        assert len(updated.recent_workflows) == 10
        assert updated.recent_workflows[0] == "wf-new"
        assert "wf-9" not in updated.recent_workflows  # Oldest removed

    def test_add_workflow_dedup_maintains_limit(self) -> None:
        """add_workflow() maintains limit when deduplicating."""
        workflows = [f"wf-{i}" for i in range(10)]
        dashboard = DashboardState(recent_workflows=workflows)

        # Add existing workflow
        updated = dashboard.add_workflow("wf-5")

        assert len(updated.recent_workflows) == 10
        assert updated.recent_workflows[0] == "wf-5"
        # wf-5 moved to front, wf-9 removed
        assert updated.recent_workflows.count("wf-5") == 1


class TestWithSelection:
    """Test with_selection() method."""

    def test_with_selection_sets_index(self) -> None:
        """with_selection() sets selection index."""
        dashboard = DashboardState(recent_workflows=["wf-1", "wf-2"])
        updated = dashboard.with_selection(1)

        assert dashboard.selected_index is None
        assert updated.selected_index == 1

    def test_with_selection_clears_selection(self) -> None:
        """with_selection() can clear selection."""
        dashboard = DashboardState(
            recent_workflows=["wf-1"], selected_index=0
        )
        updated = dashboard.with_selection(None)

        assert dashboard.selected_index == 0
        assert updated.selected_index is None

    def test_with_selection_updates_existing(self) -> None:
        """with_selection() updates existing selection."""
        dashboard = DashboardState(
            recent_workflows=["wf-1", "wf-2", "wf-3"], selected_index=0
        )
        updated = dashboard.with_selection(2)

        assert updated.selected_index == 2

    def test_with_selection_preserves_other_fields(self) -> None:
        """with_selection() preserves other fields."""
        dashboard = DashboardState(
            project_name="test", recent_workflows=["wf-1"], scroll_position=5
        )
        updated = dashboard.with_selection(0)

        assert updated.project_name == "test"
        assert updated.recent_workflows == ["wf-1"]
        assert updated.scroll_position == 5


class TestScrollPosition:
    """Test scroll_position field."""

    def test_scroll_position_default(self) -> None:
        """scroll_position defaults to 0."""
        dashboard = DashboardState()
        assert dashboard.scroll_position == 0

    def test_scroll_position_validation(self) -> None:
        """scroll_position must be >= 0."""
        with pytest.raises(ValueError):
            DashboardState(scroll_position=-1)

    def test_scroll_position_can_be_updated(self) -> None:
        """scroll_position can be updated via model_copy."""
        dashboard = DashboardState()
        updated = dashboard.model_copy(update={"scroll_position": 10})

        assert dashboard.scroll_position == 0
        assert updated.scroll_position == 10


class TestDashboardStateInvariants:
    """Test dashboard state invariants."""

    def test_valid_dashboard_invariants(self) -> None:
        """Valid dashboard passes invariant checks."""
        dashboard = DashboardState(
            project_name="test",
            recent_workflows=["wf-1", "wf-2"],
            selected_index=1,
        )

        # Should not raise
        dashboard.assert_invariants()

    def test_invariant_max_10_workflows(self) -> None:
        """Cannot have more than 10 recent workflows."""
        workflows = [f"wf-{i}" for i in range(11)]
        dashboard = DashboardState(recent_workflows=workflows)

        with pytest.raises(AssertionError, match="should not exceed 10"):
            dashboard.assert_invariants()

    def test_invariant_no_duplicate_workflows(self) -> None:
        """Recent workflows cannot have duplicates."""
        dashboard = DashboardState(recent_workflows=["wf-1", "wf-2", "wf-1"])

        with pytest.raises(AssertionError, match="should not have duplicates"):
            dashboard.assert_invariants()

    def test_invariant_selected_index_valid_range(self) -> None:
        """Selected index must be within workflows range."""
        dashboard = DashboardState(
            recent_workflows=["wf-1", "wf-2"], selected_index=5
        )

        with pytest.raises(AssertionError, match="must be within recent workflows range"):
            dashboard.assert_invariants()

    def test_invariant_selected_index_negative(self) -> None:
        """Selected index cannot be negative."""
        dashboard = DashboardState(
            recent_workflows=["wf-1"], selected_index=-1
        )

        with pytest.raises(AssertionError, match="must be within recent workflows range"):
            dashboard.assert_invariants()

    def test_invariant_none_selection_valid(self) -> None:
        """None selection is always valid."""
        dashboard = DashboardState(
            recent_workflows=["wf-1"], selected_index=None
        )

        # Should not raise
        dashboard.assert_invariants()

    def test_invariant_empty_workflows_with_selection(self) -> None:
        """Cannot have selection with empty workflows."""
        dashboard = DashboardState(recent_workflows=[], selected_index=0)

        with pytest.raises(AssertionError, match="must be within recent workflows range"):
            dashboard.assert_invariants()


class TestDashboardStateSerialization:
    """Test dashboard state serialization."""

    def test_dashboard_state_serialization(self) -> None:
        """DashboardState can be serialized."""
        dashboard = DashboardState(
            project_name="test-project",
            recent_workflows=["wf-1", "wf-2", "wf-3"],
            selected_index=1,
            scroll_position=10,
        )

        json_str = dashboard.model_dump_json()
        loaded = DashboardState.model_validate_json(json_str)

        assert loaded.project_name == dashboard.project_name
        assert loaded.recent_workflows == dashboard.recent_workflows
        assert loaded.selected_index == dashboard.selected_index
        assert loaded.scroll_position == dashboard.scroll_position

    def test_dashboard_state_with_none_values(self) -> None:
        """DashboardState with None values can be serialized."""
        dashboard = DashboardState(
            project_name=None, recent_workflows=[], selected_index=None
        )

        json_str = dashboard.model_dump_json()
        loaded = DashboardState.model_validate_json(json_str)

        assert loaded.project_name is None
        assert loaded.recent_workflows == []
        assert loaded.selected_index is None

    def test_dashboard_state_dict_round_trip(self) -> None:
        """DashboardState can round-trip through dict."""
        dashboard = DashboardState(
            project_name="test", recent_workflows=["wf-1", "wf-2"]
        )

        data = dashboard.model_dump()
        loaded = DashboardState.model_validate(data)

        assert loaded == dashboard


class TestDashboardStateImmutability:
    """Test dashboard state immutability (conceptual)."""

    def test_methods_return_new_instance(self) -> None:
        """Methods return new instances."""
        dashboard = DashboardState()

        updated1 = dashboard.with_project("test")
        updated2 = dashboard.add_workflow("wf-1")
        updated3 = dashboard.with_selection(0)

        # All should be different instances
        assert updated1 is not dashboard
        assert updated2 is not dashboard
        assert updated3 is not dashboard

    def test_original_unchanged_after_updates(self) -> None:
        """Original dashboard unchanged after updates."""
        dashboard = DashboardState(
            project_name="original", recent_workflows=["wf-1"], selected_index=0
        )

        dashboard.with_project("new")
        dashboard.add_workflow("wf-2")
        dashboard.with_selection(None)

        # Original should be unchanged
        assert dashboard.project_name == "original"
        assert dashboard.recent_workflows == ["wf-1"]
        assert dashboard.selected_index == 0


class TestDashboardStateChaining:
    """Test chaining dashboard state updates."""

    def test_chain_all_updates(self) -> None:
        """Test chaining all update methods."""
        dashboard = (
            DashboardState()
            .with_project("test-project")
            .add_workflow("wf-1")
            .add_workflow("wf-2")
            .add_workflow("wf-3")
            .with_selection(0)
        )

        assert dashboard.project_name == "test-project"
        assert dashboard.recent_workflows == ["wf-3", "wf-2", "wf-1"]
        assert dashboard.selected_index == 0
        dashboard.assert_invariants()

    def test_chain_with_limit(self) -> None:
        """Test chaining respects 10 workflow limit."""
        dashboard = DashboardState()

        # Add 12 workflows
        for i in range(12):
            dashboard = dashboard.add_workflow(f"wf-{i}")

        assert len(dashboard.recent_workflows) == 10
        assert dashboard.recent_workflows[0] == "wf-11"  # Most recent
        assert "wf-0" not in dashboard.recent_workflows  # Oldest removed
        assert "wf-1" not in dashboard.recent_workflows
        dashboard.assert_invariants()

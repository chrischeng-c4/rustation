"""Tests for pure view render functions.

These tests verify that render functions are pure (deterministic)
and produce correct output for given state.
"""

from __future__ import annotations

from rstn.state import AppState
from rstn.state.types import ViewType
from rstn.state.worktree import Command, ContentType, WorktreeViewState
from rstn.tui.render import (
    CommandListRender,
    ContentAreaRender,
    StatusBarRender,
    ViewRender,
)
from rstn.tui.render.views import (
    render_app,
    render_command_list,
    render_content_area,
    render_dashboard_view,
    render_settings_view,
    render_status_bar,
    render_worktree_view,
)


class TestRenderCommandList:
    """Test render_command_list() pure function."""

    def test_empty_commands(self) -> None:
        """Empty commands renders with has_commands=False."""
        state = WorktreeViewState(commands=[])
        result = render_command_list(state)

        assert isinstance(result, CommandListRender)
        assert not result.has_commands
        assert result.items == []
        assert result.selected_index == 0

    def test_commands_with_selection(self) -> None:
        """Commands render with selection indicator."""
        commands = [
            Command(id="cmd-1", label="First"),
            Command(id="cmd-2", label="Second"),
            Command(id="cmd-3", label="Third"),
        ]
        state = WorktreeViewState(commands=commands, selected_command_index=1)
        result = render_command_list(state)

        assert result.has_commands
        assert len(result.items) == 3
        assert result.selected_index == 1
        # Second item should have selection indicator
        assert result.items[0].startswith(" ")
        assert result.items[1].startswith(">")
        assert result.items[2].startswith(" ")

    def test_disabled_command(self) -> None:
        """Disabled commands show disabled suffix."""
        commands = [
            Command(id="cmd-1", label="Enabled", enabled=True),
            Command(id="cmd-2", label="Disabled", enabled=False),
        ]
        state = WorktreeViewState(commands=commands)
        result = render_command_list(state)

        assert "(disabled)" not in result.items[0]
        assert "(disabled)" in result.items[1]

    def test_deterministic_output(self) -> None:
        """Same state produces same output (pure function)."""
        commands = [Command(id="x", label="X")]
        state = WorktreeViewState(commands=commands)

        result1 = render_command_list(state)
        result2 = render_command_list(state)

        assert result1 == result2


class TestRenderContentArea:
    """Test render_content_area() pure function."""

    def test_empty_content(self) -> None:
        """Empty content renders with placeholder."""
        state = WorktreeViewState(content_type=ContentType.EMPTY)
        result = render_content_area(state)

        assert isinstance(result, ContentAreaRender)
        assert result.content_type == "empty"

    def test_spec_content(self) -> None:
        """Spec content renders with title and content."""
        state = WorktreeViewState(
            content_type=ContentType.SPEC,
            spec_content="# My Spec\n\nDescription here",
        )
        result = render_content_area(state)

        assert result.content_type == "spec"
        assert result.title == "Specification"
        assert "My Spec" in result.content

    def test_plan_content(self) -> None:
        """Plan content renders with title and content."""
        state = WorktreeViewState(
            content_type=ContentType.PLAN,
            plan_content="# Implementation Plan\n\n1. Step one",
        )
        result = render_content_area(state)

        assert result.content_type == "plan"
        assert result.title == "Plan"
        assert "Implementation Plan" in result.content

    def test_log_content(self) -> None:
        """Log content renders log messages."""
        state = WorktreeViewState(
            content_type=ContentType.LOG,
            log_content=["Log message 1", "Log message 2"],
        )
        result = render_content_area(state)

        assert result.content_type == "log"
        assert result.title == "Logs"
        assert "Log message 1" in result.content

    def test_help_content(self) -> None:
        """Help content renders help text."""
        state = WorktreeViewState(content_type=ContentType.HELP)
        result = render_content_area(state)

        assert result.content_type == "help"
        assert result.title == "Help"
        assert "Navigation" in result.content

    def test_workflow_output_in_empty_mode(self) -> None:
        """Workflow output shows when content_type is EMPTY."""
        state = WorktreeViewState(
            content_type=ContentType.EMPTY,
            workflow_output="Processing...\nStep 1 complete.",
        )
        result = render_content_area(state)

        assert result.content_type == "workflow"
        assert "Processing" in result.content

    def test_no_spec_content(self) -> None:
        """Spec type with None content shows placeholder."""
        state = WorktreeViewState(
            content_type=ContentType.SPEC,
            spec_content=None,
        )
        result = render_content_area(state)

        assert "No specification loaded" in result.content


class TestRenderStatusBar:
    """Test render_status_bar() pure function."""

    def test_ready_status(self) -> None:
        """Ready status shows ready message."""
        state = AppState()
        result = render_status_bar(state)

        assert isinstance(result, StatusBarRender)
        assert result.style == "normal"
        assert "Ready" in result.message

    def test_error_status(self) -> None:
        """Error message shows with error style."""
        state = AppState(error_message="Something went wrong")
        result = render_status_bar(state)

        assert result.style == "error"
        assert "Something went wrong" in result.message

    def test_workflow_running_status(self) -> None:
        """Active workflow shows in status."""
        worktree = WorktreeViewState(active_workflow_id="wf-123")
        state = AppState(current_view=ViewType.WORKTREE, worktree_view=worktree)
        result = render_status_bar(state)

        assert "wf-123" in result.message

    def test_dashboard_view_status(self) -> None:
        """Dashboard view shows dashboard in status."""
        state = AppState(current_view=ViewType.DASHBOARD)
        result = render_status_bar(state)

        assert "Dashboard" in result.message


class TestRenderWorktreeView:
    """Test render_worktree_view() complete view render."""

    def test_basic_render(self) -> None:
        """Basic worktree view renders all components."""
        state = AppState(current_view=ViewType.WORKTREE)
        result = render_worktree_view(state)

        assert isinstance(result, ViewRender)
        assert result.view_name == "worktree"
        assert isinstance(result.command_list, CommandListRender)
        assert isinstance(result.content_area, ContentAreaRender)
        assert isinstance(result.status_bar, StatusBarRender)


class TestRenderDashboardView:
    """Test render_dashboard_view() complete view render."""

    def test_basic_render(self) -> None:
        """Basic dashboard view renders all components."""
        state = AppState(current_view=ViewType.DASHBOARD)
        result = render_dashboard_view(state)

        assert isinstance(result, ViewRender)
        assert result.view_name == "dashboard"
        assert "Dashboard" in result.content_area.title

    def test_project_name_shown(self) -> None:
        """Project name appears in dashboard content."""
        state = AppState(current_view=ViewType.DASHBOARD)
        state = state.model_copy(
            update={"dashboard_view": state.dashboard_view.with_project("my-project")}
        )
        result = render_dashboard_view(state)

        assert "my-project" in result.content_area.content


class TestRenderSettingsView:
    """Test render_settings_view() complete view render."""

    def test_basic_render(self) -> None:
        """Basic settings view renders all components."""
        state = AppState(current_view=ViewType.SETTINGS)
        result = render_settings_view(state)

        assert isinstance(result, ViewRender)
        assert result.view_name == "settings"
        assert "Settings" in result.content_area.title

    def test_settings_values_shown(self) -> None:
        """Settings values appear in content."""
        state = AppState(current_view=ViewType.SETTINGS)
        result = render_settings_view(state)

        # Default settings
        assert "dark" in result.content_area.content.lower()
        assert "Enabled" in result.content_area.content


class TestRenderApp:
    """Test render_app() view routing."""

    def test_routes_to_worktree(self) -> None:
        """Routes to worktree view renderer."""
        state = AppState(current_view=ViewType.WORKTREE)
        result = render_app(state)

        assert result.view_name == "worktree"

    def test_routes_to_dashboard(self) -> None:
        """Routes to dashboard view renderer."""
        state = AppState(current_view=ViewType.DASHBOARD)
        result = render_app(state)

        assert result.view_name == "dashboard"

    def test_routes_to_settings(self) -> None:
        """Routes to settings view renderer."""
        state = AppState(current_view=ViewType.SETTINGS)
        result = render_app(state)

        assert result.view_name == "settings"

    def test_deterministic_routing(self) -> None:
        """Same state routes to same view (pure function)."""
        state = AppState(current_view=ViewType.WORKTREE)

        result1 = render_app(state)
        result2 = render_app(state)

        assert result1.view_name == result2.view_name
        assert result1.command_list == result2.command_list


class TestRenderOutputSerialization:
    """Verify render outputs can be serialized for debugging."""

    def test_command_list_render_to_dict(self) -> None:
        """CommandListRender can be converted to dict."""
        render = CommandListRender(
            items=["  Item 1", "> Item 2"],
            selected_index=1,
            has_commands=True,
        )
        # Dataclass has __dict__ via slots
        import dataclasses

        data = dataclasses.asdict(render)
        assert data["items"] == ["  Item 1", "> Item 2"]
        assert data["selected_index"] == 1

    def test_view_render_to_dict(self) -> None:
        """ViewRender can be converted to dict."""
        import dataclasses

        state = AppState()
        render = render_app(state)
        data = dataclasses.asdict(render)

        assert "command_list" in data
        assert "content_area" in data
        assert "status_bar" in data
        assert "view_name" in data

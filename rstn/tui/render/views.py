"""Pure view render functions.

All functions in this module are pure: same input state produces same output.
No side effects, no I/O, no time reads.

UI = render(State)
"""

from __future__ import annotations

from rstn.state import AppState
from rstn.state.dashboard import DashboardState
from rstn.state.settings import SettingsState
from rstn.state.types import ViewType
from rstn.state.worktree import ContentType, WorktreeViewState
from rstn.tui.render import (
    CommandListRender,
    ContentAreaRender,
    StatusBarRender,
    ViewRender,
)
from rstn.tui.render.content import (
    format_help_content,
    format_log_content,
    format_plan_content,
    format_spec_content,
    format_workflow_output,
)

__all__ = [
    "render_app",
    "render_command_list",
    "render_content_area",
    "render_status_bar",
    "render_worktree_view",
    "render_dashboard_view",
    "render_settings_view",
]


def render_command_list(state: WorktreeViewState) -> CommandListRender:
    """Render command list with selection indicator.

    Args:
        state: Worktree view state

    Returns:
        CommandListRender with formatted command strings
    """
    if not state.commands:
        return CommandListRender(
            items=[],
            selected_index=0,
            has_commands=False,
        )

    items = []
    for i, cmd in enumerate(state.commands):
        prefix = ">" if i == state.selected_command_index else " "
        suffix = "" if cmd.enabled else " (disabled)"
        items.append(f"{prefix} {cmd.label}{suffix}")

    return CommandListRender(
        items=items,
        selected_index=state.selected_command_index,
        has_commands=True,
    )


def render_content_area(state: WorktreeViewState) -> ContentAreaRender:
    """Render content area based on content type.

    Args:
        state: Worktree view state

    Returns:
        ContentAreaRender with title and content
    """
    content_type = state.content_type

    # Determine title and content based on type
    if content_type == ContentType.SPEC:
        return ContentAreaRender(
            title="Specification",
            content=format_spec_content(state.spec_content),
            content_type="spec",
        )

    if content_type == ContentType.PLAN:
        return ContentAreaRender(
            title="Plan",
            content=format_plan_content(state.plan_content),
            content_type="plan",
        )

    if content_type == ContentType.LOG:
        return ContentAreaRender(
            title="Logs",
            content=format_log_content(state.log_content),
            content_type="log",
        )

    if content_type == ContentType.HELP:
        return ContentAreaRender(
            title="Help",
            content=format_help_content(),
            content_type="help",
        )

    if content_type == ContentType.TIMELINE:
        return ContentAreaRender(
            title="Timeline",
            content="[Timeline view not yet implemented]",
            content_type="timeline",
        )

    # EMPTY or if there's workflow output
    if state.workflow_output:
        return ContentAreaRender(
            title="Workflow Output",
            content=format_workflow_output(state.workflow_output),
            content_type="workflow",
        )

    return ContentAreaRender(
        title="Content",
        content="[Empty]",
        content_type="empty",
    )


def render_status_bar(state: AppState) -> StatusBarRender:
    """Render status bar with error or ready message.

    Args:
        state: Root application state

    Returns:
        StatusBarRender with message and style
    """
    if state.error_message:
        return StatusBarRender(
            message=f"Error: {state.error_message}",
            style="error",
        )

    # Show worktree status if on worktree view
    if state.current_view == ViewType.WORKTREE:
        worktree = state.worktree_view
        if worktree.active_workflow_id:
            return StatusBarRender(
                message=f"Running workflow: {worktree.active_workflow_id}",
                style="normal",
            )
        return StatusBarRender(
            message=worktree.status_message,
            style="normal",
        )

    # Default status for other views
    view_name = state.current_view.value.title()
    return StatusBarRender(
        message=f"{view_name} | Ready",
        style="normal",
    )


def render_worktree_view(state: AppState) -> ViewRender:
    """Render complete worktree view.

    Args:
        state: Root application state

    Returns:
        ViewRender with all components for worktree view
    """
    worktree = state.worktree_view
    return ViewRender(
        command_list=render_command_list(worktree),
        content_area=render_content_area(worktree),
        status_bar=render_status_bar(state),
        view_name="worktree",
    )


def render_dashboard_view(state: AppState) -> ViewRender:
    """Render complete dashboard view.

    Args:
        state: Root application state

    Returns:
        ViewRender with all components for dashboard view
    """
    dashboard = state.dashboard_view
    _render_dashboard_content(dashboard)

    # Dashboard uses a simplified command list (recent workflows)
    items = []
    for i, workflow_id in enumerate(dashboard.recent_workflows):
        prefix = ">" if i == dashboard.selected_index else " "
        items.append(f"{prefix} {workflow_id}")

    command_list = CommandListRender(
        items=items if items else ["  No recent workflows"],
        selected_index=dashboard.selected_index or 0,
        has_commands=bool(dashboard.recent_workflows),
    )

    # Dashboard content shows project info
    project = dashboard.project_name or "No project"
    content = f"""
Project: {project}

Recent Workflows: {len(dashboard.recent_workflows)}

Select a workflow to view details.
""".strip()

    content_area = ContentAreaRender(
        title="Dashboard",
        content=content,
        content_type="dashboard",
    )

    return ViewRender(
        command_list=command_list,
        content_area=content_area,
        status_bar=render_status_bar(state),
        view_name="dashboard",
    )


def _render_dashboard_content(dashboard: DashboardState) -> str:
    """Render dashboard content area.

    Args:
        dashboard: Dashboard view state

    Returns:
        Formatted dashboard content
    """
    project = dashboard.project_name or "No project"
    return f"""
Project: {project}

Recent Workflows: {len(dashboard.recent_workflows)}
""".strip()


def render_settings_view(state: AppState) -> ViewRender:
    """Render complete settings view.

    Args:
        state: Root application state

    Returns:
        ViewRender with all components for settings view
    """
    settings = state.settings_view

    # Settings uses option list as "commands"
    options = [
        ("Theme", settings.theme.value),
        ("Mouse", "Enabled" if settings.mouse_enabled else "Disabled"),
        ("Auto Save", "Enabled" if settings.auto_save else "Disabled"),
        ("Log Level", settings.log_level),
    ]

    items = [f"  {name}: {value}" for name, value in options]

    command_list = CommandListRender(
        items=items,
        selected_index=0,
        has_commands=True,
    )

    content_area = ContentAreaRender(
        title="Settings",
        content=_render_settings_content(settings),
        content_type="settings",
    )

    return ViewRender(
        command_list=command_list,
        content_area=content_area,
        status_bar=render_status_bar(state),
        view_name="settings",
    )


def _render_settings_content(settings: SettingsState) -> str:
    """Render settings detail content.

    Args:
        settings: Settings view state

    Returns:
        Formatted settings content
    """
    return f"""
Settings
========

Theme:      {settings.theme.value}
Mouse:      {"Enabled" if settings.mouse_enabled else "Disabled"}
Auto Save:  {"Enabled" if settings.auto_save else "Disabled"}
Log Level:  {settings.log_level}

Use j/k to navigate, Enter to edit.
""".strip()


def render_app(state: AppState) -> ViewRender:
    """Main render function - routes to appropriate view renderer.

    This is the primary entry point for rendering. Based on state.current_view,
    it delegates to the appropriate view renderer.

    Args:
        state: Root application state

    Returns:
        ViewRender with all rendered components for current view
    """
    if state.current_view == ViewType.WORKTREE:
        return render_worktree_view(state)

    if state.current_view == ViewType.DASHBOARD:
        return render_dashboard_view(state)

    if state.current_view == ViewType.SETTINGS:
        return render_settings_view(state)

    # Fallback to worktree (should never happen)
    return render_worktree_view(state)

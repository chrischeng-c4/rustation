"""Rendering layer for rstn TUI.

This module provides pure render functions that transform AppState into
renderable output. Following State-First MVI: UI = render(State).

Exports:
    - RenderOutput types (dataclasses for render results)
    - render_app() - Main render function
    - View-specific render functions
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

# Import render functions - this works because views.py imports render types
# from this module using string annotations
if TYPE_CHECKING:
    pass

__all__ = [
    # Render output types
    "CommandListRender",
    "ContentAreaRender",
    "StatusBarRender",
    "TabBarRender",
    "FooterRender",
    "ViewRender",
    # Render functions
    "render_app",
    "render_command_list",
    "render_content_area",
    "render_status_bar",
    "render_tab_bar",
    "render_footer",
]


@dataclass(frozen=True)
class CommandListRender:
    """Rendered command list output.

    Attributes:
        items: Formatted command strings with selection indicator (">" prefix)
        selected_index: Currently selected command index
        has_commands: Whether there are any commands to display
    """

    items: list[str]
    selected_index: int
    has_commands: bool


@dataclass(frozen=True)
class ContentAreaRender:
    """Rendered content area output.

    Attributes:
        title: Content area title (e.g., "Specification", "Plan")
        content: The actual content to display
        content_type: Type of content ("empty", "spec", "plan", "workflow", "log")
    """

    title: str
    content: str
    content_type: str


@dataclass(frozen=True)
class StatusBarRender:
    """Rendered status bar output.

    Attributes:
        message: Status message to display
        style: Style hint ("normal", "error", "warning", "success")
    """

    message: str
    style: str


@dataclass(frozen=True)
class TabBarRender:
    """Rendered tab bar output.

    Attributes:
        tabs: List of (label, is_active) tuples
        active_index: Currently active tab index (0-indexed)
    """

    tabs: list[tuple[str, bool]]
    active_index: int


@dataclass(frozen=True)
class FooterRender:
    """Rendered footer output.

    Attributes:
        shortcuts: List of shortcut descriptions ("q quit", "y copy", etc.)
    """

    shortcuts: list[str]


@dataclass(frozen=True)
class ViewRender:
    """Complete view render output.

    Contains all rendered components for a view.

    Attributes:
        tab_bar: Rendered tab bar
        command_list: Rendered command list
        content_area: Rendered content area
        status_bar: Rendered status bar
        footer: Rendered footer
        view_name: Name of the current view ("worktree", "dashboard", "settings")
    """

    tab_bar: TabBarRender
    command_list: CommandListRender
    content_area: ContentAreaRender
    status_bar: StatusBarRender
    footer: FooterRender
    view_name: str


# Re-export render functions - import at end to avoid circular imports
# ruff: noqa: E402
from rstn.tui.render.views import (  # noqa: E402
    render_app,
    render_command_list,
    render_content_area,
    render_footer,
    render_status_bar,
    render_tab_bar,
)

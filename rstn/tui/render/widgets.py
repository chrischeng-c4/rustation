"""Custom Textual widgets for rstn TUI.

These widgets accept render output dataclasses and update their display.
They bridge the pure render functions with Textual's widget system.
"""

from __future__ import annotations

from textual.widgets import Static

from rstn.tui.render import (
    CommandListRender,
    ContentAreaRender,
    FooterRender,
    StatusBarRender,
    TabBarRender,
)

__all__ = [
    "CommandListWidget",
    "ContentAreaWidget",
    "StatusBarWidget",
    "TabBarWidget",
    "FooterWidget",
]


class CommandListWidget(Static):
    """Widget for displaying the command list.

    Shows a list of commands with selection indicator.
    Updates from CommandListRender output.
    """

    DEFAULT_CSS = """
    CommandListWidget {
        width: 100%;
        height: 100%;
        padding: 0 1;
    }
    """

    def __init__(
        self,
        content: str = "",
        *,
        name: str | None = None,
        id: str | None = None,  # noqa: A002
        classes: str | None = None,
        disabled: bool = False,
    ) -> None:
        """Initialize command list widget."""
        super().__init__(
            content,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )
        self._last_render: CommandListRender | None = None

    def update_from_render(self, render_output: CommandListRender) -> None:
        """Update widget from render output.

        Args:
            render_output: CommandListRender containing formatted items
        """
        self._last_render = render_output

        if not render_output.has_commands:
            self.update("No commands available")
            return

        # Join items with newlines
        content = "\n".join(render_output.items)
        self.update(content)

    @property
    def selected_index(self) -> int:
        """Get currently selected index."""
        if self._last_render is None:
            return 0
        return self._last_render.selected_index


class ContentAreaWidget(Static):
    """Widget for displaying the main content area.

    Shows spec, plan, workflow output, or other content.
    Updates from ContentAreaRender output.
    """

    DEFAULT_CSS = """
    ContentAreaWidget {
        width: 100%;
        height: 100%;
        padding: 0 1;
    }
    """

    def __init__(
        self,
        content: str = "",
        *,
        name: str | None = None,
        id: str | None = None,  # noqa: A002
        classes: str | None = None,
        disabled: bool = False,
    ) -> None:
        """Initialize content area widget."""
        super().__init__(
            content,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )
        self._last_render: ContentAreaRender | None = None

    def update_from_render(self, render_output: ContentAreaRender) -> None:
        """Update widget from render output.

        Args:
            render_output: ContentAreaRender containing content
        """
        self._last_render = render_output
        self.update(render_output.content)

    @property
    def content_type(self) -> str:
        """Get current content type."""
        if self._last_render is None:
            return "empty"
        return self._last_render.content_type

    @property
    def title(self) -> str:
        """Get current title."""
        if self._last_render is None:
            return "Content"
        return self._last_render.title


class StatusBarWidget(Static):
    """Widget for displaying the status bar.

    Shows status messages with appropriate styling.
    Updates from StatusBarRender output.
    """

    DEFAULT_CSS = """
    StatusBarWidget {
        height: 1;
        background: $boost;
        padding: 0 1;
    }

    StatusBarWidget.error {
        background: $error;
        color: $text;
    }

    StatusBarWidget.warning {
        background: $warning;
        color: $text;
    }

    StatusBarWidget.success {
        background: $success;
        color: $text;
    }
    """

    def __init__(
        self,
        content: str = "",
        *,
        name: str | None = None,
        id: str | None = None,  # noqa: A002
        classes: str | None = None,
        disabled: bool = False,
    ) -> None:
        """Initialize status bar widget."""
        super().__init__(
            content,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )
        self._last_render: StatusBarRender | None = None

    def update_from_render(self, render_output: StatusBarRender) -> None:
        """Update widget from render output.

        Args:
            render_output: StatusBarRender containing message and style
        """
        self._last_render = render_output

        # Update content
        self.update(render_output.message)

        # Update CSS classes for styling
        self.remove_class("error", "warning", "success", "normal")
        if render_output.style in ("error", "warning", "success"):
            self.add_class(render_output.style)

    @property
    def current_style(self) -> str:
        """Get current style."""
        if self._last_render is None:
            return "normal"
        return self._last_render.style


class TabBarWidget(Static):
    """Widget for displaying the tab bar.

    Shows tabs for high-level view navigation: 1 Worktree | 2 Dashboard | 3 Settings
    Active tab is highlighted.
    """

    DEFAULT_CSS = """
    TabBarWidget {
        height: 1;
        background: $surface;
        padding: 0 1;
    }

    TabBarWidget .active-tab {
        background: $accent;
        color: $text;
    }
    """

    def __init__(
        self,
        content: str = "",
        *,
        name: str | None = None,
        id: str | None = None,  # noqa: A002
        classes: str | None = None,
        disabled: bool = False,
    ) -> None:
        """Initialize tab bar widget."""
        super().__init__(
            content,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )
        self._last_render: TabBarRender | None = None

    def update_from_render(self, render_output: TabBarRender) -> None:
        """Update widget from render output.

        Args:
            render_output: TabBarRender containing tab labels and active state
        """
        self._last_render = render_output

        # Format tabs with visual indicators
        parts = []
        for label, is_active in render_output.tabs:
            if is_active:
                parts.append(f"[bold reverse] {label} [/]")
            else:
                parts.append(f" {label} ")

        content = "│".join(parts)
        self.update(content)

    @property
    def active_index(self) -> int:
        """Get currently active tab index."""
        if self._last_render is None:
            return 0
        return self._last_render.active_index


class FooterWidget(Static):
    """Widget for displaying the footer with shortcuts.

    Shows keyboard shortcuts: q quit | y copy visual | Y copy state
    """

    DEFAULT_CSS = """
    FooterWidget {
        height: 1;
        background: $boost;
        padding: 0 1;
        text-style: dim;
    }
    """

    def __init__(
        self,
        content: str = "",
        *,
        name: str | None = None,
        id: str | None = None,  # noqa: A002
        classes: str | None = None,
        disabled: bool = False,
    ) -> None:
        """Initialize footer widget."""
        super().__init__(
            content,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )
        self._last_render: FooterRender | None = None

    def update_from_render(self, render_output: FooterRender) -> None:
        """Update widget from render output.

        Args:
            render_output: FooterRender containing shortcut descriptions
        """
        self._last_render = render_output

        # Format shortcuts with separator
        content = " | ".join(render_output.shortcuts)
        self.update(content)

    @property
    def shortcuts(self) -> list[str]:
        """Get current shortcuts."""
        if self._last_render is None:
            return []
        return self._last_render.shortcuts

"""Tests for TUI widgets."""

from __future__ import annotations

from rstn.tui.render import (
    CommandListRender,
    ContentAreaRender,
    StatusBarRender,
)
from rstn.tui.render.widgets import (
    CommandListWidget,
    ContentAreaWidget,
    StatusBarWidget,
)


class TestCommandListWidget:
    """Tests for CommandListWidget."""

    def test_widget_initialization(self) -> None:
        """Test widget can be initialized."""
        widget = CommandListWidget()
        assert widget is not None
        assert widget._last_render is None

    def test_widget_with_content(self) -> None:
        """Test widget with initial content."""
        widget = CommandListWidget(content="Test content")
        assert widget is not None

    def test_widget_with_id(self) -> None:
        """Test widget with ID."""
        widget = CommandListWidget(id="command-list")
        assert widget is not None

    def test_widget_with_classes(self) -> None:
        """Test widget with CSS classes."""
        widget = CommandListWidget(classes="my-class")
        assert widget is not None

    def test_selected_index_default(self) -> None:
        """Test selected index defaults to 0."""
        widget = CommandListWidget()
        assert widget.selected_index == 0

    def test_last_render_stores_value(self) -> None:
        """Test _last_render stores render object."""
        widget = CommandListWidget()
        render = CommandListRender(items=[], selected_index=0, has_commands=False)

        # Directly set _last_render to test the property
        widget._last_render = render

        assert widget._last_render == render
        assert widget.selected_index == 0

    def test_selected_index_from_render(self) -> None:
        """Test selected_index property uses render."""
        widget = CommandListWidget()
        render = CommandListRender(
            items=["  Command 1", "> Command 2", "  Command 3"],
            selected_index=1,
            has_commands=True,
        )

        # Directly set _last_render
        widget._last_render = render

        assert widget.selected_index == 1


class TestContentAreaWidget:
    """Tests for ContentAreaWidget."""

    def test_widget_initialization(self) -> None:
        """Test widget can be initialized."""
        widget = ContentAreaWidget()
        assert widget is not None
        assert widget._last_render is None

    def test_widget_with_content(self) -> None:
        """Test widget with initial content."""
        widget = ContentAreaWidget(content="Initial content")
        assert widget is not None

    def test_widget_with_id(self) -> None:
        """Test widget with ID."""
        widget = ContentAreaWidget(id="content-area")
        assert widget is not None

    def test_content_type_default(self) -> None:
        """Test content type defaults to 'empty'."""
        widget = ContentAreaWidget()
        assert widget.content_type == "empty"

    def test_title_default(self) -> None:
        """Test title defaults to 'Content'."""
        widget = ContentAreaWidget()
        assert widget.title == "Content"

    def test_last_render_stores_value(self) -> None:
        """Test _last_render stores render object."""
        widget = ContentAreaWidget()
        render = ContentAreaRender(
            title="My Title",
            content="Some content here",
            content_type="spec",
        )

        # Directly set _last_render to test properties
        widget._last_render = render

        assert widget._last_render == render
        assert widget.title == "My Title"
        assert widget.content_type == "spec"

    def test_content_type_from_render(self) -> None:
        """Test content_type property uses render."""
        widget = ContentAreaWidget()
        render = ContentAreaRender(
            title="Spec",
            content="# Heading\n\nContent",
            content_type="markdown",
        )

        # Directly set _last_render
        widget._last_render = render

        assert widget.content_type == "markdown"


class TestStatusBarWidget:
    """Tests for StatusBarWidget."""

    def test_widget_initialization(self) -> None:
        """Test widget can be initialized."""
        widget = StatusBarWidget()
        assert widget is not None
        assert widget._last_render is None

    def test_widget_with_content(self) -> None:
        """Test widget with initial content."""
        widget = StatusBarWidget(content="Status message")
        assert widget is not None

    def test_widget_with_id(self) -> None:
        """Test widget with ID."""
        widget = StatusBarWidget(id="status-bar")
        assert widget is not None

    def test_current_style_default(self) -> None:
        """Test current style defaults to 'normal'."""
        widget = StatusBarWidget()
        assert widget.current_style == "normal"

    def test_last_render_normal(self) -> None:
        """Test _last_render stores normal render."""
        widget = StatusBarWidget()
        render = StatusBarRender(
            message="Ready",
            style="normal",
        )

        # Directly set _last_render
        widget._last_render = render

        assert widget._last_render == render
        assert widget.current_style == "normal"

    def test_current_style_error(self) -> None:
        """Test current_style property for error."""
        widget = StatusBarWidget()
        render = StatusBarRender(
            message="Error occurred",
            style="error",
        )

        widget._last_render = render

        assert widget.current_style == "error"

    def test_current_style_warning(self) -> None:
        """Test current_style property for warning."""
        widget = StatusBarWidget()
        render = StatusBarRender(
            message="Warning",
            style="warning",
        )

        widget._last_render = render

        assert widget.current_style == "warning"

    def test_current_style_success(self) -> None:
        """Test current_style property for success."""
        widget = StatusBarWidget()
        render = StatusBarRender(
            message="Success!",
            style="success",
        )

        widget._last_render = render

        assert widget.current_style == "success"


class TestWidgetDefaultCSS:
    """Tests for widget CSS defaults."""

    def test_command_list_css(self) -> None:
        """Test CommandListWidget has CSS."""
        assert CommandListWidget.DEFAULT_CSS is not None
        assert "CommandListWidget" in CommandListWidget.DEFAULT_CSS

    def test_content_area_css(self) -> None:
        """Test ContentAreaWidget has CSS."""
        assert ContentAreaWidget.DEFAULT_CSS is not None
        assert "ContentAreaWidget" in ContentAreaWidget.DEFAULT_CSS

    def test_status_bar_css(self) -> None:
        """Test StatusBarWidget has CSS."""
        assert StatusBarWidget.DEFAULT_CSS is not None
        assert "StatusBarWidget" in StatusBarWidget.DEFAULT_CSS
        # Should have style classes
        assert "error" in StatusBarWidget.DEFAULT_CSS
        assert "warning" in StatusBarWidget.DEFAULT_CSS
        assert "success" in StatusBarWidget.DEFAULT_CSS

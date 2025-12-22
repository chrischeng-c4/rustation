"""Tests for content formatting helpers."""

from __future__ import annotations

from rstn.tui.render.content import (
    format_help_content,
    format_log_content,
    format_plan_content,
    format_spec_content,
    format_workflow_output,
    truncate_with_ellipsis,
)


class TestFormatSpecContent:
    """Test format_spec_content()."""

    def test_none_content(self) -> None:
        """None returns placeholder."""
        result = format_spec_content(None)
        assert "No specification loaded" in result

    def test_empty_content(self) -> None:
        """Empty string returns placeholder."""
        result = format_spec_content("")
        assert "No specification loaded" in result

    def test_content_stripped(self) -> None:
        """Content is stripped of leading/trailing whitespace."""
        result = format_spec_content("  \nContent\n  ")
        assert result == "Content"


class TestFormatPlanContent:
    """Test format_plan_content()."""

    def test_none_content(self) -> None:
        """None returns placeholder."""
        result = format_plan_content(None)
        assert "No plan loaded" in result

    def test_content(self) -> None:
        """Content is returned stripped."""
        result = format_plan_content("# Plan\n\n1. Step 1")
        assert "Plan" in result
        assert "Step 1" in result


class TestFormatWorkflowOutput:
    """Test format_workflow_output()."""

    def test_empty_output(self) -> None:
        """Empty output returns placeholder."""
        result = format_workflow_output("")
        assert "running" in result.lower()

    def test_output(self) -> None:
        """Output is returned as-is."""
        result = format_workflow_output("Processing...")
        assert result == "Processing..."


class TestFormatLogContent:
    """Test format_log_content()."""

    def test_empty_logs(self) -> None:
        """Empty logs returns placeholder."""
        result = format_log_content([])
        assert "No log messages" in result

    def test_logs_joined(self) -> None:
        """Logs are joined with newlines."""
        result = format_log_content(["Line 1", "Line 2"])
        assert "Line 1" in result
        assert "Line 2" in result
        assert "\n" in result

    def test_max_lines(self) -> None:
        """Only max_lines are included."""
        logs = [f"Line {i}" for i in range(100)]
        result = format_log_content(logs, max_lines=5)
        lines = result.split("\n")
        assert len(lines) == 5


class TestFormatHelpContent:
    """Test format_help_content()."""

    def test_has_navigation(self) -> None:
        """Help content includes navigation info."""
        result = format_help_content()
        assert "Navigation" in result
        assert "j/k" in result

    def test_has_keybindings(self) -> None:
        """Help content includes keybindings."""
        result = format_help_content()
        assert "q" in result
        assert "Quit" in result


class TestTruncateWithEllipsis:
    """Test truncate_with_ellipsis()."""

    def test_short_text(self) -> None:
        """Short text is not truncated."""
        result = truncate_with_ellipsis("hello", 10)
        assert result == "hello"

    def test_exact_length(self) -> None:
        """Text at exact length is not truncated."""
        result = truncate_with_ellipsis("hello", 5)
        assert result == "hello"

    def test_long_text(self) -> None:
        """Long text is truncated with ellipsis."""
        result = truncate_with_ellipsis("hello world", 8)
        assert result == "hello..."
        assert len(result) == 8

    def test_minimum_length(self) -> None:
        """Max length is enforced to at least 4."""
        result = truncate_with_ellipsis("hello", 2)
        # Should be clamped to 4
        assert result == "h..."

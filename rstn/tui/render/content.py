"""Content formatting helpers for rendering.

Pure functions for formatting different content types.
"""

from __future__ import annotations

__all__ = [
    "format_spec_content",
    "format_plan_content",
    "format_workflow_output",
    "format_log_content",
    "format_help_content",
    "truncate_with_ellipsis",
]


def format_spec_content(content: str | None) -> str:
    """Format specification content for display.

    Args:
        content: Raw specification markdown content

    Returns:
        Formatted content string, or placeholder if None
    """
    if not content:
        return "[No specification loaded]"
    return content.strip()


def format_plan_content(content: str | None) -> str:
    """Format plan content for display.

    Args:
        content: Raw plan markdown content

    Returns:
        Formatted content string, or placeholder if None
    """
    if not content:
        return "[No plan loaded]"
    return content.strip()


def format_workflow_output(output: str) -> str:
    """Format streaming workflow output.

    Args:
        output: Current workflow output buffer

    Returns:
        Formatted output string
    """
    if not output:
        return "[Workflow running...]"
    return output


def format_log_content(logs: list[str], max_lines: int = 50) -> str:
    """Format log messages for display.

    Args:
        logs: List of log messages (newest first)
        max_lines: Maximum number of lines to display

    Returns:
        Formatted log content with timestamps
    """
    if not logs:
        return "[No log messages]"

    # Take most recent logs up to max_lines
    recent = logs[:max_lines]
    return "\n".join(recent)


def format_help_content() -> str:
    """Generate help content.

    Returns:
        Help text with keybindings and usage info
    """
    return """
rstn TUI Help
=============

Navigation:
  j/k     - Move up/down in command list
  1/2/3   - Switch to Worktree/Dashboard/Settings view
  q       - Quit

Commands:
  Enter   - Execute selected command
  Esc     - Cancel current operation

For more info, see: docs/manual/cli/commands.md
""".strip()


def truncate_with_ellipsis(text: str, max_length: int) -> str:
    """Truncate text with ellipsis if too long.

    Args:
        text: Text to truncate
        max_length: Maximum length (must be >= 4 for "...")

    Returns:
        Truncated text with "..." if needed
    """
    if max_length < 4:
        max_length = 4

    if len(text) <= max_length:
        return text

    return text[: max_length - 3] + "..."

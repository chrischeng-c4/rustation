"""Core state types for rstn.

Basic types used across different state modules.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field

# Type alias for workflow identifiers
WorkflowId = str


class ViewType(str, Enum):
    """Application view types.

    Represents which view is currently active in the TUI.
    """

    WORKTREE = "worktree"
    DASHBOARD = "dashboard"
    SETTINGS = "settings"


class UiState(BaseModel):
    """UI-specific state (not business logic).

    Contains transient UI state like scroll positions and selections.
    """

    model_config = {"frozen": False}

    scroll_offset: int = Field(default=0, ge=0, description="Vertical scroll offset")
    selected_index: int | None = Field(default=None, description="Currently selected item index")
    cursor_position: int = Field(default=0, ge=0, description="Cursor position in input")

    def with_scroll(self, offset: int) -> UiState:
        """Create new state with updated scroll offset.

        Args:
            offset: New scroll offset (will be clamped to >= 0)

        Returns:
            New UiState with updated scroll
        """
        return self.model_copy(update={"scroll_offset": max(0, offset)})

    def with_selection(self, index: int | None) -> UiState:
        """Create new state with updated selection.

        Args:
            index: New selected index (None to clear selection)

        Returns:
            New UiState with updated selection
        """
        return self.model_copy(update={"selected_index": index})

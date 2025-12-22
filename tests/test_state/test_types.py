"""Tests for core state types."""

from __future__ import annotations

import pytest
from rstn.state.types import UiState, ViewType


class TestViewType:
    """Test ViewType enum."""

    def test_view_type_values(self) -> None:
        """ViewType has expected values."""
        assert ViewType.WORKTREE == "worktree"  # type: ignore
        assert ViewType.DASHBOARD == "dashboard"  # type: ignore
        assert ViewType.SETTINGS == "settings"  # type: ignore

    def test_view_type_serialization(self) -> None:
        """ViewType can be serialized."""
        assert ViewType.WORKTREE.value == "worktree"
        assert ViewType("dashboard") == ViewType.DASHBOARD


class TestUiState:
    """Test UiState."""

    def test_default_ui_state(self) -> None:
        """UiState has correct defaults."""
        state = UiState()

        assert state.scroll_offset == 0
        assert state.selected_index is None
        assert state.cursor_position == 0

    def test_ui_state_with_values(self) -> None:
        """UiState can be created with custom values."""
        state = UiState(scroll_offset=10, selected_index=5, cursor_position=20)

        assert state.scroll_offset == 10
        assert state.selected_index == 5
        assert state.cursor_position == 20

    def test_with_scroll(self) -> None:
        """with_scroll updates scroll offset."""
        state = UiState()
        new_state = state.with_scroll(15)

        # Original unchanged
        assert state.scroll_offset == 0
        # New state updated
        assert new_state.scroll_offset == 15

    def test_with_scroll_negative_clamped(self) -> None:
        """Negative scroll offset is clamped to 0."""
        state = UiState()
        new_state = state.with_scroll(-5)

        assert new_state.scroll_offset == 0

    def test_with_selection(self) -> None:
        """with_selection updates selected index."""
        state = UiState()
        new_state = state.with_selection(3)

        assert state.selected_index is None
        assert new_state.selected_index == 3

    def test_with_selection_clear(self) -> None:
        """with_selection can clear selection."""
        state = UiState(selected_index=5)
        new_state = state.with_selection(None)

        assert state.selected_index == 5
        assert new_state.selected_index is None

    def test_ui_state_serialization(self) -> None:
        """UiState can be serialized/deserialized."""
        state = UiState(scroll_offset=10, selected_index=3, cursor_position=15)

        # Serialize
        data = state.model_dump()
        assert data["scroll_offset"] == 10
        assert data["selected_index"] == 3

        # Deserialize
        loaded = UiState.model_validate(data)
        assert loaded == state

    def test_ui_state_validation(self) -> None:
        """UiState validates field constraints."""
        # Negative scroll_offset should fail validation
        with pytest.raises(ValueError):
            UiState(scroll_offset=-1)

        # Negative cursor_position should fail validation
        with pytest.raises(ValueError):
            UiState(cursor_position=-1)

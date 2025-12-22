"""Tests for reduce function and reducers."""

from __future__ import annotations

from rstn.effect import LogInfo, QuitApp, Render
from rstn.msg import (
    KeyModifiers,
    KeyPressed,
    Noop,
    Quit,
    SelectCommand,
    SwitchView,
    Tick,
    WorkflowCompleted,
)
from rstn.reduce import reduce
from rstn.state import AppState, Command, ViewType


class TestReduceDispatcher:
    """Test reduce() function message dispatching."""

    def test_reduce_noop(self) -> None:
        """reduce() handles Noop message."""
        state = AppState()
        new_state, effects = reduce(state, Noop())

        assert new_state == state
        assert effects == []

    def test_reduce_unknown_message(self) -> None:
        """reduce() handles unknown message types."""
        state = AppState()

        # WorkflowCompleted is not yet handled in basic reducers
        new_state, effects = reduce(state, WorkflowCompleted(workflow_id="wf-123"))

        assert new_state == state
        assert len(effects) == 1
        assert isinstance(effects[0], LogInfo)
        assert "Unknown message type" in effects[0].message


class TestReduceQuit:
    """Test quit reducer."""

    def test_reduce_quit(self) -> None:
        """reduce_quit() stops the application."""
        state = AppState(running=True)
        new_state, effects = reduce(state, Quit())

        assert state.running is True
        assert new_state.running is False
        assert len(effects) == 2
        assert isinstance(effects[0], QuitApp)
        assert isinstance(effects[1], LogInfo)

    def test_reduce_quit_idempotent(self) -> None:
        """Quitting when already not running."""
        state = AppState(running=False)
        new_state, effects = reduce(state, Quit())

        assert new_state.running is False
        assert len(effects) == 2


class TestReduceKeyPressed:
    """Test key press reducer."""

    def test_reduce_ctrl_c_quits(self) -> None:
        """Ctrl+C quits the application."""
        state = AppState(running=True)
        msg = KeyPressed(key="c", modifiers=KeyModifiers.ctrl_key())
        new_state, effects = reduce(state, msg)

        assert new_state.running is False
        assert len(effects) == 2
        assert isinstance(effects[0], QuitApp)

    def test_reduce_key_1_switches_to_worktree(self) -> None:
        """Key '1' switches to worktree view."""
        state = AppState(current_view=ViewType.DASHBOARD)
        msg = KeyPressed(key="1", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.current_view == ViewType.WORKTREE
        assert len(effects) == 2
        assert isinstance(effects[0], Render)
        assert isinstance(effects[1], LogInfo)

    def test_reduce_key_2_switches_to_dashboard(self) -> None:
        """Key '2' switches to dashboard view."""
        state = AppState(current_view=ViewType.WORKTREE)
        msg = KeyPressed(key="2", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.current_view == ViewType.DASHBOARD
        assert len(effects) == 2

    def test_reduce_key_3_switches_to_settings(self) -> None:
        """Key '3' switches to settings view."""
        state = AppState(current_view=ViewType.WORKTREE)
        msg = KeyPressed(key="3", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.current_view == ViewType.SETTINGS
        assert len(effects) == 2

    def test_reduce_key_j_selects_next_command(self) -> None:
        """Key 'j' selects next command (vim-style down)."""
        # Setup state with commands
        commands = [
            Command(id="cmd1", label="Command 1"),
            Command(id="cmd2", label="Command 2"),
            Command(id="cmd3", label="Command 3"),
        ]
        worktree = AppState().worktree_view.model_copy(
            update={"commands": commands, "selected_command_index": 0}
        )
        state = AppState(worktree_view=worktree)

        msg = KeyPressed(key="j", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.worktree_view.selected_command_index == 1
        assert len(effects) == 1
        assert isinstance(effects[0], Render)

    def test_reduce_key_j_at_bottom(self) -> None:
        """Key 'j' at bottom stays at bottom."""
        commands = [
            Command(id="cmd1", label="Command 1"),
            Command(id="cmd2", label="Command 2"),
        ]
        worktree = AppState().worktree_view.model_copy(
            update={"commands": commands, "selected_command_index": 1}
        )
        state = AppState(worktree_view=worktree)

        msg = KeyPressed(key="j", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        # Should stay at index 1 (last item)
        assert new_state.worktree_view.selected_command_index == 1

    def test_reduce_key_k_selects_previous_command(self) -> None:
        """Key 'k' selects previous command (vim-style up)."""
        commands = [
            Command(id="cmd1", label="Command 1"),
            Command(id="cmd2", label="Command 2"),
        ]
        worktree = AppState().worktree_view.model_copy(
            update={"commands": commands, "selected_command_index": 1}
        )
        state = AppState(worktree_view=worktree)

        msg = KeyPressed(key="k", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.worktree_view.selected_command_index == 0
        assert len(effects) == 1

    def test_reduce_key_k_at_top(self) -> None:
        """Key 'k' at top stays at top."""
        commands = [Command(id="cmd1", label="Command 1")]
        worktree = AppState().worktree_view.model_copy(
            update={"commands": commands, "selected_command_index": 0}
        )
        state = AppState(worktree_view=worktree)

        msg = KeyPressed(key="k", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state.worktree_view.selected_command_index == 0

    def test_reduce_unknown_key(self) -> None:
        """Unknown keys are ignored."""
        state = AppState()
        msg = KeyPressed(key="x", modifiers=KeyModifiers())
        new_state, effects = reduce(state, msg)

        assert new_state == state
        assert effects == []

    def test_reduce_key_with_modifier(self) -> None:
        """Keys with modifiers (other than Ctrl+C) are ignored."""
        state = AppState()
        msg = KeyPressed(key="a", modifiers=KeyModifiers.shift_key())
        new_state, effects = reduce(state, msg)

        assert new_state == state
        assert effects == []


class TestReduceSwitchView:
    """Test view switching reducer."""

    def test_reduce_switch_view(self) -> None:
        """reduce_switch_view() switches views."""
        state = AppState(current_view=ViewType.WORKTREE)
        msg = SwitchView(view=ViewType.DASHBOARD)
        new_state, effects = reduce(state, msg)

        assert new_state.current_view == ViewType.DASHBOARD
        assert len(effects) == 2
        assert isinstance(effects[0], Render)
        assert isinstance(effects[1], LogInfo)

    def test_reduce_switch_view_same_view(self) -> None:
        """Switching to current view does nothing."""
        state = AppState(current_view=ViewType.DASHBOARD)
        msg = SwitchView(view=ViewType.DASHBOARD)
        new_state, effects = reduce(state, msg)

        assert new_state == state
        assert effects == []


class TestReduceSelectCommand:
    """Test command selection reducer."""

    def test_reduce_select_command(self) -> None:
        """reduce_select_command() selects command."""
        commands = [
            Command(id="cmd1", label="Command 1"),
            Command(id="cmd2", label="Command 2"),
        ]
        worktree = AppState().worktree_view.model_copy(
            update={"commands": commands, "selected_command_index": 0}
        )
        state = AppState(worktree_view=worktree)

        msg = SelectCommand(index=1)
        new_state, effects = reduce(state, msg)

        assert new_state.worktree_view.selected_command_index == 1
        assert len(effects) == 1
        assert isinstance(effects[0], Render)

    def test_reduce_select_command_clamps_to_max(self) -> None:
        """Select command clamps to max index."""
        commands = [Command(id="cmd1", label="Command 1")]
        worktree = AppState().worktree_view.model_copy(update={"commands": commands})
        state = AppState(worktree_view=worktree)

        msg = SelectCommand(index=999)
        new_state, effects = reduce(state, msg)

        # Should clamp to 0 (last valid index)
        assert new_state.worktree_view.selected_command_index == 0

    def test_reduce_select_command_no_commands(self) -> None:
        """Select command with no commands does nothing."""
        state = AppState()  # No commands by default

        msg = SelectCommand(index=0)
        new_state, effects = reduce(state, msg)

        assert new_state == state
        assert effects == []


class TestReduceTick:
    """Test tick reducer."""

    def test_reduce_tick(self) -> None:
        """reduce_tick() returns state unchanged."""
        state = AppState()
        msg = Tick()
        new_state, effects = reduce(state, msg)

        assert new_state == state
        assert effects == []


class TestReducePurity:
    """Test that reducers are pure functions."""

    def test_reduce_does_not_mutate_state(self) -> None:
        """reduce() does not mutate original state."""
        state = AppState(running=True, current_view=ViewType.WORKTREE)
        original_running = state.running
        original_view = state.current_view

        # Call reduce with a state-changing message
        msg = SwitchView(view=ViewType.DASHBOARD)
        new_state, _ = reduce(state, msg)

        # Original state unchanged
        assert state.running == original_running
        assert state.current_view == original_view

        # New state has changes
        assert new_state.current_view == ViewType.DASHBOARD

    def test_reduce_returns_new_state_instance(self) -> None:
        """reduce() returns a new state instance."""
        state = AppState()
        msg = SwitchView(view=ViewType.DASHBOARD)
        new_state, _ = reduce(state, msg)

        # Different instances when state changes
        assert new_state is not state

    def test_reduce_returns_same_state_when_no_change(self) -> None:
        """reduce() returns same state when no change."""
        state = AppState()
        msg = Noop()
        new_state, _ = reduce(state, msg)

        # Same instance when no change
        assert new_state is state

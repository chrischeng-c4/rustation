"""Reducer for rstn v2 MVI architecture.

reduce(state, msg) -> (new_state, effects)

The reducer is a pure function that takes the current state and a message,
and returns the new state and any effects to execute.

CRITICAL: Reducers must be pure functions:
- No I/O
- No async
- No reading current time
- No random numbers
- Only return new state + effects
"""

from __future__ import annotations

from rstn.effect import AppEffect, LogInfo, Render
from rstn.msg import AppMsg, KeyPressed, Noop, Quit, SelectCommand, SwitchView, Tick
from rstn.state import AppState


def reduce(state: AppState, msg: AppMsg) -> tuple[AppState, list[AppEffect]]:
    """Main reducer function.

    Takes current state and a message, returns new state and effects.

    Args:
        state: Current application state
        msg: Message to process

    Returns:
        Tuple of (new_state, effects)

    Examples:
        >>> state = AppState()
        >>> new_state, effects = reduce(state, Quit())
        >>> assert not new_state.running
    """
    # Dispatch to specific reducers based on message type
    if isinstance(msg, KeyPressed):
        return reduce_key_pressed(state, msg)
    elif isinstance(msg, SwitchView):
        return reduce_switch_view(state, msg)
    elif isinstance(msg, SelectCommand):
        return reduce_select_command(state, msg)
    elif isinstance(msg, Tick):
        return reduce_tick(state, msg)
    elif isinstance(msg, Quit):
        return reduce_quit(state, msg)
    elif isinstance(msg, Noop):
        return state, []
    else:
        # Unknown message type - log and ignore
        return state, [LogInfo(message=f"Unknown message type: {type(msg).__name__}")]


# ========================================
# Basic Reducers
# ========================================


def reduce_key_pressed(state: AppState, msg: KeyPressed) -> tuple[AppState, list[AppEffect]]:
    """Handle key pressed events.

    Args:
        state: Current state
        msg: KeyPressed message

    Returns:
        Tuple of (new_state, effects)
    """
    # Handle Ctrl+C to quit
    if msg.key == "c" and msg.modifiers.ctrl:
        return reduce_quit(state, Quit())

    # Handle view switching
    if msg.key == "1" and msg.modifiers.is_empty():
        from rstn.state.types import ViewType

        return reduce_switch_view(state, SwitchView(view=ViewType.WORKTREE))
    elif msg.key == "2" and msg.modifiers.is_empty():
        from rstn.state.types import ViewType

        return reduce_switch_view(state, SwitchView(view=ViewType.DASHBOARD))
    elif msg.key == "3" and msg.modifiers.is_empty():
        from rstn.state.types import ViewType

        return reduce_switch_view(state, SwitchView(view=ViewType.SETTINGS))

    # Handle command selection (j/k for vim-like navigation)
    if msg.key == "j" and msg.modifiers.is_empty():
        current_idx = state.worktree_view.selected_command_index
        new_idx = min(current_idx + 1, len(state.worktree_view.commands) - 1)
        return reduce_select_command(state, SelectCommand(index=new_idx))
    elif msg.key == "k" and msg.modifiers.is_empty():
        current_idx = state.worktree_view.selected_command_index
        new_idx = max(current_idx - 1, 0)
        return reduce_select_command(state, SelectCommand(index=new_idx))

    # Default: no state change
    return state, []


def reduce_switch_view(state: AppState, msg: SwitchView) -> tuple[AppState, list[AppEffect]]:
    """Handle view switching.

    Args:
        state: Current state
        msg: SwitchView message

    Returns:
        Tuple of (new_state, effects)
    """
    if state.current_view == msg.view:
        # Already on this view, no change
        return state, []

    new_state = state.model_copy(update={"current_view": msg.view})
    return new_state, [Render(), LogInfo(message=f"Switched to {msg.view} view")]


def reduce_select_command(
    state: AppState, msg: SelectCommand
) -> tuple[AppState, list[AppEffect]]:
    """Handle command selection.

    Args:
        state: Current state
        msg: SelectCommand message

    Returns:
        Tuple of (new_state, effects)
    """
    if not state.worktree_view.commands:
        # No commands available
        return state, []

    # Clamp index to valid range
    index = max(0, min(msg.index, len(state.worktree_view.commands) - 1))

    new_worktree = state.worktree_view.select_command(index)
    new_state = state.model_copy(update={"worktree_view": new_worktree})

    return new_state, [Render()]


def reduce_tick(state: AppState, msg: Tick) -> tuple[AppState, list[AppEffect]]:
    """Handle tick events.

    Args:
        state: Current state
        msg: Tick message

    Returns:
        Tuple of (new_state, effects)
    """
    # Tick events can be used for animations, polling, etc.
    # For now, just return state unchanged
    return state, []


def reduce_quit(state: AppState, msg: Quit) -> tuple[AppState, list[AppEffect]]:
    """Handle quit request.

    Args:
        state: Current state
        msg: Quit message

    Returns:
        Tuple of (new_state, effects)
    """
    from rstn.effect import QuitApp

    new_state = state.model_copy(update={"running": False})
    return new_state, [QuitApp(), LogInfo(message="Quitting application")]


# Export main reduce function
__all__ = ["reduce"]

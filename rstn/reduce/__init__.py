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

from rstn.effect import AppEffect, CopyToClipboard, LogInfo, Render
from rstn.msg import (
    AppMsg,
    ClaudeCompleted,
    ClaudeStreamDelta,
    CopyContentRequested,
    CopyStateRequested,
    KeyPressed,
    McpCompleteTaskReceived,
    McpReportStatusReceived,
    McpServerStarted,
    McpServerStopped,
    Noop,
    Quit,
    ScrollContent,
    SelectCommand,
    SwitchView,
    Tick,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowStartRequested,
)
from rstn.reduce.workflow import reduce_workflow, reduce_workflow_start
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
    from rstn.logging import get_logger

    log = get_logger("rstn.reduce")
    log.debug("Processing message", msg_type=type(msg).__name__, msg_data=msg.model_dump())

    # Dispatch to specific reducers based on message type
    if isinstance(msg, KeyPressed):
        return reduce_key_pressed(state, msg)
    elif isinstance(msg, SwitchView):
        return reduce_switch_view(state, msg)
    elif isinstance(msg, SelectCommand):
        return reduce_select_command(state, msg)
    elif isinstance(msg, ScrollContent):
        return reduce_scroll_content(state, msg)
    elif isinstance(msg, CopyContentRequested):
        return reduce_copy_content(state, msg)
    elif isinstance(msg, CopyStateRequested):
        return reduce_copy_state(state, msg)
    elif isinstance(
        msg,
        (
            WorkflowStartRequested,
            ClaudeStreamDelta,
            ClaudeCompleted,
            WorkflowCompleted,
            WorkflowFailed,
        ),
    ):
        return reduce_workflow(state, msg)
    elif isinstance(msg, Tick):
        return reduce_tick(state, msg)
    elif isinstance(msg, Quit):
        return reduce_quit(state, msg)
    elif isinstance(msg, Noop):
        return state, []
    # MCP Events
    elif isinstance(msg, McpServerStarted):
        return reduce_mcp_server_started(state, msg)
    elif isinstance(msg, McpServerStopped):
        return reduce_mcp_server_stopped(state, msg)
    elif isinstance(msg, McpReportStatusReceived):
        return reduce_mcp_report_status(state, msg)
    elif isinstance(msg, McpCompleteTaskReceived):
        return reduce_mcp_complete_task(state, msg)
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
    from rstn.state.types import ViewType

    key = msg.key
    mods = msg.modifiers

    # ========================================
    # Input Mode Handling
    # ========================================
    if state.worktree_view.input_mode:
        if key == "enter" and mods.is_empty():
            # Submit input
            prompt = state.worktree_view.input_buffer
            if not prompt.strip():
                return state, []

            import uuid

            workflow_id = str(uuid.uuid4())
            new_worktree = state.worktree_view.exit_input_mode()
            new_state = state.model_copy(update={"worktree_view": new_worktree})

            return reduce_workflow_start(
                new_state,
                WorkflowStartRequested(
                    workflow_id=workflow_id,
                    workflow_type="prompt-claude",
                    params=prompt,
                ),
            )
        elif key == "esc" and mods.is_empty():
            # Cancel input mode
            new_worktree = state.worktree_view.exit_input_mode()
            new_state = state.model_copy(update={"worktree_view": new_worktree})
            return new_state, [Render()]
        elif key == "backspace" and mods.is_empty():
            # Remove last character
            new_buffer = state.worktree_view.input_buffer[:-1]
            new_worktree = state.worktree_view.model_copy(update={"input_buffer": new_buffer})
            new_state = state.model_copy(update={"worktree_view": new_worktree})
            return new_state, [Render()]
        elif len(key) == 1 and mods.is_empty():
            # Append character
            new_buffer = state.worktree_view.input_buffer + key
            new_worktree = state.worktree_view.model_copy(update={"input_buffer": new_buffer})
            new_state = state.model_copy(update={"worktree_view": new_worktree})
            return new_state, [Render()]

        # Ignore other keys in input mode
        return state, []

    # ========================================
    # Global Actions (any context)
    # ========================================

    # Quit: q or Ctrl+C
    if key == "q" and mods.is_empty():
        return reduce_quit(state, Quit())
    if key == "c" and mods.ctrl:
        return reduce_quit(state, Quit())

    # Copy Visual Content: y
    if key == "y" and mods.is_empty():
        return reduce_copy_content(state, CopyContentRequested())

    # Copy Full State: Y (shift+y)
    if key == "Y" or (key == "y" and mods.shift):
        return reduce_copy_state(state, CopyStateRequested())

    # ========================================
    # Navigation (Tab Bar / View Switching)
    # ========================================

    if key == "1" and mods.is_empty():
        return reduce_switch_view(state, SwitchView(view=ViewType.WORKTREE))
    if key == "2" and mods.is_empty():
        return reduce_switch_view(state, SwitchView(view=ViewType.DASHBOARD))
    if key == "3" and mods.is_empty():
        return reduce_switch_view(state, SwitchView(view=ViewType.SETTINGS))

    # ========================================
    # Worktree Context
    # ========================================

    if state.current_view == ViewType.WORKTREE:
        # j/k: Move selection up/down
        if key == "j" and mods.is_empty():
            if not state.worktree_view.commands:
                return state, []
            current_idx = state.worktree_view.selected_command_index
            new_idx = min(current_idx + 1, len(state.worktree_view.commands) - 1)
            return reduce_select_command(state, SelectCommand(index=new_idx))

        if key == "k" and mods.is_empty():
            if not state.worktree_view.commands:
                return state, []
            current_idx = state.worktree_view.selected_command_index
            new_idx = max(current_idx - 1, 0)
            return reduce_select_command(state, SelectCommand(index=new_idx))

        # Tab: Switch focus between Sidebar and Content
        if key == "tab" and mods.is_empty():
            # Toggle focus between sidebar and content
            new_ui = state.ui_state.model_copy(
                update={
                    "selected_index": 1 if state.ui_state.selected_index == 0 else 0
                }
            )
            new_state = state.model_copy(update={"ui_state": new_ui})
            return new_state, [Render()]

        # Enter: Execute selected command
        if key == "enter" and mods.is_empty() and state.worktree_view.commands:
            selected = state.worktree_view.commands[
                state.worktree_view.selected_command_index
            ]

            # If it's prompt-claude, enter input mode first
            if selected.id == "prompt-claude":
                new_worktree = state.worktree_view.enter_input_mode(
                    prompt="Enter prompt for Claude:"
                )
                new_state = state.model_copy(update={"worktree_view": new_worktree})
                return new_state, [Render()]

            # Default: execute workflow directly if applicable
            if selected.workflow_type:
                import uuid

                workflow_id = str(uuid.uuid4())
                return reduce_workflow_start(
                    state,
                    WorkflowStartRequested(
                        workflow_id=workflow_id,
                        workflow_type=selected.workflow_type,
                        params="{}",  # Default empty params
                    ),
                )

            return state, [LogInfo(message=f"Execute command: {selected.label}")]

        # h/l: Scroll content left/right
        if key == "h" and mods.is_empty():
            return reduce_scroll_content(state, ScrollContent(delta=-1))

        if key == "l" and mods.is_empty():
            return reduce_scroll_content(state, ScrollContent(delta=1))

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


def reduce_scroll_content(
    state: AppState, msg: ScrollContent
) -> tuple[AppState, list[AppEffect]]:
    """Handle content scrolling.

    Args:
        state: Current state
        msg: ScrollContent message

    Returns:
        Tuple of (new_state, effects)
    """
    new_scroll = max(0, state.worktree_view.content_scroll + msg.delta)
    new_worktree = state.worktree_view.model_copy(
        update={"content_scroll": new_scroll}
    )
    new_state = state.model_copy(update={"worktree_view": new_worktree})
    return new_state, [Render()]


def reduce_copy_content(
    state: AppState, msg: CopyContentRequested
) -> tuple[AppState, list[AppEffect]]:
    """Handle copy visual content request.

    Copies the current view's visible content to clipboard.

    Args:
        state: Current state
        msg: CopyContentRequested message

    Returns:
        Tuple of (new_state, effects)
    """
    from rstn.state.types import ViewType

    # Get content based on current view
    content = ""
    if state.current_view == ViewType.WORKTREE:
        # Copy the worktree content (spec, plan, or workflow output)
        worktree = state.worktree_view
        if worktree.spec_content:
            content = worktree.spec_content
        elif worktree.plan_content:
            content = worktree.plan_content
        elif worktree.workflow_output:
            content = worktree.workflow_output
        else:
            content = f"Worktree: {len(worktree.commands)} commands"
    elif state.current_view == ViewType.DASHBOARD:
        content = "Dashboard view content"
    elif state.current_view == ViewType.SETTINGS:
        content = "Settings view content"

    if not content:
        return state, [LogInfo(message="No content to copy")]

    return state, [
        CopyToClipboard(content=content),
        LogInfo(message="Copied view content to clipboard"),
    ]


def reduce_copy_state(
    state: AppState, msg: CopyStateRequested
) -> tuple[AppState, list[AppEffect]]:
    """Handle copy full state request.

    Copies the entire application state as JSON to clipboard.

    Args:
        state: Current state
        msg: CopyStateRequested message

    Returns:
        Tuple of (new_state, effects)
    """
    # Serialize state to JSON
    state_json = state.model_dump_json(indent=2)

    return state, [
        CopyToClipboard(content=state_json),
        LogInfo(message="Copied full state JSON to clipboard"),
    ]


# ========================================
# MCP Reducers
# ========================================


def reduce_mcp_server_started(
    state: AppState, msg: McpServerStarted
) -> tuple[AppState, list[AppEffect]]:
    """Handle MCP server started event.

    Updates status message to show MCP server is available.

    Args:
        state: Current state
        msg: McpServerStarted message

    Returns:
        Tuple of (new_state, effects)
    """
    # Update worktree status to show MCP is ready
    new_worktree = state.worktree_view.model_copy(
        update={
            "status_message": f"MCP server ready on port {msg.port}",
        }
    )
    new_state = state.model_copy(update={"worktree_view": new_worktree})

    return new_state, [
        Render(),
        LogInfo(message=f"MCP server started on port {msg.port} (session: {msg.session_id})"),
    ]


def reduce_mcp_server_stopped(
    state: AppState, msg: McpServerStopped
) -> tuple[AppState, list[AppEffect]]:
    """Handle MCP server stopped event.

    Args:
        state: Current state
        msg: McpServerStopped message

    Returns:
        Tuple of (new_state, effects)
    """
    return state, [LogInfo(message="MCP server stopped")]


def reduce_mcp_report_status(
    state: AppState, msg: McpReportStatusReceived
) -> tuple[AppState, list[AppEffect]]:
    """Handle MCP status report from Claude Code.

    Status types:
    - needs_input: Claude needs user input, enter input mode with prompt
    - completed: Task completed successfully
    - error: Task failed with error message

    Args:
        state: Current state
        msg: McpReportStatusReceived message

    Returns:
        Tuple of (new_state, effects)
    """
    if msg.status == "needs_input":
        # Enter input mode with the prompt from Claude
        prompt = msg.prompt or "Input requested by Claude:"
        new_worktree = state.worktree_view.enter_input_mode(prompt=prompt)
        new_state = state.model_copy(update={"worktree_view": new_worktree})
        return new_state, [Render()]

    elif msg.status == "completed":
        # Update status message
        new_worktree = state.worktree_view.model_copy(
            update={"status_message": "Task completed"}
        )
        new_state = state.model_copy(update={"worktree_view": new_worktree})
        return new_state, [
            Render(),
            LogInfo(message="Claude reported task completed"),
        ]

    elif msg.status == "error":
        # Show error in status
        error_msg = msg.message or "Unknown error"
        new_worktree = state.worktree_view.model_copy(
            update={"status_message": f"Error: {error_msg}"}
        )
        new_state = state.model_copy(update={"worktree_view": new_worktree})
        return new_state, [
            Render(),
            LogInfo(message=f"Claude reported error: {error_msg}"),
        ]

    # Unknown status - just log
    return state, [LogInfo(message=f"Unknown MCP status: {msg.status}")]


def reduce_mcp_complete_task(
    state: AppState, msg: McpCompleteTaskReceived
) -> tuple[AppState, list[AppEffect]]:
    """Handle MCP task completion request from Claude Code.

    Args:
        state: Current state
        msg: McpCompleteTaskReceived message

    Returns:
        Tuple of (new_state, effects)
    """
    # For now, just log the task completion
    # In a full implementation, this would update task status in tasks.md
    return state, [
        LogInfo(message=f"Task {msg.task_id} marked complete (skip_validation={msg.skip_validation})"),
    ]


# Export main reduce function
__all__ = ["reduce"]

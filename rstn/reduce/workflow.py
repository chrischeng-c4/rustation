"""Reducer for workflow-related messages.

Handles starting, updating, and completing workflows.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.service.session_config import SessionConfigManager
from rstn.effect import AppEffect, LogInfo, Render, RunClaudeCli
from rstn.msg import (
    AppMsg,
    ClaudeCompleted,
    ClaudeStreamDelta,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowStartRequested,
)
from rstn.resources import get_system_prompt_path
from rstn.state import AppState
from rstn.state.workflow import WorkflowState, WorkflowStatus
from rstn.state.workflows.prompt import PromptClaudeData


def reduce_workflow(state: AppState, msg: AppMsg) -> tuple[AppState, list[AppEffect]]:
    """Reducer for workflow messages.

    Args:
        state: Current state
        msg: Workflow message

    Returns:
        Tuple of (new_state, effects)
    """
    if isinstance(msg, WorkflowStartRequested):
        return reduce_workflow_start(state, msg)
    elif isinstance(msg, ClaudeStreamDelta):
        return reduce_claude_delta(state, msg)
    elif isinstance(msg, ClaudeCompleted):
        return reduce_claude_completed(state, msg)
    elif isinstance(msg, WorkflowCompleted):
        return reduce_workflow_completed(state, msg)
    elif isinstance(msg, WorkflowFailed):
        return reduce_workflow_failed(state, msg)

    return state, []


def reduce_workflow_start(
    state: AppState, msg: WorkflowStartRequested
) -> tuple[AppState, list[AppEffect]]:
    """Handle request to start a workflow.

    Args:
        state: Current state
        msg: WorkflowStartRequested message

    Returns:
        Tuple of (new_state, effects)
    """
    workflow_id = msg.workflow_id
    workflow_type = msg.workflow_type

    effects: list[AppEffect] = [Render()]

    if workflow_type == "prompt-claude":
        # Phase 4: Implementation of Prompt Claude Start
        prompt = msg.params # Simplified for now, in real it would be JSON

        # 1. Prepare session-specific MCP config
        # Note: In a pure reducer, we shouldn't do I/O.
        # But for mcp_config_path, we'll assume the path can be deterministic
        # or we provide it as an Effect that then triggers the start.
        # To follow MVI strictly:
        # Reducer -> Effect(CreateConfig) -> Msg(ConfigReady) -> Reducer -> Effect(RunClaude)

        # However, for the first implementation, let's keep it simple:
        # We'll use a deterministic path and assume the Executor handles missing dirs.
        session_mgr = SessionConfigManager()
        mcp_config_path = session_mgr.get_session_dir(workflow_id) / "mcp-config.json"

        # 2. Create state
        data = PromptClaudeData(
            prompt=prompt,
            mcp_config_path=str(mcp_config_path)
        )

        workflow = WorkflowState[PromptClaudeData](
            id=workflow_id,
            workflow_type=workflow_type,
            status=WorkflowStatus.RUNNING,
            data=data
        )

        # 3. Update AppState
        new_active = state.active_workflows.copy()
        new_active[workflow_id] = workflow

        new_worktree = state.worktree_view.model_copy(update={
            "active_workflow_id": workflow_id,
            "workflow_output": f"🚀 Starting Claude session: {workflow_id}\n\n",
            "status_message": "Claude is thinking..."
        })

        new_state = state.model_copy(update={
            "active_workflows": new_active,
            "worktree_view": new_worktree
        })

        # 4. Dispatch Claude CLI Effect
        effects.append(RunClaudeCli(
            prompt=prompt,
            output_format="stream-json",
            cwd=Path(state.project_root or "."),
            workflow_id=workflow_id,
            mcp_config_path=mcp_config_path,
            system_prompt_file=get_system_prompt_path(),
            max_turns=10,
            permission_mode="ask"
        ))

        effects.append(LogInfo(message=f"Started workflow {workflow_type} ({workflow_id})"))

        return new_state, effects

    return state, [LogInfo(message=f"Unsupported workflow type: {workflow_type}")]


def reduce_claude_delta(
    state: AppState, msg: ClaudeStreamDelta
) -> tuple[AppState, list[AppEffect]]:
    """Handle Claude stream delta.

    Args:
        state: Current state
        msg: ClaudeStreamDelta message

    Returns:
        Tuple of (new_state, effects)
    """
    workflow_id = msg.workflow_id
    delta = msg.delta

    if workflow_id not in state.active_workflows:
        return state, []

    workflow = state.active_workflows[workflow_id]
    if not isinstance(workflow.data, PromptClaudeData):
        return state, []

    # Update workflow data
    new_data = workflow.data.append_output(delta)
    new_workflow = workflow.model_copy(update={"data": new_data})

    new_active = state.active_workflows.copy()
    new_active[workflow_id] = new_workflow

    # Update Worktree view output
    new_worktree = state.worktree_view.append_workflow_output(delta)

    new_state = state.model_copy(update={
        "active_workflows": new_active,
        "worktree_view": new_worktree
    })

    return new_state, [Render()]


def reduce_claude_completed(
    state: AppState, msg: ClaudeCompleted
) -> tuple[AppState, list[AppEffect]]:
    """Handle Claude execution completion.

    Args:
        state: Current state
        msg: ClaudeCompleted message

    Returns:
        Tuple of (new_state, effects)
    """
    workflow_id = msg.workflow_id

    if workflow_id not in state.active_workflows:
        return state, []

    workflow = state.active_workflows[workflow_id]

    # Transition to completed or failed
    if msg.success:
        new_workflow = workflow.complete()
        status_msg = "Claude finished"
    else:
        new_workflow = workflow.fail(msg.error or "Unknown error")
        status_msg = f"Claude failed: {msg.error}"

    new_active = state.active_workflows.copy()
    new_active[workflow_id] = new_workflow

    new_worktree = state.worktree_view.model_copy(update={
        "status_message": status_msg
    })

    new_state = state.model_copy(update={
        "active_workflows": new_active,
        "worktree_view": new_worktree
    })

    return new_state, [Render(), LogInfo(message=status_msg)]


def reduce_workflow_completed(
    state: AppState, msg: WorkflowCompleted
) -> tuple[AppState, list[AppEffect]]:
    """Handle generic workflow completion."""
    # Similar to claude_completed but for generic workflows
    return state, []


def reduce_workflow_failed(
    state: AppState, msg: WorkflowFailed
) -> tuple[AppState, list[AppEffect]]:
    """Handle generic workflow failure."""
    return state, []

"""MCP tool implementations for rstn.

Implements the MCP tools defined in kb/internals/mcp/tools.md plus additional
tools for state access and hook execution.
"""

from __future__ import annotations

import json
from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import TYPE_CHECKING, Any

from rstn.logging import get_logger
from rstn.mcp.types import (
    ARTIFACT_FILENAMES,
    McpStatus,
    McpToolResponse,
    SpecArtifact,
)

if TYPE_CHECKING:
    from rstn.msg import AppMsg
    from rstn.state import AppState

log = get_logger("rstn.mcp.tools")


class McpToolRegistry:
    """Registry and executor for MCP tools."""

    def __init__(
        self,
        state_getter: Callable[[], AppState],
        msg_sender: Callable[[AppMsg], Awaitable[None]],
        project_root: Path,
    ) -> None:
        """Initialize tool registry.

        Args:
            state_getter: Callback to get current AppState
            msg_sender: Callback to send AppMsg to TUI queue
            project_root: Project root directory
        """
        self.state_getter = state_getter
        self.msg_sender = msg_sender
        self.project_root = project_root

        # Register tools
        self._tools: dict[str, dict] = {
            "rstn_get_app_state": {
                "description": "Get current TUI application state as JSON",
                "inputSchema": {"type": "object", "properties": {}, "required": []},
            },
            "rstn_report_status": {
                "description": "Report task status to TUI (needs_input/completed/error)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["needs_input", "completed", "error"],
                            "description": "Current task status",
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Prompt to show user (for needs_input)",
                        },
                        "message": {
                            "type": "string",
                            "description": "Error message (for error status)",
                        },
                    },
                    "required": ["status"],
                },
            },
            "rstn_read_spec": {
                "description": "Read a spec artifact for the current feature",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "artifact": {
                            "type": "string",
                            "enum": ["spec", "plan", "tasks", "checklist", "analysis"],
                            "description": "Artifact to read",
                        },
                    },
                    "required": ["artifact"],
                },
            },
            "rstn_get_context": {
                "description": "Get current feature context and metadata",
                "inputSchema": {"type": "object", "properties": {}, "required": []},
            },
            "rstn_complete_task": {
                "description": "Mark a task as complete with optional validation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "Task ID (e.g., T001, T002)",
                        },
                        "skip_validation": {
                            "type": "boolean",
                            "default": False,
                            "description": "Skip validation checks",
                        },
                    },
                    "required": ["task_id"],
                },
            },
            "rstn_run_hook": {
                "description": "Run a project-configured hook",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "hook_name": {
                            "type": "string",
                            "description": "Name of the hook (e.g., lint, test)",
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Additional arguments to pass",
                        },
                    },
                    "required": ["hook_name"],
                },
            },
        }

    def list_tools(self) -> list[dict]:
        """List available tools with schemas."""
        return [
            {"name": name, **schema}
            for name, schema in self._tools.items()
        ]

    def has_tool(self, name: str) -> bool:
        """Check if a tool exists."""
        return name in self._tools

    async def call_tool(self, name: str, arguments: dict[str, Any]) -> McpToolResponse:
        """Call a tool by name.

        Args:
            name: Tool name
            arguments: Tool arguments

        Returns:
            McpToolResponse
        """
        handler = getattr(self, f"_tool_{name}", None)
        if handler is None:
            return McpToolResponse.error(f"Tool handler not found: {name}")

        return await handler(arguments)

    # ========================================
    # Tool Implementations
    # ========================================

    async def _tool_rstn_get_app_state(self, args: dict) -> McpToolResponse:
        """Get current TUI application state."""
        state = self.state_getter()
        return McpToolResponse.text(state.model_dump_json(indent=2))

    async def _tool_rstn_report_status(self, args: dict) -> McpToolResponse:
        """Report task status to TUI."""
        from rstn.msg import McpReportStatusReceived

        status_str = args.get("status")
        if not status_str:
            return McpToolResponse.error("Missing 'status' field")

        try:
            status = McpStatus(status_str)
        except ValueError:
            return McpToolResponse.error(
                f"Invalid status: {status_str}. Must be one of: needs_input, completed, error"
            )

        # Send message to TUI
        msg = McpReportStatusReceived(
            status=status.value,
            prompt=args.get("prompt"),
            message=args.get("message"),
        )
        await self.msg_sender(msg)

        return McpToolResponse.text(f"Status '{status.value}' reported successfully")

    async def _tool_rstn_read_spec(self, args: dict) -> McpToolResponse:
        """Read a spec artifact."""
        artifact_str = args.get("artifact")
        if not artifact_str:
            return McpToolResponse.error("Missing 'artifact' field")

        try:
            artifact = SpecArtifact(artifact_str)
        except ValueError:
            valid = ", ".join(a.value for a in SpecArtifact)
            return McpToolResponse.error(
                f"Invalid artifact: {artifact_str}. Must be one of: {valid}"
            )

        # Get current feature context
        context = self._get_feature_context()
        if not context.get("spec_dir"):
            return McpToolResponse.error(
                f"Artifact '{artifact_str}' not found. Feature context: None"
            )

        # Read artifact file
        filename = ARTIFACT_FILENAMES[artifact]
        artifact_path = self.project_root / context["spec_dir"] / filename

        if not artifact_path.exists():
            return McpToolResponse.error(
                f"Artifact '{artifact_str}' not found at {artifact_path}"
            )

        content = artifact_path.read_text()
        return McpToolResponse.text(content)

    async def _tool_rstn_get_context(self, args: dict) -> McpToolResponse:
        """Get current feature context."""
        context = self._get_feature_context()
        return McpToolResponse.text(json.dumps(context))

    async def _tool_rstn_complete_task(self, args: dict) -> McpToolResponse:
        """Mark a task as complete."""
        from rstn.msg import McpCompleteTaskReceived

        task_id = args.get("task_id")
        if not task_id:
            return McpToolResponse.error("Missing 'task_id' field")

        skip_validation = args.get("skip_validation", False)

        # Send message to TUI
        msg = McpCompleteTaskReceived(
            task_id=task_id,
            skip_validation=skip_validation,
        )
        await self.msg_sender(msg)

        return McpToolResponse.text(
            f"Task {task_id} marked for completion. Processing..."
        )

    async def _tool_rstn_run_hook(self, args: dict) -> McpToolResponse:
        """Run a project-configured hook."""
        from rstn.mcp.hooks import load_hook_config, run_hook

        hook_name = args.get("hook_name")
        if not hook_name:
            return McpToolResponse.error("Missing 'hook_name' field")

        hook_args = args.get("args", [])

        # Load hook config
        config = load_hook_config(self.project_root)
        if hook_name not in config.hooks:
            available = ", ".join(config.hooks.keys()) if config.hooks else "none"
            return McpToolResponse.error(
                f"Hook not found: {hook_name}. Available: {available}"
            )

        # Run hook
        hook = config.hooks[hook_name]
        result = await run_hook(hook, hook_args, self.project_root)

        return McpToolResponse.text(result.model_dump_json(indent=2))

    # ========================================
    # Helper Methods
    # ========================================

    def _get_feature_context(self) -> dict:
        """Get current feature context from state or git.

        Returns:
            Dict with feature_number, feature_name, branch, phase, spec_dir
        """
        # TODO: Extract phase and other info from app state in full implementation
        _ = self.state_getter()  # Reserved for future use
        context = {
            "feature_number": None,
            "feature_name": None,
            "branch": None,
            "phase": None,
            "spec_dir": None,
        }

        # Try to detect from project root
        specs_dir = self.project_root / "specs"
        if specs_dir.exists():
            # Find most recent spec directory
            spec_dirs = sorted(specs_dir.iterdir(), reverse=True)
            if spec_dirs:
                spec_dir = spec_dirs[0]
                if spec_dir.is_dir():
                    name = spec_dir.name
                    # Parse "NNN-feature-name" format
                    parts = name.split("-", 1)
                    if len(parts) == 2 and parts[0].isdigit():
                        context["feature_number"] = parts[0]
                        context["feature_name"] = parts[1]
                        context["spec_dir"] = f"specs/{name}"
                        context["branch"] = name

        return context

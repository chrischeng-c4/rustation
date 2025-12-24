"""FastAPI routes for MCP protocol.

Implements the HTTP endpoints that Claude Code uses to invoke MCP tools.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from rstn.logging import get_logger
from rstn.mcp.types import McpToolResponse

if TYPE_CHECKING:
    from rstn.msg import AppMsg
    from rstn.state import AppState

log = get_logger("rstn.mcp.routes")


class ToolCallRequest(BaseModel):
    """Request body for tool invocation."""

    arguments: dict[str, Any] = {}


def create_router(
    state_getter: Callable[[], AppState],
    msg_sender: Callable[[AppMsg], Awaitable[None]],
    project_root: Path,
) -> APIRouter:
    """Create FastAPI router with MCP endpoints.

    Args:
        state_getter: Callback to get current AppState
        msg_sender: Callback to send AppMsg to TUI queue
        project_root: Project root directory

    Returns:
        Configured APIRouter
    """
    from rstn.mcp.tools import McpToolRegistry

    router = APIRouter(prefix="/mcp", tags=["mcp"])

    # Create tool registry with dependencies
    registry = McpToolRegistry(
        state_getter=state_getter,
        msg_sender=msg_sender,
        project_root=project_root,
    )

    @router.get("/")
    async def mcp_info():
        """MCP server info endpoint."""
        return {
            "name": "rstn",
            "version": "0.1.0",
            "protocol": "mcp-http",
        }

    @router.get("/tools")
    async def list_tools():
        """List available MCP tools."""
        return {
            "tools": registry.list_tools(),
        }

    @router.post("/tools/{tool_name}")
    async def call_tool(tool_name: str, request: ToolCallRequest):
        """Invoke an MCP tool.

        Args:
            tool_name: Name of the tool to invoke
            request: Tool arguments

        Returns:
            MCP tool response
        """
        log.info("MCP tool call", tool=tool_name, args=request.arguments)

        if not registry.has_tool(tool_name):
            raise HTTPException(
                status_code=404,
                detail=f"Tool not found: {tool_name}",
            )

        try:
            result = await registry.call_tool(tool_name, request.arguments)
            log.debug("MCP tool result", tool=tool_name, is_error=result.isError)
            return result.model_dump()
        except Exception as e:
            log.error("MCP tool error", tool=tool_name, error=str(e))
            return McpToolResponse.error(f"Tool error: {e}").model_dump()

    @router.get("/health")
    async def health_check():
        """Health check endpoint."""
        return {"status": "ok"}

    return router

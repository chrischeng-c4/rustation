"""MCP server module for rstn.

Provides an embedded HTTP MCP server for Claude Code integration.

Usage:
    from rstn.mcp import McpServer, McpServerConfig

    server = McpServer(
        config=McpServerConfig(session_id="..."),
        state_getter=lambda: app_state,
        msg_sender=queue.put,
        project_root=Path("."),
    )
    port = await server.start()
    # ... later
    await server.stop()
"""

from rstn.mcp.server import McpServer
from rstn.mcp.types import (
    HookConfig,
    HookDefinition,
    HookResult,
    McpServerConfig,
    McpStatus,
    McpToolResponse,
    SpecArtifact,
)

__all__ = [
    # Server
    "McpServer",
    "McpServerConfig",
    # Types
    "McpStatus",
    "McpToolResponse",
    "SpecArtifact",
    # Hooks
    "HookConfig",
    "HookDefinition",
    "HookResult",
]

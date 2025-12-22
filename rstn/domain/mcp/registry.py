"""MCP server registry.

Pure functions for managing available MCP servers.
"""

from __future__ import annotations

from rstn.domain.mcp.types import McpServerConfig, McpServerType


def get_available_servers() -> list[McpServerConfig]:
    """Get list of available MCP servers.

    Pure function - no I/O.

    Returns:
        List of available server configurations
    """
    return [
        McpServerConfig(
            name="rstn",
            server_type=McpServerType.HTTP,
            url="http://localhost:3000/mcp",
            enabled=True,
        ),
        McpServerConfig(
            name="filesystem",
            server_type=McpServerType.STDIO,
            command="npx",
            args=["-y", "@anthropic/mcp-filesystem"],
            enabled=False,
        ),
        McpServerConfig(
            name="sequential-thinking",
            server_type=McpServerType.STDIO,
            command="npx",
            args=["-y", "@anthropic/mcp-sequential-thinking"],
            enabled=True,
        ),
    ]


def get_default_servers() -> list[McpServerConfig]:
    """Get default enabled MCP servers.

    Pure function - no I/O.

    Returns:
        List of default enabled server configurations
    """
    return [s for s in get_available_servers() if s.enabled]


def find_server(name: str) -> McpServerConfig | None:
    """Find server by name.

    Pure function - no I/O.

    Args:
        name: Server name to find

    Returns:
        Server configuration if found
    """
    for server in get_available_servers():
        if server.name == name:
            return server
    return None

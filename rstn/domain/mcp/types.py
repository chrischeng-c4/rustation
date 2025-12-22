"""MCP domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class McpServerType(str, Enum):
    """Type of MCP server."""

    STDIO = "stdio"
    HTTP = "http"
    SSE = "sse"


class McpServerConfig(BaseModel):
    """Configuration for a single MCP server."""

    model_config = {"frozen": True}

    name: str = Field(description="Server name")
    server_type: McpServerType = Field(description="Server type")
    command: str | None = Field(default=None, description="Command for stdio servers")
    args: list[str] = Field(default_factory=list, description="Command arguments")
    url: str | None = Field(default=None, description="URL for HTTP/SSE servers")
    env: dict[str, str] = Field(default_factory=dict, description="Environment variables")
    enabled: bool = Field(default=True, description="Whether server is enabled")


class McpConfig(BaseModel):
    """Complete MCP configuration."""

    model_config = {"frozen": True}

    servers: dict[str, McpServerConfig] = Field(
        default_factory=dict, description="Server configurations by name"
    )

    def to_claude_format(self) -> dict[str, Any]:
        """Convert to Claude Code MCP config format.

        Returns:
            Dict in Claude Code's expected format
        """
        result: dict[str, Any] = {"mcpServers": {}}

        for name, server in self.servers.items():
            if not server.enabled:
                continue

            if server.server_type == McpServerType.HTTP:
                result["mcpServers"][name] = {
                    "type": "http",
                    "url": server.url,
                }
            elif server.server_type == McpServerType.STDIO:
                config: dict[str, Any] = {
                    "command": server.command,
                    "args": server.args,
                }
                if server.env:
                    config["env"] = server.env
                result["mcpServers"][name] = config

        return result

    @classmethod
    def from_claude_format(cls, data: dict[str, Any]) -> McpConfig:
        """Parse from Claude Code MCP config format.

        Args:
            data: Dict in Claude Code's format

        Returns:
            McpConfig instance
        """
        servers: dict[str, McpServerConfig] = {}
        mcp_servers = data.get("mcpServers", {})

        for name, config in mcp_servers.items():
            if "type" in config and config["type"] == "http":
                servers[name] = McpServerConfig(
                    name=name,
                    server_type=McpServerType.HTTP,
                    url=config.get("url"),
                )
            else:
                servers[name] = McpServerConfig(
                    name=name,
                    server_type=McpServerType.STDIO,
                    command=config.get("command"),
                    args=config.get("args", []),
                    env=config.get("env", {}),
                )

        return cls(servers=servers)

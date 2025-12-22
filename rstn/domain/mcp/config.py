"""MCP configuration operations.

Pure functions for building MCP configurations.
Effect creators for config file operations.
"""

from __future__ import annotations

import json
from pathlib import Path

from rstn.domain.mcp.types import McpConfig, McpServerConfig
from rstn.effect import AppEffect, WriteFile


def build_mcp_config(servers: list[McpServerConfig]) -> McpConfig:
    """Build MCP config from server list.

    Pure function - no I/O.

    Args:
        servers: List of server configurations

    Returns:
        Complete MCP configuration
    """
    server_dict = {s.name: s for s in servers if s.enabled}
    return McpConfig(servers=server_dict)


def parse_mcp_config(content: str) -> McpConfig:
    """Parse MCP config from JSON string.

    Pure function - no I/O.

    Args:
        content: JSON content to parse

    Returns:
        Parsed MCP configuration
    """
    data = json.loads(content)
    return McpConfig.from_claude_format(data)


def serialize_mcp_config(config: McpConfig, pretty: bool = True) -> str:
    """Serialize MCP config to JSON string.

    Pure function - no I/O.

    Args:
        config: MCP configuration
        pretty: Whether to pretty-print

    Returns:
        JSON string
    """
    data = config.to_claude_format()
    if pretty:
        return json.dumps(data, indent=2)
    return json.dumps(data)


def create_mcp_config_effects(
    config: McpConfig,
    output_path: Path,
) -> list[AppEffect]:
    """Create effects to write MCP config file.

    Effect creator - returns effects, doesn't execute.

    Args:
        config: MCP configuration to write
        output_path: Path to write config file

    Returns:
        List of effects to execute
    """
    content = serialize_mcp_config(config, pretty=True)
    return [
        WriteFile(
            path=output_path,
            contents=content,
        )
    ]


def merge_mcp_configs(base: McpConfig, overlay: McpConfig) -> McpConfig:
    """Merge two MCP configs, with overlay taking precedence.

    Pure function - no I/O.

    Args:
        base: Base configuration
        overlay: Configuration to overlay

    Returns:
        Merged configuration
    """
    merged_servers = dict(base.servers)
    merged_servers.update(overlay.servers)
    return McpConfig(servers=merged_servers)


def create_session_config(
    base_config: McpConfig,
    session_servers: list[McpServerConfig],
) -> McpConfig:
    """Create session-specific MCP config.

    Pure function - no I/O.

    Adds session-specific servers to base config.

    Args:
        base_config: Base MCP configuration
        session_servers: Session-specific servers

    Returns:
        Session configuration
    """
    session_config = build_mcp_config(session_servers)
    return merge_mcp_configs(base_config, session_config)

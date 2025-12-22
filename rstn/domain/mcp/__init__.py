"""MCP (Model Context Protocol) domain operations for rstn.

Provides MCP configuration management including:
- MCP server registry
- Configuration generation
- Session management

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.mcp.config import (
    build_mcp_config,
    create_mcp_config_effects,
    parse_mcp_config,
)
from rstn.domain.mcp.registry import (
    get_available_servers,
    get_default_servers,
)
from rstn.domain.mcp.types import (
    McpConfig,
    McpServerConfig,
    McpServerType,
)

__all__ = [
    # Types
    "McpConfig",
    "McpServerConfig",
    "McpServerType",
    # Registry functions
    "get_available_servers",
    "get_default_servers",
    # Config functions
    "build_mcp_config",
    "create_mcp_config_effects",
    "parse_mcp_config",
]

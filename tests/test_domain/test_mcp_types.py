"""Tests for MCP domain types."""

from __future__ import annotations

import pytest
from rstn.domain.mcp.types import (
    McpConfig,
    McpServerConfig,
    McpServerType,
)


class TestMcpServerType:
    """Tests for McpServerType enum."""

    def test_server_type_values(self) -> None:
        """Test all server type values."""
        assert McpServerType.STDIO.value == "stdio"
        assert McpServerType.HTTP.value == "http"
        assert McpServerType.SSE.value == "sse"

    def test_server_type_is_string_enum(self) -> None:
        """Test McpServerType is a string enum."""
        for server_type in McpServerType:
            assert isinstance(server_type.value, str)


class TestMcpServerConfig:
    """Tests for McpServerConfig model."""

    def test_http_server_config(self) -> None:
        """Test HTTP server configuration."""
        config = McpServerConfig(
            name="rstn",
            server_type=McpServerType.HTTP,
            url="http://localhost:8080",
        )
        assert config.name == "rstn"
        assert config.server_type == McpServerType.HTTP
        assert config.url == "http://localhost:8080"
        assert config.command is None
        assert config.args == []
        assert config.env == {}
        assert config.enabled is True

    def test_stdio_server_config(self) -> None:
        """Test STDIO server configuration."""
        config = McpServerConfig(
            name="my-server",
            server_type=McpServerType.STDIO,
            command="python",
            args=["-m", "my_server"],
            env={"DEBUG": "1"},
        )
        assert config.name == "my-server"
        assert config.server_type == McpServerType.STDIO
        assert config.command == "python"
        assert config.args == ["-m", "my_server"]
        assert config.env == {"DEBUG": "1"}

    def test_server_config_disabled(self) -> None:
        """Test disabled server configuration."""
        config = McpServerConfig(
            name="disabled-server",
            server_type=McpServerType.HTTP,
            url="http://localhost:8080",
            enabled=False,
        )
        assert config.enabled is False

    def test_server_config_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        config = McpServerConfig(
            name="test",
            server_type=McpServerType.STDIO,
            command="node",
            args=["server.js"],
            env={"PORT": "3000"},
        )
        json_str = config.model_dump_json()
        restored = McpServerConfig.model_validate_json(json_str)
        assert restored == config

    def test_server_config_immutable(self) -> None:
        """Test config is immutable (frozen)."""
        config = McpServerConfig(
            name="test",
            server_type=McpServerType.HTTP,
        )
        with pytest.raises(Exception):
            config.name = "new-name"  # type: ignore


class TestMcpConfig:
    """Tests for McpConfig model."""

    def test_empty_config(self) -> None:
        """Test empty MCP config."""
        config = McpConfig()
        assert config.servers == {}

    def test_config_with_servers(self) -> None:
        """Test config with multiple servers."""
        servers = {
            "rstn": McpServerConfig(
                name="rstn",
                server_type=McpServerType.HTTP,
                url="http://localhost:8080",
            ),
            "my-server": McpServerConfig(
                name="my-server",
                server_type=McpServerType.STDIO,
                command="python",
                args=["-m", "server"],
            ),
        }
        config = McpConfig(servers=servers)
        assert len(config.servers) == 2
        assert "rstn" in config.servers
        assert "my-server" in config.servers

    def test_to_claude_format_http(self) -> None:
        """Test converting HTTP server to Claude format."""
        servers = {
            "rstn": McpServerConfig(
                name="rstn",
                server_type=McpServerType.HTTP,
                url="http://localhost:8080",
            ),
        }
        config = McpConfig(servers=servers)
        claude_format = config.to_claude_format()

        assert "mcpServers" in claude_format
        assert "rstn" in claude_format["mcpServers"]
        assert claude_format["mcpServers"]["rstn"]["type"] == "http"
        assert claude_format["mcpServers"]["rstn"]["url"] == "http://localhost:8080"

    def test_to_claude_format_stdio(self) -> None:
        """Test converting STDIO server to Claude format."""
        servers = {
            "my-server": McpServerConfig(
                name="my-server",
                server_type=McpServerType.STDIO,
                command="python",
                args=["-m", "server"],
                env={"DEBUG": "1"},
            ),
        }
        config = McpConfig(servers=servers)
        claude_format = config.to_claude_format()

        assert "mcpServers" in claude_format
        assert "my-server" in claude_format["mcpServers"]
        server_config = claude_format["mcpServers"]["my-server"]
        assert server_config["command"] == "python"
        assert server_config["args"] == ["-m", "server"]
        assert server_config["env"] == {"DEBUG": "1"}

    def test_to_claude_format_excludes_disabled(self) -> None:
        """Test disabled servers are excluded from Claude format."""
        servers = {
            "enabled": McpServerConfig(
                name="enabled",
                server_type=McpServerType.HTTP,
                url="http://localhost:8080",
                enabled=True,
            ),
            "disabled": McpServerConfig(
                name="disabled",
                server_type=McpServerType.HTTP,
                url="http://localhost:9090",
                enabled=False,
            ),
        }
        config = McpConfig(servers=servers)
        claude_format = config.to_claude_format()

        assert "enabled" in claude_format["mcpServers"]
        assert "disabled" not in claude_format["mcpServers"]

    def test_from_claude_format_http(self) -> None:
        """Test parsing HTTP server from Claude format."""
        data = {
            "mcpServers": {
                "rstn": {
                    "type": "http",
                    "url": "http://localhost:8080",
                },
            },
        }
        config = McpConfig.from_claude_format(data)

        assert "rstn" in config.servers
        assert config.servers["rstn"].server_type == McpServerType.HTTP
        assert config.servers["rstn"].url == "http://localhost:8080"

    def test_from_claude_format_stdio(self) -> None:
        """Test parsing STDIO server from Claude format."""
        data = {
            "mcpServers": {
                "my-server": {
                    "command": "python",
                    "args": ["-m", "server"],
                    "env": {"DEBUG": "1"},
                },
            },
        }
        config = McpConfig.from_claude_format(data)

        assert "my-server" in config.servers
        server = config.servers["my-server"]
        assert server.server_type == McpServerType.STDIO
        assert server.command == "python"
        assert server.args == ["-m", "server"]
        assert server.env == {"DEBUG": "1"}

    def test_from_claude_format_empty(self) -> None:
        """Test parsing empty Claude format."""
        data: dict[str, dict[str, object]] = {"mcpServers": {}}
        config = McpConfig.from_claude_format(data)
        assert config.servers == {}

    def test_from_claude_format_no_mcp_servers(self) -> None:
        """Test parsing Claude format without mcpServers."""
        data: dict[str, object] = {}
        config = McpConfig.from_claude_format(data)
        assert config.servers == {}

    def test_round_trip_http(self) -> None:
        """Test round-trip conversion for HTTP server."""
        servers = {
            "rstn": McpServerConfig(
                name="rstn",
                server_type=McpServerType.HTTP,
                url="http://localhost:8080",
            ),
        }
        original = McpConfig(servers=servers)
        claude_format = original.to_claude_format()
        restored = McpConfig.from_claude_format(claude_format)

        assert "rstn" in restored.servers
        assert restored.servers["rstn"].server_type == McpServerType.HTTP
        assert restored.servers["rstn"].url == "http://localhost:8080"

    def test_round_trip_stdio(self) -> None:
        """Test round-trip conversion for STDIO server."""
        servers = {
            "my-server": McpServerConfig(
                name="my-server",
                server_type=McpServerType.STDIO,
                command="node",
                args=["index.js"],
            ),
        }
        original = McpConfig(servers=servers)
        claude_format = original.to_claude_format()
        restored = McpConfig.from_claude_format(claude_format)

        assert "my-server" in restored.servers
        assert restored.servers["my-server"].server_type == McpServerType.STDIO
        assert restored.servers["my-server"].command == "node"

    def test_config_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        servers = {
            "test": McpServerConfig(
                name="test",
                server_type=McpServerType.HTTP,
                url="http://test",
            ),
        }
        config = McpConfig(servers=servers)
        json_str = config.model_dump_json()
        restored = McpConfig.model_validate_json(json_str)
        assert len(restored.servers) == 1

    def test_config_immutable(self) -> None:
        """Test config is immutable (frozen)."""
        config = McpConfig()
        with pytest.raises(Exception):
            config.servers = {}  # type: ignore

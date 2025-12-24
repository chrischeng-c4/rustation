"""Tests for MCP types."""

import pytest
from rstn.mcp.types import (
    HookConfig,
    HookDefinition,
    HookResult,
    McpServerConfig,
    McpStatus,
    McpToolResponse,
    SpecArtifact,
)


class TestMcpServerConfig:
    """Tests for McpServerConfig."""

    def test_default_values(self) -> None:
        """Test default values."""
        config = McpServerConfig(session_id="test-123")
        assert config.host == "127.0.0.1"
        assert config.port == 0
        assert config.session_id == "test-123"

    def test_custom_values(self) -> None:
        """Test custom values."""
        config = McpServerConfig(
            host="0.0.0.0",
            port=8080,
            session_id="custom-session",
        )
        assert config.host == "0.0.0.0"
        assert config.port == 8080
        assert config.session_id == "custom-session"

    def test_frozen(self) -> None:
        """Test that config is immutable."""
        config = McpServerConfig(session_id="test")
        with pytest.raises(Exception):
            config.port = 9999  # type: ignore


class TestMcpStatus:
    """Tests for McpStatus enum."""

    def test_values(self) -> None:
        """Test enum values."""
        assert McpStatus.NEEDS_INPUT == "needs_input"
        assert McpStatus.COMPLETED == "completed"
        assert McpStatus.ERROR == "error"

    def test_from_string(self) -> None:
        """Test creating from string."""
        status = McpStatus("needs_input")
        assert status == McpStatus.NEEDS_INPUT


class TestMcpToolResponse:
    """Tests for McpToolResponse."""

    def test_text_response(self) -> None:
        """Test creating text response."""
        response = McpToolResponse.text("Hello, world!")
        assert response.content == [{"type": "text", "text": "Hello, world!"}]
        assert response.isError is False

    def test_error_response(self) -> None:
        """Test creating error response."""
        response = McpToolResponse.error("Something went wrong")
        assert response.content == [{"type": "text", "text": "Something went wrong"}]
        assert response.isError is True

    def test_text_with_error_flag(self) -> None:
        """Test creating text response with error flag."""
        response = McpToolResponse.text("Error message", is_error=True)
        assert response.isError is True

    def test_serialization(self) -> None:
        """Test JSON serialization."""
        response = McpToolResponse.text("Test")
        json_data = response.model_dump()
        assert "content" in json_data
        assert "isError" in json_data


class TestHookDefinition:
    """Tests for HookDefinition."""

    def test_minimal_hook(self) -> None:
        """Test minimal hook definition."""
        hook = HookDefinition(command="echo hello")
        assert hook.command == "echo hello"
        assert hook.timeout_secs == 120
        assert hook.cwd is None
        assert hook.env == {}

    def test_full_hook(self) -> None:
        """Test full hook definition."""
        hook = HookDefinition(
            command="pytest",
            timeout_secs=300,
            cwd="/project",
            env={"PYTHONPATH": "."},
        )
        assert hook.command == "pytest"
        assert hook.timeout_secs == 300
        assert hook.cwd == "/project"
        assert hook.env == {"PYTHONPATH": "."}


class TestHookConfig:
    """Tests for HookConfig."""

    def test_empty_config(self) -> None:
        """Test empty hook config."""
        config = HookConfig()
        assert config.hooks == {}

    def test_with_hooks(self) -> None:
        """Test config with hooks."""
        config = HookConfig(
            hooks={
                "lint": HookDefinition(command="ruff check ."),
                "test": HookDefinition(command="pytest", timeout_secs=300),
            }
        )
        assert "lint" in config.hooks
        assert "test" in config.hooks
        assert config.hooks["lint"].command == "ruff check ."


class TestHookResult:
    """Tests for HookResult."""

    def test_successful_result(self) -> None:
        """Test successful hook result."""
        result = HookResult(
            hook_name="test",
            exit_code=0,
            stdout="All tests passed",
            stderr="",
            duration_secs=1.5,
        )
        assert result.hook_name == "test"
        assert result.exit_code == 0
        assert result.stdout == "All tests passed"
        assert result.stderr == ""
        assert result.duration_secs == 1.5

    def test_failed_result(self) -> None:
        """Test failed hook result."""
        result = HookResult(
            hook_name="lint",
            exit_code=1,
            stdout="",
            stderr="Error: linting failed",
            duration_secs=0.5,
        )
        assert result.exit_code == 1
        assert result.stderr == "Error: linting failed"


class TestSpecArtifact:
    """Tests for SpecArtifact enum."""

    def test_values(self) -> None:
        """Test enum values."""
        assert SpecArtifact.SPEC == "spec"
        assert SpecArtifact.PLAN == "plan"
        assert SpecArtifact.TASKS == "tasks"
        assert SpecArtifact.CHECKLIST == "checklist"
        assert SpecArtifact.ANALYSIS == "analysis"

    def test_from_string(self) -> None:
        """Test creating from string."""
        artifact = SpecArtifact("spec")
        assert artifact == SpecArtifact.SPEC

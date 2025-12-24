"""Tests for MCP tools."""

from pathlib import Path
from unittest.mock import AsyncMock

import pytest
from rstn.mcp.tools import McpToolRegistry
from rstn.state import AppState


@pytest.fixture
def state_getter() -> callable:
    """Create a state getter fixture."""
    return lambda: AppState()


@pytest.fixture
def msg_sender() -> AsyncMock:
    """Create a mock message sender."""
    return AsyncMock()


@pytest.fixture
def project_root(tmp_path: Path) -> Path:
    """Create a temporary project root."""
    return tmp_path


@pytest.fixture
def registry(
    state_getter: callable,
    msg_sender: AsyncMock,
    project_root: Path,
) -> McpToolRegistry:
    """Create a tool registry fixture."""
    return McpToolRegistry(
        state_getter=state_getter,
        msg_sender=msg_sender,
        project_root=project_root,
    )


class TestMcpToolRegistry:
    """Tests for McpToolRegistry."""

    def test_list_tools(self, registry: McpToolRegistry) -> None:
        """Test listing available tools."""
        tools = registry.list_tools()
        assert len(tools) == 6

        tool_names = [t["name"] for t in tools]
        assert "rstn_get_app_state" in tool_names
        assert "rstn_report_status" in tool_names
        assert "rstn_read_spec" in tool_names
        assert "rstn_get_context" in tool_names
        assert "rstn_complete_task" in tool_names
        assert "rstn_run_hook" in tool_names

    def test_has_tool(self, registry: McpToolRegistry) -> None:
        """Test checking if tool exists."""
        assert registry.has_tool("rstn_get_app_state") is True
        assert registry.has_tool("rstn_report_status") is True
        assert registry.has_tool("nonexistent_tool") is False

    @pytest.mark.asyncio
    async def test_call_unknown_tool(self, registry: McpToolRegistry) -> None:
        """Test calling unknown tool."""
        result = await registry.call_tool("unknown_tool", {})
        assert result.isError is True
        assert "not found" in result.content[0]["text"]


class TestGetAppState:
    """Tests for rstn_get_app_state tool."""

    @pytest.mark.asyncio
    async def test_get_app_state(self, registry: McpToolRegistry) -> None:
        """Test getting app state."""
        result = await registry.call_tool("rstn_get_app_state", {})
        assert result.isError is False
        # Result should be valid JSON
        import json

        state_data = json.loads(result.content[0]["text"])
        assert "running" in state_data
        assert "current_view" in state_data


class TestReportStatus:
    """Tests for rstn_report_status tool."""

    @pytest.mark.asyncio
    async def test_report_needs_input(
        self,
        registry: McpToolRegistry,
        msg_sender: AsyncMock,
    ) -> None:
        """Test reporting needs_input status."""
        result = await registry.call_tool(
            "rstn_report_status",
            {"status": "needs_input", "prompt": "Enter your answer:"},
        )
        assert result.isError is False
        assert "successfully" in result.content[0]["text"]
        msg_sender.assert_called_once()

    @pytest.mark.asyncio
    async def test_report_completed(
        self,
        registry: McpToolRegistry,
        msg_sender: AsyncMock,
    ) -> None:
        """Test reporting completed status."""
        result = await registry.call_tool(
            "rstn_report_status",
            {"status": "completed"},
        )
        assert result.isError is False
        msg_sender.assert_called_once()

    @pytest.mark.asyncio
    async def test_report_error(
        self,
        registry: McpToolRegistry,
        msg_sender: AsyncMock,
    ) -> None:
        """Test reporting error status."""
        result = await registry.call_tool(
            "rstn_report_status",
            {"status": "error", "message": "Something went wrong"},
        )
        assert result.isError is False
        msg_sender.assert_called_once()

    @pytest.mark.asyncio
    async def test_missing_status(self, registry: McpToolRegistry) -> None:
        """Test missing status field."""
        result = await registry.call_tool("rstn_report_status", {})
        assert result.isError is True
        assert "Missing" in result.content[0]["text"]

    @pytest.mark.asyncio
    async def test_invalid_status(self, registry: McpToolRegistry) -> None:
        """Test invalid status value."""
        result = await registry.call_tool(
            "rstn_report_status",
            {"status": "invalid_status"},
        )
        assert result.isError is True
        assert "Invalid status" in result.content[0]["text"]


class TestReadSpec:
    """Tests for rstn_read_spec tool."""

    @pytest.mark.asyncio
    async def test_missing_artifact(self, registry: McpToolRegistry) -> None:
        """Test missing artifact field."""
        result = await registry.call_tool("rstn_read_spec", {})
        assert result.isError is True
        assert "Missing" in result.content[0]["text"]

    @pytest.mark.asyncio
    async def test_invalid_artifact(self, registry: McpToolRegistry) -> None:
        """Test invalid artifact value."""
        result = await registry.call_tool(
            "rstn_read_spec",
            {"artifact": "invalid"},
        )
        assert result.isError is True
        assert "Invalid artifact" in result.content[0]["text"]

    @pytest.mark.asyncio
    async def test_no_feature_context(self, registry: McpToolRegistry) -> None:
        """Test reading spec without feature context."""
        result = await registry.call_tool(
            "rstn_read_spec",
            {"artifact": "spec"},
        )
        assert result.isError is True
        assert "not found" in result.content[0]["text"]

    @pytest.mark.asyncio
    async def test_read_existing_spec(
        self, project_root: Path, registry: McpToolRegistry
    ) -> None:
        """Test reading existing spec file."""
        # Create spec directory and file
        spec_dir = project_root / "specs" / "001-test-feature"
        spec_dir.mkdir(parents=True)
        spec_file = spec_dir / "spec.md"
        spec_file.write_text("# Test Spec\n\nThis is a test spec.")

        result = await registry.call_tool(
            "rstn_read_spec",
            {"artifact": "spec"},
        )
        assert result.isError is False
        assert "Test Spec" in result.content[0]["text"]


class TestGetContext:
    """Tests for rstn_get_context tool."""

    @pytest.mark.asyncio
    async def test_get_context_empty(self, registry: McpToolRegistry) -> None:
        """Test getting context with no specs."""
        result = await registry.call_tool("rstn_get_context", {})
        assert result.isError is False

        import json

        context = json.loads(result.content[0]["text"])
        assert "feature_number" in context
        assert "feature_name" in context
        assert "branch" in context

    @pytest.mark.asyncio
    async def test_get_context_with_spec(
        self, project_root: Path, registry: McpToolRegistry
    ) -> None:
        """Test getting context with existing spec."""
        # Create spec directory
        spec_dir = project_root / "specs" / "001-test-feature"
        spec_dir.mkdir(parents=True)

        result = await registry.call_tool("rstn_get_context", {})
        assert result.isError is False

        import json

        context = json.loads(result.content[0]["text"])
        assert context["feature_number"] == "001"
        assert context["feature_name"] == "test-feature"


class TestCompleteTask:
    """Tests for rstn_complete_task tool."""

    @pytest.mark.asyncio
    async def test_complete_task(
        self,
        registry: McpToolRegistry,
        msg_sender: AsyncMock,
    ) -> None:
        """Test completing a task."""
        result = await registry.call_tool(
            "rstn_complete_task",
            {"task_id": "T001"},
        )
        assert result.isError is False
        assert "T001" in result.content[0]["text"]
        msg_sender.assert_called_once()

    @pytest.mark.asyncio
    async def test_complete_task_skip_validation(
        self,
        registry: McpToolRegistry,
        msg_sender: AsyncMock,
    ) -> None:
        """Test completing a task with skip_validation."""
        result = await registry.call_tool(
            "rstn_complete_task",
            {"task_id": "T002", "skip_validation": True},
        )
        assert result.isError is False
        msg_sender.assert_called_once()

    @pytest.mark.asyncio
    async def test_missing_task_id(self, registry: McpToolRegistry) -> None:
        """Test missing task_id field."""
        result = await registry.call_tool("rstn_complete_task", {})
        assert result.isError is True
        assert "Missing" in result.content[0]["text"]


class TestRunHook:
    """Tests for rstn_run_hook tool."""

    @pytest.mark.asyncio
    async def test_missing_hook_name(self, registry: McpToolRegistry) -> None:
        """Test missing hook_name field."""
        result = await registry.call_tool("rstn_run_hook", {})
        assert result.isError is True
        assert "Missing" in result.content[0]["text"]

    @pytest.mark.asyncio
    async def test_hook_not_found(self, registry: McpToolRegistry) -> None:
        """Test hook not found."""
        result = await registry.call_tool(
            "rstn_run_hook",
            {"hook_name": "nonexistent"},
        )
        assert result.isError is True
        assert "not found" in result.content[0]["text"]

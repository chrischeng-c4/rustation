"""Tests for MCP hooks."""

from pathlib import Path

import pytest
from rstn.mcp.hooks import load_hook_config, run_hook
from rstn.mcp.types import HookConfig, HookDefinition


class TestLoadHookConfig:
    """Tests for load_hook_config."""

    def test_no_config(self, tmp_path: Path) -> None:
        """Test loading when no config file exists."""
        config = load_hook_config(tmp_path)
        assert isinstance(config, HookConfig)
        assert config.hooks == {}

    def test_load_json_config(self, tmp_path: Path) -> None:
        """Test loading from JSON file."""
        rstn_dir = tmp_path / ".rstn"
        rstn_dir.mkdir()

        config_file = rstn_dir / "hooks.json"
        config_file.write_text(
            """{
            "hooks": {
                "lint": {"command": "ruff check ."},
                "test": {"command": "pytest", "timeout_secs": 300}
            }
        }"""
        )

        config = load_hook_config(tmp_path)
        assert "lint" in config.hooks
        assert "test" in config.hooks
        assert config.hooks["lint"].command == "ruff check ."
        assert config.hooks["test"].timeout_secs == 300

    def test_load_yaml_config(self, tmp_path: Path) -> None:
        """Test loading from YAML file."""
        pytest.importorskip("yaml")

        rstn_dir = tmp_path / ".rstn"
        rstn_dir.mkdir()

        config_file = rstn_dir / "hooks.yaml"
        config_file.write_text(
            """hooks:
  format:
    command: "ruff format ."
    timeout_secs: 60
"""
        )

        config = load_hook_config(tmp_path)
        assert "format" in config.hooks
        assert config.hooks["format"].command == "ruff format ."

    def test_yaml_priority_over_json(self, tmp_path: Path) -> None:
        """Test that YAML takes priority over JSON."""
        pytest.importorskip("yaml")

        rstn_dir = tmp_path / ".rstn"
        rstn_dir.mkdir()

        # Create both files
        (rstn_dir / "hooks.json").write_text(
            '{"hooks": {"from": {"command": "json"}}}'
        )
        (rstn_dir / "hooks.yaml").write_text(
            "hooks:\n  from:\n    command: yaml\n"
        )

        config = load_hook_config(tmp_path)
        assert config.hooks["from"].command == "yaml"

    def test_invalid_json(self, tmp_path: Path) -> None:
        """Test handling invalid JSON."""
        rstn_dir = tmp_path / ".rstn"
        rstn_dir.mkdir()

        config_file = rstn_dir / "hooks.json"
        config_file.write_text("not valid json")

        config = load_hook_config(tmp_path)
        assert config.hooks == {}


class TestRunHook:
    """Tests for run_hook."""

    @pytest.mark.asyncio
    async def test_run_simple_hook(self, tmp_path: Path) -> None:
        """Test running a simple hook."""
        hook = HookDefinition(command="echo hello")

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == 0
        assert "hello" in result.stdout
        assert result.hook_name == "echo"

    @pytest.mark.asyncio
    async def test_run_hook_with_args(self, tmp_path: Path) -> None:
        """Test running a hook with arguments."""
        hook = HookDefinition(command="echo")

        result = await run_hook(hook, ["world"], tmp_path)

        assert result.exit_code == 0
        assert "world" in result.stdout

    @pytest.mark.asyncio
    async def test_run_failing_hook(self, tmp_path: Path) -> None:
        """Test running a hook that fails."""
        hook = HookDefinition(command="exit 1")

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == 1

    @pytest.mark.asyncio
    async def test_run_hook_with_cwd(self, tmp_path: Path) -> None:
        """Test running a hook with custom cwd."""
        subdir = tmp_path / "subdir"
        subdir.mkdir()

        hook = HookDefinition(command="pwd", cwd=str(subdir))

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == 0
        assert "subdir" in result.stdout

    @pytest.mark.asyncio
    async def test_run_hook_with_env(self, tmp_path: Path) -> None:
        """Test running a hook with custom environment."""
        hook = HookDefinition(
            command="echo $MY_VAR",
            env={"MY_VAR": "custom_value"},
        )

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == 0
        assert "custom_value" in result.stdout

    @pytest.mark.asyncio
    async def test_hook_timeout(self, tmp_path: Path) -> None:
        """Test hook timeout."""
        hook = HookDefinition(
            command="sleep 10",
            timeout_secs=1,
        )

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == -1
        assert "timed out" in result.stderr

    @pytest.mark.asyncio
    async def test_hook_duration_tracking(self, tmp_path: Path) -> None:
        """Test that hook duration is tracked."""
        hook = HookDefinition(command="echo fast")

        result = await run_hook(hook, [], tmp_path)

        assert result.duration_secs >= 0
        assert result.duration_secs < 5  # Should be very fast

    @pytest.mark.asyncio
    async def test_hook_stderr_capture(self, tmp_path: Path) -> None:
        """Test capturing stderr."""
        hook = HookDefinition(command="echo error >&2")

        result = await run_hook(hook, [], tmp_path)

        assert result.exit_code == 0
        assert "error" in result.stderr

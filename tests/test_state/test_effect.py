"""Tests for effect types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.effect import (
    AgentKind,
    Batch,
    CancelAgent,
    CancelWorkflow,
    DeleteFile,
    LoadState,
    LogDebug,
    LogError,
    LogInfo,
    QuitApp,
    ReadFile,
    Render,
    RunBashScript,
    RunCommand,
    SaveState,
    SpawnAgent,
    StartTimer,
    StopTimer,
    WriteFile,
)
from rstn.state import AppState


class TestAgentKind:
    """Test AgentKind enum."""

    def test_agent_kind_values(self) -> None:
        """AgentKind has expected values."""
        assert AgentKind.EXPLORE == "explore"  # type: ignore
        assert AgentKind.PLAN == "plan"  # type: ignore
        assert AgentKind.GENERAL_PURPOSE == "general_purpose"  # type: ignore


class TestAgentEffects:
    """Test agent execution effect types."""

    def test_spawn_agent(self) -> None:
        """SpawnAgent effect."""
        effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.EXPLORE,
            prompt="Test prompt",
        )
        assert effect.workflow_id == "wf-123"
        assert effect.agent_kind == AgentKind.EXPLORE
        assert effect.prompt == "Test prompt"
        assert effect.mcp_config_path is None

    def test_spawn_agent_with_mcp_config(self) -> None:
        """SpawnAgent with MCP config."""
        effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.PLAN,
            prompt="Plan this",
            mcp_config_path=Path("/tmp/mcp.json"),
        )
        assert effect.mcp_config_path == Path("/tmp/mcp.json")

    def test_spawn_agent_immutable(self) -> None:
        """SpawnAgent is immutable."""
        effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.EXPLORE,
            prompt="Test",
        )
        with pytest.raises(Exception):  # noqa: B017
            effect.workflow_id = "wf-456"

    def test_cancel_agent(self) -> None:
        """CancelAgent effect."""
        effect = CancelAgent(workflow_id="wf-123")
        assert effect.workflow_id == "wf-123"


class TestFileEffects:
    """Test file operation effect types."""

    def test_write_file(self) -> None:
        """WriteFile effect."""
        effect = WriteFile(path=Path("/tmp/test.txt"), contents="Hello World")
        assert effect.path == Path("/tmp/test.txt")
        assert effect.contents == "Hello World"

    def test_read_file(self) -> None:
        """ReadFile effect."""
        effect = ReadFile(path=Path("/tmp/test.txt"))
        assert effect.path == Path("/tmp/test.txt")

    def test_delete_file(self) -> None:
        """DeleteFile effect."""
        effect = DeleteFile(path=Path("/tmp/test.txt"))
        assert effect.path == Path("/tmp/test.txt")


class TestCommandEffects:
    """Test command execution effect types."""

    def test_run_command(self) -> None:
        """RunCommand effect."""
        effect = RunCommand(
            cmd="ls",
            args=["-la"],
            cwd=Path("/tmp"),
        )
        assert effect.cmd == "ls"
        assert effect.args == ["-la"]
        assert effect.cwd == Path("/tmp")

    def test_run_command_no_args(self) -> None:
        """RunCommand with no arguments."""
        effect = RunCommand(cmd="pwd", cwd=Path("/tmp"))
        assert effect.args == []

    def test_run_bash_script(self) -> None:
        """RunBashScript effect."""
        effect = RunBashScript(
            script_path=Path("/tmp/script.sh"),
            args=["arg1", "arg2"],
        )
        assert effect.script_path == Path("/tmp/script.sh")
        assert effect.args == ["arg1", "arg2"]


class TestTimerEffects:
    """Test timer effect types."""

    def test_start_timer(self) -> None:
        """StartTimer effect."""
        effect = StartTimer(timer_id="tick", delay_ms=1000)
        assert effect.timer_id == "tick"
        assert effect.delay_ms == 1000

    def test_start_timer_validation(self) -> None:
        """StartTimer validates delay_ms > 0."""
        with pytest.raises(ValueError):
            StartTimer(timer_id="tick", delay_ms=0)

        with pytest.raises(ValueError):
            StartTimer(timer_id="tick", delay_ms=-100)

    def test_stop_timer(self) -> None:
        """StopTimer effect."""
        effect = StopTimer(timer_id="tick")
        assert effect.timer_id == "tick"


class TestWorkflowEffects:
    """Test workflow management effect types."""

    def test_cancel_workflow(self) -> None:
        """CancelWorkflow effect."""
        effect = CancelWorkflow(workflow_id="wf-123")
        assert effect.workflow_id == "wf-123"


class TestStateEffects:
    """Test state persistence effect types."""

    def test_save_state(self) -> None:
        """SaveState effect."""
        state = AppState()
        effect = SaveState(path=Path("/tmp/state.json"), state=state)
        assert effect.path == Path("/tmp/state.json")
        assert effect.state == state

    def test_load_state(self) -> None:
        """LoadState effect."""
        effect = LoadState(path=Path("/tmp/state.json"))
        assert effect.path == Path("/tmp/state.json")


class TestLoggingEffects:
    """Test logging effect types."""

    def test_log_info(self) -> None:
        """LogInfo effect."""
        effect = LogInfo(message="Info message")
        assert effect.message == "Info message"

    def test_log_error(self) -> None:
        """LogError effect."""
        effect = LogError(message="Error message")
        assert effect.message == "Error message"

    def test_log_debug(self) -> None:
        """LogDebug effect."""
        effect = LogDebug(message="Debug message")
        assert effect.message == "Debug message"


class TestUIEffects:
    """Test UI effect types."""

    def test_render(self) -> None:
        """Render effect."""
        effect = Render()
        assert effect is not None

    def test_quit_app(self) -> None:
        """QuitApp effect."""
        effect = QuitApp()
        assert effect is not None


class TestBatchEffect:
    """Test Batch effect."""

    def test_batch_empty(self) -> None:
        """Batch with no effects."""
        effect = Batch(effects=[])
        assert effect.effects == []

    def test_batch_multiple_effects(self) -> None:
        """Batch with multiple effects."""
        effects = [
            LogInfo(message="Starting"),
            Render(),
            LogInfo(message="Done"),
        ]
        batch = Batch(effects=effects)
        assert len(batch.effects) == 3
        assert isinstance(batch.effects[0], LogInfo)
        assert isinstance(batch.effects[1], Render)

    def test_batch_immutable(self) -> None:
        """Batch is immutable."""
        batch = Batch(effects=[])
        with pytest.raises(Exception):  # noqa: B017
            batch.effects = [LogInfo(message="test")]


class TestEffectSerialization:
    """Test effect serialization."""

    def test_spawn_agent_serialization(self) -> None:
        """SpawnAgent can be serialized."""
        effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.EXPLORE,
            prompt="Test",
        )
        json_str = effect.model_dump_json()
        loaded = SpawnAgent.model_validate_json(json_str)
        assert loaded.workflow_id == effect.workflow_id
        assert loaded.agent_kind == effect.agent_kind

    def test_write_file_serialization(self) -> None:
        """WriteFile can be serialized."""
        effect = WriteFile(path=Path("/tmp/test.txt"), contents="Hello")
        json_str = effect.model_dump_json()
        loaded = WriteFile.model_validate_json(json_str)
        assert loaded.path == effect.path
        assert loaded.contents == effect.contents

    def test_batch_serialization(self) -> None:
        """Batch can be serialized."""
        batch = Batch(
            effects=[
                LogInfo(message="test"),
                Render(),
            ]
        )
        json_str = batch.model_dump_json()
        loaded = Batch.model_validate_json(json_str)
        assert len(loaded.effects) == 2

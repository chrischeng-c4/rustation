"""Tests for message types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.msg import (
    AgentCompleted,
    AgentStreamDelta,
    CommandCompleted,
    EffectCompleted,
    EffectFailed,
    ErrorOccurred,
    FileReadCompleted,
    FileWritten,
    KeyCode,
    KeyModifiers,
    KeyPressed,
    MouseClicked,
    Noop,
    Quit,
    ScrollContent,
    SelectCommand,
    StateSaved,
    SwitchView,
    Tick,
    WorkflowCancelled,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowStartRequested,
    WorkflowStepCompleted,
)
from rstn.state.types import ViewType


class TestKeyModifiers:
    """Test KeyModifiers."""

    def test_default_modifiers(self) -> None:
        """Default modifiers have no keys pressed."""
        mods = KeyModifiers()
        assert not mods.ctrl
        assert not mods.shift
        assert not mods.alt
        assert mods.is_empty()

    def test_ctrl_key_helper(self) -> None:
        """ctrl_key() creates modifiers with only Ctrl."""
        mods = KeyModifiers.ctrl_key()
        assert mods.ctrl
        assert not mods.shift
        assert not mods.alt
        assert not mods.is_empty()

    def test_shift_key_helper(self) -> None:
        """shift_key() creates modifiers with only Shift."""
        mods = KeyModifiers.shift_key()
        assert not mods.ctrl
        assert mods.shift
        assert not mods.alt
        assert not mods.is_empty()

    def test_alt_key_helper(self) -> None:
        """alt_key() creates modifiers with only Alt."""
        mods = KeyModifiers.alt_key()
        assert not mods.ctrl
        assert not mods.shift
        assert mods.alt
        assert not mods.is_empty()

    def test_multiple_modifiers(self) -> None:
        """Multiple modifiers can be pressed."""
        mods = KeyModifiers(ctrl=True, shift=True)
        assert mods.ctrl
        assert mods.shift
        assert not mods.alt
        assert not mods.is_empty()

    def test_modifiers_immutable(self) -> None:
        """KeyModifiers is immutable."""
        mods = KeyModifiers()
        with pytest.raises(Exception):  # noqa: B017  # ValidationError from Pydantic
            mods.ctrl = True


class TestKeyCode:
    """Test KeyCode enum."""

    def test_key_code_values(self) -> None:
        """KeyCode has expected values."""
        assert KeyCode.ENTER == "enter"  # type: ignore
        assert KeyCode.ESC == "esc"  # type: ignore
        assert KeyCode.UP == "up"  # type: ignore
        assert KeyCode.DOWN == "down"  # type: ignore


class TestUserInputMessages:
    """Test user input message types."""

    def test_key_pressed(self) -> None:
        """KeyPressed message."""
        msg = KeyPressed(key="a", modifiers=KeyModifiers())
        assert msg.key == "a"
        assert msg.modifiers.is_empty()

    def test_key_pressed_with_modifiers(self) -> None:
        """KeyPressed with modifiers."""
        msg = KeyPressed(key="c", modifiers=KeyModifiers.ctrl_key())
        assert msg.key == "c"
        assert msg.modifiers.ctrl

    def test_key_pressed_immutable(self) -> None:
        """KeyPressed is immutable."""
        msg = KeyPressed(key="a")
        with pytest.raises(Exception):  # noqa: B017
            msg.key = "b"

    def test_mouse_clicked(self) -> None:
        """MouseClicked message."""
        msg = MouseClicked(x=10, y=20)
        assert msg.x == 10
        assert msg.y == 20

    def test_mouse_clicked_validation(self) -> None:
        """MouseClicked validates coordinates."""
        with pytest.raises(ValueError):
            MouseClicked(x=-1, y=0)

    def test_tick(self) -> None:
        """Tick message."""
        msg = Tick()
        assert msg is not None


class TestViewMessages:
    """Test view-related message types."""

    def test_switch_view(self) -> None:
        """SwitchView message."""
        msg = SwitchView(view=ViewType.DASHBOARD)
        assert msg.view == ViewType.DASHBOARD

    def test_scroll_content(self) -> None:
        """ScrollContent message."""
        msg = ScrollContent(delta=5)
        assert msg.delta == 5

        msg_neg = ScrollContent(delta=-3)
        assert msg_neg.delta == -3

    def test_select_command(self) -> None:
        """SelectCommand message."""
        msg = SelectCommand(index=2)
        assert msg.index == 2

    def test_select_command_validation(self) -> None:
        """SelectCommand validates index."""
        with pytest.raises(ValueError):
            SelectCommand(index=-1)


class TestWorkflowMessages:
    """Test workflow-related message types."""

    def test_workflow_start_requested(self) -> None:
        """WorkflowStartRequested message."""
        msg = WorkflowStartRequested(
            workflow_id="wf-123", workflow_type="prompt-claude", params='{"prompt": "test"}'
        )
        assert msg.workflow_id == "wf-123"
        assert msg.workflow_type == "prompt-claude"
        assert msg.params == '{"prompt": "test"}'

    def test_workflow_step_completed(self) -> None:
        """WorkflowStepCompleted message."""
        msg = WorkflowStepCompleted(workflow_id="wf-123", step_name="plan", success=True)
        assert msg.workflow_id == "wf-123"
        assert msg.step_name == "plan"
        assert msg.success

    def test_workflow_completed(self) -> None:
        """WorkflowCompleted message."""
        msg = WorkflowCompleted(workflow_id="wf-123")
        assert msg.workflow_id == "wf-123"

    def test_workflow_failed(self) -> None:
        """WorkflowFailed message."""
        msg = WorkflowFailed(workflow_id="wf-123", error="Test error")
        assert msg.workflow_id == "wf-123"
        assert msg.error == "Test error"

    def test_workflow_cancelled(self) -> None:
        """WorkflowCancelled message."""
        msg = WorkflowCancelled(workflow_id="wf-123")
        assert msg.workflow_id == "wf-123"


class TestAgentMessages:
    """Test agent-related message types."""

    def test_agent_stream_delta(self) -> None:
        """AgentStreamDelta message."""
        msg = AgentStreamDelta(workflow_id="wf-123", delta="Hello ")
        assert msg.workflow_id == "wf-123"
        assert msg.delta == "Hello "

    def test_agent_completed(self) -> None:
        """AgentCompleted message."""
        msg = AgentCompleted(workflow_id="wf-123", output="Final output")
        assert msg.workflow_id == "wf-123"
        assert msg.output == "Final output"


class TestEffectResultMessages:
    """Test effect result message types."""

    def test_effect_completed(self) -> None:
        """EffectCompleted message."""
        msg = EffectCompleted(effect_id="eff-123", result="success")
        assert msg.effect_id == "eff-123"
        assert msg.result == "success"

    def test_effect_failed(self) -> None:
        """EffectFailed message."""
        msg = EffectFailed(effect_id="eff-123", error="Test error")
        assert msg.effect_id == "eff-123"
        assert msg.error == "Test error"

    def test_file_written(self) -> None:
        """FileWritten message."""
        msg = FileWritten(path=Path("/tmp/test.txt"))
        assert msg.path == Path("/tmp/test.txt")

    def test_file_read_completed(self) -> None:
        """FileReadCompleted message."""
        msg = FileReadCompleted(path=Path("/tmp/test.txt"), contents="Hello")
        assert msg.path == Path("/tmp/test.txt")
        assert msg.contents == "Hello"

    def test_command_completed(self) -> None:
        """CommandCompleted message."""
        msg = CommandCompleted(exit_code=0, stdout="output", stderr="")
        assert msg.exit_code == 0
        assert msg.stdout == "output"
        assert msg.stderr == ""


class TestSystemMessages:
    """Test system message types."""

    def test_error_occurred(self) -> None:
        """ErrorOccurred message."""
        msg = ErrorOccurred(message="Test error")
        assert msg.message == "Test error"

    def test_state_saved(self) -> None:
        """StateSaved message."""
        msg = StateSaved(path=Path("/tmp/state.json"))
        assert msg.path == Path("/tmp/state.json")

    def test_quit(self) -> None:
        """Quit message."""
        msg = Quit()
        assert msg is not None

    def test_noop(self) -> None:
        """Noop message."""
        msg = Noop()
        assert msg is not None


class TestMessageSerialization:
    """Test message serialization."""

    def test_key_pressed_serialization(self) -> None:
        """KeyPressed can be serialized."""
        msg = KeyPressed(key="a", modifiers=KeyModifiers.ctrl_key())
        json_str = msg.model_dump_json()
        loaded = KeyPressed.model_validate_json(json_str)
        assert loaded.key == msg.key
        assert loaded.modifiers.ctrl == msg.modifiers.ctrl

    def test_workflow_message_serialization(self) -> None:
        """Workflow messages can be serialized."""
        msg = WorkflowStartRequested(
            workflow_id="wf-123", workflow_type="test", params="{}"
        )
        json_str = msg.model_dump_json()
        loaded = WorkflowStartRequested.model_validate_json(json_str)
        assert loaded.workflow_id == msg.workflow_id
        assert loaded.workflow_type == msg.workflow_type

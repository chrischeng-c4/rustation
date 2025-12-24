"""Tests for WorktreeViewState."""

from __future__ import annotations

import pytest
from rstn.state.worktree import Command, ContentType, WorktreeViewState


class TestContentType:
    """Test ContentType enum."""

    def test_content_type_values(self) -> None:
        """ContentType has expected values."""
        assert ContentType.EMPTY == "empty"  # type: ignore
        assert ContentType.SPEC == "spec"  # type: ignore
        assert ContentType.PLAN == "plan"  # type: ignore
        assert ContentType.TIMELINE == "timeline"  # type: ignore
        assert ContentType.LOG == "log"  # type: ignore
        assert ContentType.HELP == "help"  # type: ignore


class TestCommand:
    """Test Command model."""

    def test_command_creation(self) -> None:
        """Create command with required fields."""
        cmd = Command(id="cmd-1", label="Test Command")

        assert cmd.id == "cmd-1"
        assert cmd.label == "Test Command"
        assert cmd.description == ""
        assert cmd.enabled is True
        assert cmd.workflow_type is None

    def test_command_with_all_fields(self) -> None:
        """Create command with all fields."""
        cmd = Command(
            id="cmd-1",
            label="Test",
            description="A test command",
            enabled=False,
            workflow_type="test-workflow",
        )

        assert cmd.id == "cmd-1"
        assert cmd.description == "A test command"
        assert cmd.enabled is False
        assert cmd.workflow_type == "test-workflow"

    def test_command_serialization(self) -> None:
        """Command can be serialized."""
        cmd = Command(
            id="cmd-1", label="Test", description="Desc", workflow_type="wf-type"
        )

        json_str = cmd.model_dump_json()
        loaded = Command.model_validate_json(json_str)

        assert loaded.id == cmd.id
        assert loaded.label == cmd.label
        assert loaded.description == cmd.description
        assert loaded.workflow_type == cmd.workflow_type


class TestWorktreeViewStateCreation:
    """Test WorktreeViewState creation."""

    def test_default_worktree_state(self) -> None:
        """Create default worktree state."""
        worktree = WorktreeViewState()

        # Content
        assert worktree.content_type == ContentType.EMPTY
        assert worktree.spec_content is None
        assert worktree.plan_content is None
        assert worktree.log_content == []

        # Commands (now includes default prompt-claude command)
        assert len(worktree.commands) == 1
        assert worktree.commands[0].id == "prompt-claude"
        assert worktree.selected_command_index == 0

        # Workflow
        assert worktree.active_workflow_id is None
        assert worktree.workflow_output == ""

        # UI
        assert worktree.command_list_scroll == 0
        assert worktree.content_scroll == 0

        # Input
        assert worktree.input_mode is False
        assert worktree.input_buffer == ""
        assert worktree.input_prompt == ""

        # Status
        assert worktree.status_message == "Ready"

    def test_worktree_state_with_commands(self) -> None:
        """Create worktree state with commands."""
        commands = [
            Command(id="cmd-1", label="Command 1"),
            Command(id="cmd-2", label="Command 2"),
        ]
        worktree = WorktreeViewState(commands=commands)

        assert len(worktree.commands) == 2
        assert worktree.commands[0].id == "cmd-1"


class TestWithContent:
    """Test with_content() method."""

    def test_with_content_empty(self) -> None:
        """with_content() sets empty content."""
        worktree = WorktreeViewState()
        updated = worktree.with_content(ContentType.EMPTY)

        assert worktree.content_type == ContentType.EMPTY
        assert updated.content_type == ContentType.EMPTY

    def test_with_content_spec(self) -> None:
        """with_content() sets spec content."""
        worktree = WorktreeViewState()
        updated = worktree.with_content(ContentType.SPEC, "# Spec\nContent here")

        assert updated.content_type == ContentType.SPEC
        assert updated.spec_content == "# Spec\nContent here"

    def test_with_content_plan(self) -> None:
        """with_content() sets plan content."""
        worktree = WorktreeViewState()
        updated = worktree.with_content(ContentType.PLAN, "# Plan\n1. Step 1")

        assert updated.content_type == ContentType.PLAN
        assert updated.plan_content == "# Plan\n1. Step 1"

    def test_with_content_preserves_other_fields(self) -> None:
        """with_content() preserves other fields."""
        commands = [Command(id="cmd-1", label="Test")]
        worktree = WorktreeViewState(
            commands=commands,
            selected_command_index=0,
            status_message="Custom",
        )
        updated = worktree.with_content(ContentType.SPEC, "Test")

        assert updated.commands == commands
        assert updated.selected_command_index == 0
        assert updated.status_message == "Custom"

    def test_with_content_overwrites_existing(self) -> None:
        """with_content() overwrites existing content."""
        worktree = WorktreeViewState().with_content(ContentType.SPEC, "Old spec")
        updated = worktree.with_content(ContentType.SPEC, "New spec")

        assert worktree.spec_content == "Old spec"
        assert updated.spec_content == "New spec"


class TestWithWorkflow:
    """Test with_workflow() method."""

    def test_with_workflow_sets_id(self) -> None:
        """with_workflow() sets workflow ID."""
        worktree = WorktreeViewState()
        updated = worktree.with_workflow("wf-123")

        assert worktree.active_workflow_id is None
        assert updated.active_workflow_id == "wf-123"

    def test_with_workflow_clears_output_on_new(self) -> None:
        """with_workflow() clears output when setting new workflow."""
        worktree = WorktreeViewState(workflow_output="old output")
        updated = worktree.with_workflow("wf-123")

        assert updated.active_workflow_id == "wf-123"
        assert updated.workflow_output == ""

    def test_with_workflow_clears_workflow(self) -> None:
        """with_workflow(None) clears workflow but preserves output."""
        worktree = WorktreeViewState(
            active_workflow_id="wf-123", workflow_output="output"
        )
        updated = worktree.with_workflow(None)

        assert updated.active_workflow_id is None
        assert updated.workflow_output == "output"  # Preserved


class TestAppendWorkflowOutput:
    """Test append_workflow_output() method."""

    def test_append_workflow_output_to_empty(self) -> None:
        """append_workflow_output() appends to empty output."""
        worktree = WorktreeViewState()
        updated = worktree.append_workflow_output("Hello")

        assert worktree.workflow_output == ""
        assert updated.workflow_output == "Hello"

    def test_append_workflow_output_accumulates(self) -> None:
        """append_workflow_output() accumulates text."""
        worktree = WorktreeViewState(workflow_output="Hello ")
        updated = worktree.append_workflow_output("World")

        assert updated.workflow_output == "Hello World"

    def test_append_workflow_output_preserves_state(self) -> None:
        """append_workflow_output() preserves other state."""
        worktree = WorktreeViewState(
            active_workflow_id="wf-123", status_message="Running"
        )
        updated = worktree.append_workflow_output("Output")

        assert updated.active_workflow_id == "wf-123"
        assert updated.status_message == "Running"


class TestAddLog:
    """Test add_log() method."""

    def test_add_log_to_empty(self) -> None:
        """add_log() adds to empty log."""
        worktree = WorktreeViewState()
        updated = worktree.add_log("First log")

        assert worktree.log_content == []
        assert updated.log_content == ["First log"]

    def test_add_log_prepends(self) -> None:
        """add_log() prepends to log (latest first)."""
        worktree = WorktreeViewState(log_content=["Old log"])
        updated = worktree.add_log("New log")

        assert updated.log_content == ["New log", "Old log"]

    def test_add_log_limits_to_100(self) -> None:
        """add_log() limits log to 100 messages."""
        logs = [f"log-{i}" for i in range(100)]
        worktree = WorktreeViewState(log_content=logs)
        updated = worktree.add_log("new-log")

        assert len(updated.log_content) == 100
        assert updated.log_content[0] == "new-log"
        assert "log-99" not in updated.log_content  # Oldest removed


class TestSelectCommand:
    """Test select_command() method."""

    def test_select_command_valid_index(self) -> None:
        """select_command() sets valid index."""
        commands = [
            Command(id="cmd-1", label="Command 1"),
            Command(id="cmd-2", label="Command 2"),
        ]
        worktree = WorktreeViewState(commands=commands)
        updated = worktree.select_command(1)

        assert worktree.selected_command_index == 0
        assert updated.selected_command_index == 1

    def test_select_command_clamps_to_max(self) -> None:
        """select_command() clamps to max index."""
        commands = [Command(id="cmd-1", label="Command 1")]
        worktree = WorktreeViewState(commands=commands)
        updated = worktree.select_command(999)

        assert updated.selected_command_index == 0  # Clamped to max

    def test_select_command_clamps_to_min(self) -> None:
        """select_command() clamps to 0."""
        commands = [Command(id="cmd-1", label="Command 1")]
        worktree = WorktreeViewState(commands=commands)
        updated = worktree.select_command(-5)

        assert updated.selected_command_index == 0

    def test_select_command_with_empty_commands(self) -> None:
        """select_command() with empty commands returns 0."""
        worktree = WorktreeViewState(commands=[])
        updated = worktree.select_command(5)

        assert updated.selected_command_index == 0


class TestInputMode:
    """Test input mode methods."""

    def test_enter_input_mode(self) -> None:
        """enter_input_mode() enables input mode."""
        worktree = WorktreeViewState()
        updated = worktree.enter_input_mode("Enter text:")

        assert worktree.input_mode is False
        assert updated.input_mode is True
        assert updated.input_prompt == "Enter text:"
        assert updated.input_buffer == ""

    def test_enter_input_mode_clears_buffer(self) -> None:
        """enter_input_mode() clears existing buffer."""
        worktree = WorktreeViewState(input_buffer="old text")
        updated = worktree.enter_input_mode("New prompt:")

        assert updated.input_buffer == ""
        assert updated.input_prompt == "New prompt:"

    def test_exit_input_mode(self) -> None:
        """exit_input_mode() disables input mode."""
        worktree = WorktreeViewState(
            input_mode=True, input_prompt="Prompt", input_buffer="text"
        )
        updated = worktree.exit_input_mode()

        assert worktree.input_mode is True
        assert updated.input_mode is False
        assert updated.input_prompt == ""
        assert updated.input_buffer == ""

    def test_input_mode_roundtrip(self) -> None:
        """Can enter and exit input mode."""
        worktree = WorktreeViewState()
        entered = worktree.enter_input_mode("Prompt")
        exited = entered.exit_input_mode()

        assert exited.input_mode is False
        assert exited.input_prompt == ""


class TestScrollPosition:
    """Test scroll position fields."""

    def test_command_list_scroll_default(self) -> None:
        """command_list_scroll defaults to 0."""
        worktree = WorktreeViewState()
        assert worktree.command_list_scroll == 0

    def test_content_scroll_default(self) -> None:
        """content_scroll defaults to 0."""
        worktree = WorktreeViewState()
        assert worktree.content_scroll == 0

    def test_scroll_validation(self) -> None:
        """Scroll positions must be >= 0."""
        with pytest.raises(ValueError):
            WorktreeViewState(command_list_scroll=-1)

        with pytest.raises(ValueError):
            WorktreeViewState(content_scroll=-1)

    def test_scroll_can_be_updated(self) -> None:
        """Scroll positions can be updated."""
        worktree = WorktreeViewState()
        updated = worktree.model_copy(
            update={"command_list_scroll": 10, "content_scroll": 20}
        )

        assert updated.command_list_scroll == 10
        assert updated.content_scroll == 20


class TestWorktreeStateInvariants:
    """Test worktree state invariants."""

    def test_valid_worktree_invariants(self) -> None:
        """Valid worktree passes invariant checks."""
        commands = [
            Command(id="cmd-1", label="Command 1"),
            Command(id="cmd-2", label="Command 2"),
        ]
        worktree = WorktreeViewState(commands=commands, selected_command_index=1)

        # Should not raise
        worktree.assert_invariants()

    def test_invariant_selected_index_valid_range(self) -> None:
        """Selected command index must be within range."""
        commands = [Command(id="cmd-1", label="Command 1")]
        worktree = WorktreeViewState(commands=commands, selected_command_index=5)

        with pytest.raises(AssertionError, match="must be within commands range"):
            worktree.assert_invariants()

    def test_invariant_log_max_100(self) -> None:
        """Log content should not exceed 100 messages."""
        logs = [f"log-{i}" for i in range(101)]
        worktree = WorktreeViewState(log_content=logs)

        with pytest.raises(AssertionError, match="should not exceed 100 messages"):
            worktree.assert_invariants()

    def test_invariant_input_mode_requires_prompt(self) -> None:
        """Input mode requires a prompt message."""
        worktree = WorktreeViewState(input_mode=True, input_prompt="")

        with pytest.raises(AssertionError, match="requires a prompt message"):
            worktree.assert_invariants()

    def test_invariant_input_mode_with_prompt_valid(self) -> None:
        """Input mode with prompt is valid."""
        worktree = WorktreeViewState(input_mode=True, input_prompt="Enter:")

        # Should not raise
        worktree.assert_invariants()

    def test_invariant_unique_command_ids(self) -> None:
        """Command IDs must be unique."""
        commands = [
            Command(id="cmd-1", label="Command 1"),
            Command(id="cmd-1", label="Command 2"),  # Duplicate ID
        ]
        worktree = WorktreeViewState(commands=commands)

        with pytest.raises(AssertionError, match="must be unique"):
            worktree.assert_invariants()

    def test_invariant_empty_commands_valid(self) -> None:
        """Empty commands list is valid."""
        worktree = WorktreeViewState(commands=[])

        # Should not raise
        worktree.assert_invariants()


class TestWorktreeStateSerialization:
    """Test worktree state serialization."""

    def test_worktree_state_serialization(self) -> None:
        """WorktreeViewState can be serialized."""
        commands = [
            Command(id="cmd-1", label="Command 1", description="Desc 1"),
            Command(id="cmd-2", label="Command 2"),
        ]
        worktree = WorktreeViewState(
            content_type=ContentType.SPEC,
            spec_content="# Test Spec",
            commands=commands,
            selected_command_index=1,
            active_workflow_id="wf-123",
            workflow_output="Output text",
            log_content=["log1", "log2"],
            status_message="Running",
        )

        json_str = worktree.model_dump_json()
        loaded = WorktreeViewState.model_validate_json(json_str)

        assert loaded.content_type == worktree.content_type
        assert loaded.spec_content == worktree.spec_content
        assert len(loaded.commands) == len(worktree.commands)
        assert loaded.commands[0].id == worktree.commands[0].id
        assert loaded.selected_command_index == worktree.selected_command_index
        assert loaded.active_workflow_id == worktree.active_workflow_id
        assert loaded.workflow_output == worktree.workflow_output
        assert loaded.log_content == worktree.log_content
        assert loaded.status_message == worktree.status_message

    def test_worktree_state_with_defaults(self) -> None:
        """WorktreeViewState with defaults can be serialized."""
        worktree = WorktreeViewState()

        json_str = worktree.model_dump_json()
        loaded = WorktreeViewState.model_validate_json(json_str)

        assert loaded.content_type == ContentType.EMPTY
        assert len(loaded.commands) == 1
        assert loaded.commands[0].id == "prompt-claude"
        assert loaded.selected_command_index == 0

    def test_worktree_state_dict_round_trip(self) -> None:
        """WorktreeViewState can round-trip through dict."""
        worktree = WorktreeViewState(
            content_type=ContentType.PLAN,
            plan_content="# Plan",
            commands=[Command(id="cmd-1", label="Test")],
        )

        data = worktree.model_dump()
        loaded = WorktreeViewState.model_validate(data)

        assert loaded.content_type == worktree.content_type
        assert loaded.plan_content == worktree.plan_content
        assert len(loaded.commands) == 1


class TestWorktreeStateImmutability:
    """Test worktree state immutability."""

    def test_methods_return_new_instance(self) -> None:
        """Methods return new instances."""
        worktree = WorktreeViewState()

        updated1 = worktree.with_content(ContentType.SPEC, "Test")
        updated2 = worktree.with_workflow("wf-1")
        updated3 = worktree.append_workflow_output("Output")
        updated4 = worktree.add_log("Log")
        updated5 = worktree.select_command(0)
        updated6 = worktree.enter_input_mode("Prompt")

        # All should be different instances
        assert updated1 is not worktree
        assert updated2 is not worktree
        assert updated3 is not worktree
        assert updated4 is not worktree
        assert updated5 is not worktree
        assert updated6 is not worktree

    def test_original_unchanged_after_updates(self) -> None:
        """Original worktree unchanged after updates."""
        worktree = WorktreeViewState(
            content_type=ContentType.EMPTY,
            workflow_output="",
            log_content=[],
            input_mode=False,
        )

        worktree.with_content(ContentType.SPEC, "Spec")
        worktree.with_workflow("wf-1")
        worktree.append_workflow_output("Output")
        worktree.add_log("Log")
        worktree.enter_input_mode("Prompt")

        # Original should be unchanged
        assert worktree.content_type == ContentType.EMPTY
        assert worktree.workflow_output == ""
        assert worktree.log_content == []
        assert worktree.input_mode is False


class TestWorktreeStateChaining:
    """Test chaining worktree state updates."""

    def test_chain_content_and_workflow(self) -> None:
        """Test chaining content and workflow updates."""
        worktree = (
            WorktreeViewState()
            .with_content(ContentType.SPEC, "# Spec")
            .with_workflow("wf-123")
            .append_workflow_output("Starting...")
        )

        assert worktree.content_type == ContentType.SPEC
        assert worktree.spec_content == "# Spec"
        assert worktree.active_workflow_id == "wf-123"
        assert worktree.workflow_output == "Starting..."
        worktree.assert_invariants()

    def test_chain_commands_and_logs(self) -> None:
        """Test chaining with commands and logs."""
        commands = [
            Command(id="cmd-1", label="Command 1"),
            Command(id="cmd-2", label="Command 2"),
        ]
        worktree = (
            WorktreeViewState(commands=commands)
            .select_command(1)
            .add_log("Log 1")
            .add_log("Log 2")
        )

        assert worktree.selected_command_index == 1
        assert worktree.log_content == ["Log 2", "Log 1"]
        worktree.assert_invariants()

    def test_chain_input_mode_flow(self) -> None:
        """Test chaining input mode flow."""
        worktree = (
            WorktreeViewState()
            .enter_input_mode("Enter name:")
            .model_copy(update={"input_buffer": "test"})
            .exit_input_mode()
        )

        assert worktree.input_mode is False
        assert worktree.input_prompt == ""
        assert worktree.input_buffer == ""
        worktree.assert_invariants()

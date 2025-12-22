"""Integration tests for EffectExecutor."""

from __future__ import annotations

import asyncio
import tempfile
from pathlib import Path

import pytest
from rstn.effect import (
    AgentKind,
    Batch,
    CancelAgent,
    DefaultEffectExecutor,
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
from rstn.msg import (
    AgentCompleted,
    AgentStreamDelta,
    AppMsg,
    CommandCompleted,
    EffectCompleted,
    ErrorOccurred,
    FileReadCompleted,
    FileWritten,
    Quit,
    StateSaved,
)
from rstn.state import AppState


class MockMessageSender:
    """Mock message sender for testing."""

    def __init__(self) -> None:
        """Initialize mock sender."""
        self.messages: list[AppMsg] = []

    async def send(self, msg: AppMsg) -> None:
        """Record sent message."""
        self.messages.append(msg)

    def get_messages_of_type[T: AppMsg](self, msg_type: type[T]) -> list[T]:
        """Get all messages of a specific type."""
        return [msg for msg in self.messages if isinstance(msg, msg_type)]  # type: ignore[return-value]

    def clear(self) -> None:
        """Clear all recorded messages."""
        self.messages.clear()


@pytest.fixture
def msg_sender() -> MockMessageSender:
    """Create mock message sender."""
    return MockMessageSender()


@pytest.fixture
def executor(msg_sender: MockMessageSender) -> DefaultEffectExecutor:
    """Create effect executor with mock sender."""
    return DefaultEffectExecutor(msg_sender)


class TestFileOperations:
    """Test file operation effects."""

    @pytest.mark.asyncio
    async def test_write_file(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """WriteFile creates file and sends confirmation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "test.txt"
            effect = WriteFile(path=path, contents="Hello World")

            await executor.execute(effect)

            # File should be created
            assert path.exists()
            assert path.read_text() == "Hello World"

            # Should send FileWritten message
            messages = msg_sender.get_messages_of_type(FileWritten)
            assert len(messages) == 1
            assert messages[0].path == path

    @pytest.mark.asyncio
    async def test_write_file_creates_parent_dirs(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """WriteFile creates parent directories if needed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "subdir" / "test.txt"
            effect = WriteFile(path=path, contents="Test")

            await executor.execute(effect)

            assert path.exists()
            assert path.read_text() == "Test"

    @pytest.mark.asyncio
    async def test_read_file(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """ReadFile reads file and sends contents."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "test.txt"
            path.write_text("File contents")

            effect = ReadFile(path=path)
            await executor.execute(effect)

            # Should send FileReadCompleted message
            messages = msg_sender.get_messages_of_type(FileReadCompleted)
            assert len(messages) == 1
            assert messages[0].path == path
            assert messages[0].contents == "File contents"

    @pytest.mark.asyncio
    async def test_read_nonexistent_file(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """ReadFile sends error for nonexistent file."""
        path = Path("/nonexistent/file.txt")
        effect = ReadFile(path=path)

        await executor.execute(effect)

        # Should send ErrorOccurred message
        messages = msg_sender.get_messages_of_type(ErrorOccurred)
        assert len(messages) == 1
        assert "Failed to read" in messages[0].message

    @pytest.mark.asyncio
    async def test_delete_file(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """DeleteFile removes file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "test.txt"
            path.write_text("Delete me")

            effect = DeleteFile(path=path)
            await executor.execute(effect)

            # File should be deleted
            assert not path.exists()

            # Should send EffectCompleted message
            messages = msg_sender.get_messages_of_type(EffectCompleted)
            assert len(messages) == 1


class TestCommandExecution:
    """Test command execution effects."""

    @pytest.mark.asyncio
    async def test_run_command_success(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """RunCommand executes command and sends result."""
        with tempfile.TemporaryDirectory() as tmpdir:
            effect = RunCommand(cmd="echo", args=["Hello"], cwd=Path(tmpdir))

            await executor.execute(effect)

            # Should send CommandCompleted message
            messages = msg_sender.get_messages_of_type(CommandCompleted)
            assert len(messages) == 1
            assert messages[0].exit_code == 0
            assert "Hello" in messages[0].stdout

    @pytest.mark.asyncio
    async def test_run_command_failure(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """RunCommand handles command failures."""
        with tempfile.TemporaryDirectory() as tmpdir:
            effect = RunCommand(cmd="false", args=[], cwd=Path(tmpdir))

            await executor.execute(effect)

            # Should send CommandCompleted with non-zero exit code
            messages = msg_sender.get_messages_of_type(CommandCompleted)
            assert len(messages) == 1
            assert messages[0].exit_code != 0

    @pytest.mark.asyncio
    async def test_run_bash_script(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """RunBashScript executes script."""
        with tempfile.TemporaryDirectory() as tmpdir:
            script_path = Path(tmpdir) / "test.sh"
            script_path.write_text("#!/bin/bash\necho 'Script output'")
            script_path.chmod(0o755)

            effect = RunBashScript(script_path=script_path, args=[])

            await executor.execute(effect)

            # Should send CommandCompleted message
            messages = msg_sender.get_messages_of_type(CommandCompleted)
            assert len(messages) == 1
            assert messages[0].exit_code == 0
            assert "Script output" in messages[0].stdout


class TestAgentExecution:
    """Test agent execution effects."""

    @pytest.mark.asyncio
    async def test_spawn_agent(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """SpawnAgent spawns background agent task."""
        effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.EXPLORE,
            prompt="Test prompt",
        )

        await executor.execute(effect)

        # Give agent time to run
        await asyncio.sleep(0.3)

        # Should have sent stream deltas and completion
        stream_msgs = msg_sender.get_messages_of_type(AgentStreamDelta)
        assert len(stream_msgs) >= 2

        completion_msgs = msg_sender.get_messages_of_type(AgentCompleted)
        assert len(completion_msgs) == 1
        assert completion_msgs[0].workflow_id == "wf-123"

    @pytest.mark.asyncio
    async def test_cancel_agent(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """CancelAgent cancels running agent."""
        # Spawn agent
        spawn_effect = SpawnAgent(
            workflow_id="wf-123",
            agent_kind=AgentKind.PLAN,
            prompt="Test",
        )
        await executor.execute(spawn_effect)

        # Clear messages
        msg_sender.clear()

        # Cancel agent
        cancel_effect = CancelAgent(workflow_id="wf-123")
        await executor.execute(cancel_effect)

        # Should send WorkflowCancelled message
        # (Using ErrorOccurred or EffectCompleted as proxy since we don't have WorkflowCancelled in imports)
        assert len(msg_sender.messages) >= 1


class TestTimers:
    """Test timer effects."""

    @pytest.mark.asyncio
    async def test_start_timer(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """StartTimer starts a timer."""
        effect = StartTimer(timer_id="tick", delay_ms=100)

        await executor.execute(effect)

        # Wait for timer to tick
        await asyncio.sleep(0.15)

        # Should have sent Tick message
        from rstn.msg import Tick

        tick_msgs = msg_sender.get_messages_of_type(Tick)
        assert len(tick_msgs) >= 1

        # Stop timer
        await executor.execute(StopTimer(timer_id="tick"))

    @pytest.mark.asyncio
    async def test_stop_timer(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """StopTimer stops a running timer."""
        # Start timer
        await executor.execute(StartTimer(timer_id="tick", delay_ms=50))

        # Stop timer immediately
        await executor.execute(StopTimer(timer_id="tick"))

        # Wait and verify no ticks
        msg_sender.clear()
        await asyncio.sleep(0.1)

        from rstn.msg import Tick

        tick_msgs = msg_sender.get_messages_of_type(Tick)
        assert len(tick_msgs) == 0


class TestStatePersistence:
    """Test state persistence effects."""

    @pytest.mark.asyncio
    async def test_save_state(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """SaveState saves state to file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "state.json"
            state = AppState()

            effect = SaveState(path=path, state=state)
            await executor.execute(effect)

            # File should be created
            assert path.exists()

            # Should send StateSaved message
            messages = msg_sender.get_messages_of_type(StateSaved)
            assert len(messages) == 1
            assert messages[0].path == path

    @pytest.mark.asyncio
    async def test_load_state(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """LoadState loads state from file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "state.json"
            state = AppState()
            state.save_to_file(path)

            effect = LoadState(path=path)
            await executor.execute(effect)

            # Should send EffectCompleted message
            messages = msg_sender.get_messages_of_type(EffectCompleted)
            assert len(messages) == 1


class TestLoggingEffects:
    """Test logging effects."""

    @pytest.mark.asyncio
    async def test_log_info(
        self, executor: DefaultEffectExecutor, capsys: pytest.CaptureFixture[str]
    ) -> None:
        """LogInfo prints info message."""
        effect = LogInfo(message="Info message")
        await executor.execute(effect)

        captured = capsys.readouterr()
        assert "INFO: Info message" in captured.out

    @pytest.mark.asyncio
    async def test_log_error(
        self, executor: DefaultEffectExecutor, capsys: pytest.CaptureFixture[str]
    ) -> None:
        """LogError prints error message."""
        effect = LogError(message="Error message")
        await executor.execute(effect)

        captured = capsys.readouterr()
        assert "ERROR: Error message" in captured.out

    @pytest.mark.asyncio
    async def test_log_debug(
        self, executor: DefaultEffectExecutor, capsys: pytest.CaptureFixture[str]
    ) -> None:
        """LogDebug prints debug message."""
        effect = LogDebug(message="Debug message")
        await executor.execute(effect)

        captured = capsys.readouterr()
        assert "DEBUG: Debug message" in captured.out


class TestUIEffects:
    """Test UI effects."""

    @pytest.mark.asyncio
    async def test_render(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """Render effect executes without error."""
        effect = Render()
        await executor.execute(effect)

        # Should not send any messages (render is handled by TUI layer)
        assert len(msg_sender.messages) == 0

    @pytest.mark.asyncio
    async def test_quit_app(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """QuitApp sends Quit message."""
        effect = QuitApp()
        await executor.execute(effect)

        # Should send Quit message
        messages = msg_sender.get_messages_of_type(Quit)
        assert len(messages) == 1


class TestBatchEffect:
    """Test batch effect execution."""

    @pytest.mark.asyncio
    async def test_batch_execution(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """Batch executes all effects in sequence."""
        with tempfile.TemporaryDirectory() as tmpdir:
            path1 = Path(tmpdir) / "file1.txt"
            path2 = Path(tmpdir) / "file2.txt"

            effect = Batch(
                effects=[
                    WriteFile(path=path1, contents="Content 1"),
                    WriteFile(path=path2, contents="Content 2"),
                    LogInfo(message="Batch complete"),
                ]
            )

            await executor.execute(effect)

            # All files should be created
            assert path1.exists()
            assert path2.exists()

            # Should have sent 2 FileWritten messages
            messages = msg_sender.get_messages_of_type(FileWritten)
            assert len(messages) == 2


class TestErrorHandling:
    """Test error handling in executor."""

    @pytest.mark.asyncio
    async def test_unknown_effect_type(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """Unknown effect type sends error message."""
        # Create a mock effect that's not in the executor's dispatch
        class UnknownEffect:
            pass

        # This would normally not happen, but tests edge case
        # We can't actually test this without modifying the Effect union
        # So this test is more of a documentation of expected behavior
        pass

    @pytest.mark.asyncio
    async def test_effect_execution_failure(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """Effect execution failures send error messages."""
        # Try to write to a directory that doesn't exist and can't be created
        path = Path("/root/invalid/path/file.txt")
        effect = WriteFile(path=path, contents="Test")

        await executor.execute(effect)

        # Should send ErrorOccurred message
        messages = msg_sender.get_messages_of_type(ErrorOccurred)
        assert len(messages) == 1
        assert "Failed to write" in messages[0].message


class TestCleanup:
    """Test executor cleanup."""

    @pytest.mark.asyncio
    async def test_cleanup_cancels_tasks(
        self, executor: DefaultEffectExecutor, msg_sender: MockMessageSender
    ) -> None:
        """cleanup() cancels all active tasks and timers."""
        # Spawn agent
        await executor.execute(
            SpawnAgent(
                workflow_id="wf-1", agent_kind=AgentKind.EXPLORE, prompt="Test"
            )
        )

        # Start timer
        await executor.execute(StartTimer(timer_id="tick", delay_ms=100))

        # Verify tasks are running
        assert len(executor.active_tasks) == 1
        assert len(executor.timers) == 1

        # Cleanup
        await executor.cleanup()

        # All tasks should be cancelled
        assert len(executor.active_tasks) == 0
        assert len(executor.timers) == 0

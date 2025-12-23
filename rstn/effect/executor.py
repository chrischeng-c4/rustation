"""Effect executor for executing side effects and converting results to messages.

EffectExecutor is responsible for:
1. Executing effects (file I/O, commands, agent spawning, etc.)
2. Converting execution results into AppMsg
3. Sending messages back to the main event loop
"""

from __future__ import annotations

import asyncio
import subprocess
from typing import TYPE_CHECKING, Protocol

from rstn.effect import (
    AgentKind,
    AppEffect,
    Batch,
    CancelAgent,
    CancelWorkflow,
    CopyToClipboard,
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
    StateSaved,
    WorkflowCancelled,
    WorkflowFailed,
)

if TYPE_CHECKING:
    pass


class MessageSender(Protocol):
    """Protocol for sending messages back to the main event loop."""

    async def send(self, msg: AppMsg) -> None:
        """Send a message to the main event loop."""
        ...


class EffectExecutor(Protocol):
    """Protocol for effect executors.

    EffectExecutor receives effects from the reducer and executes them,
    converting results into messages that are sent back to the main loop.
    """

    async def execute(self, effect: AppEffect) -> None:
        """Execute an effect and send result messages."""
        ...


class DefaultEffectExecutor:
    """Default implementation of EffectExecutor.

    Executes all standard effects and sends result messages back to the event loop.
    """

    def __init__(self, msg_sender: MessageSender) -> None:
        """Initialize executor with message sender.

        Args:
            msg_sender: Channel for sending messages back to main loop
        """
        self.msg_sender = msg_sender
        self.active_tasks: dict[str, asyncio.Task[None]] = {}
        self.timers: dict[str, asyncio.Task[None]] = {}

    async def execute(self, effect: AppEffect) -> None:
        """Execute an effect and send result messages.

        Args:
            effect: The effect to execute
        """
        try:
            if isinstance(effect, Batch):
                await self._execute_batch(effect)
            elif isinstance(effect, SpawnAgent):
                await self._execute_spawn_agent(effect)
            elif isinstance(effect, CancelAgent):
                await self._execute_cancel_agent(effect)
            elif isinstance(effect, WriteFile):
                await self._execute_write_file(effect)
            elif isinstance(effect, ReadFile):
                await self._execute_read_file(effect)
            elif isinstance(effect, DeleteFile):
                await self._execute_delete_file(effect)
            elif isinstance(effect, RunCommand):
                await self._execute_run_command(effect)
            elif isinstance(effect, RunBashScript):
                await self._execute_run_bash_script(effect)
            elif isinstance(effect, StartTimer):
                await self._execute_start_timer(effect)
            elif isinstance(effect, StopTimer):
                await self._execute_stop_timer(effect)
            elif isinstance(effect, CancelWorkflow):
                await self._execute_cancel_workflow(effect)
            elif isinstance(effect, SaveState):
                await self._execute_save_state(effect)
            elif isinstance(effect, LoadState):
                await self._execute_load_state(effect)
            elif isinstance(effect, LogInfo):
                await self._execute_log_info(effect)
            elif isinstance(effect, LogError):
                await self._execute_log_error(effect)
            elif isinstance(effect, LogDebug):
                await self._execute_log_debug(effect)
            elif isinstance(effect, Render):
                await self._execute_render(effect)
            elif isinstance(effect, QuitApp):
                await self._execute_quit_app(effect)
            elif isinstance(effect, CopyToClipboard):
                await self._execute_copy_to_clipboard(effect)
            else:
                await self.msg_sender.send(
                    ErrorOccurred(message=f"Unknown effect type: {type(effect)}")
                )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Effect execution failed: {e}")
            )

    async def _execute_batch(self, effect: Batch) -> None:
        """Execute batch of effects in sequence."""
        for sub_effect in effect.effects:
            await self.execute(sub_effect)

    async def _execute_spawn_agent(self, effect: SpawnAgent) -> None:
        """Spawn an agent and stream its output.

        Args:
            effect: SpawnAgent effect
        """
        # Create background task for agent execution
        task = asyncio.create_task(
            self._run_agent(effect.workflow_id, effect.agent_kind, effect.prompt)
        )
        self.active_tasks[effect.workflow_id] = task

    async def _run_agent(
        self, workflow_id: str, agent_kind: AgentKind, prompt: str
    ) -> None:
        """Run agent in background and stream output.

        Args:
            workflow_id: Workflow ID
            agent_kind: Kind of agent to run
            prompt: Prompt for the agent
        """
        try:
            # Simulate agent execution (placeholder)
            # In real implementation, this would call Claude CLI or SDK
            await asyncio.sleep(0.1)  # Simulate work

            # Send stream deltas
            await self.msg_sender.send(
                AgentStreamDelta(
                    workflow_id=workflow_id,
                    delta=f"Agent {agent_kind} started with prompt: {prompt}\n",
                )
            )

            # Simulate some output
            await asyncio.sleep(0.1)
            await self.msg_sender.send(
                AgentStreamDelta(workflow_id=workflow_id, delta="Processing...\n")
            )

            # Send completion
            await self.msg_sender.send(
                AgentCompleted(workflow_id=workflow_id, output="Agent completed")
            )

        except Exception as e:
            await self.msg_sender.send(
                WorkflowFailed(workflow_id=workflow_id, error=str(e))
            )
        finally:
            if workflow_id in self.active_tasks:
                del self.active_tasks[workflow_id]

    async def _execute_cancel_agent(self, effect: CancelAgent) -> None:
        """Cancel a running agent.

        Args:
            effect: CancelAgent effect
        """
        if effect.workflow_id in self.active_tasks:
            task = self.active_tasks[effect.workflow_id]
            task.cancel()
            # Suppress CancelledError since we initiated the cancellation
            try:
                await task
            except asyncio.CancelledError:
                pass  # noqa: SIM105
            del self.active_tasks[effect.workflow_id]

        await self.msg_sender.send(WorkflowCancelled(workflow_id=effect.workflow_id))

    async def _execute_write_file(self, effect: WriteFile) -> None:
        """Write file to disk.

        Args:
            effect: WriteFile effect
        """
        try:
            effect.path.parent.mkdir(parents=True, exist_ok=True)
            effect.path.write_text(effect.contents)
            await self.msg_sender.send(FileWritten(path=effect.path))
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to write {effect.path}: {e}")
            )

    async def _execute_read_file(self, effect: ReadFile) -> None:
        """Read file from disk.

        Args:
            effect: ReadFile effect
        """
        try:
            contents = effect.path.read_text()
            await self.msg_sender.send(
                FileReadCompleted(path=effect.path, contents=contents)
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to read {effect.path}: {e}")
            )

    async def _execute_delete_file(self, effect: DeleteFile) -> None:
        """Delete file from disk.

        Args:
            effect: DeleteFile effect
        """
        try:
            effect.path.unlink(missing_ok=True)
            await self.msg_sender.send(
                EffectCompleted(effect_id="delete_file", result=str(effect.path))
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to delete {effect.path}: {e}")
            )

    async def _execute_run_command(self, effect: RunCommand) -> None:
        """Run shell command.

        Args:
            effect: RunCommand effect
        """
        try:
            result = subprocess.run(
                [effect.cmd, *effect.args],
                cwd=effect.cwd,
                capture_output=True,
                text=True,
                timeout=30,
            )
            await self.msg_sender.send(
                CommandCompleted(
                    exit_code=result.returncode,
                    stdout=result.stdout,
                    stderr=result.stderr,
                )
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to run command {effect.cmd}: {e}")
            )

    async def _execute_run_bash_script(self, effect: RunBashScript) -> None:
        """Run bash script.

        Args:
            effect: RunBashScript effect
        """
        try:
            result = subprocess.run(
                ["bash", str(effect.script_path), *effect.args],
                capture_output=True,
                text=True,
                timeout=30,
            )
            await self.msg_sender.send(
                CommandCompleted(
                    exit_code=result.returncode,
                    stdout=result.stdout,
                    stderr=result.stderr,
                )
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(
                    message=f"Failed to run script {effect.script_path}: {e}"
                )
            )

    async def _execute_start_timer(self, effect: StartTimer) -> None:
        """Start a timer.

        Args:
            effect: StartTimer effect
        """
        # Cancel existing timer if any
        if effect.timer_id in self.timers:
            self.timers[effect.timer_id].cancel()

        # Create new timer task
        task = asyncio.create_task(self._run_timer(effect.timer_id, effect.delay_ms))
        self.timers[effect.timer_id] = task

    async def _run_timer(self, timer_id: str, delay_ms: int) -> None:
        """Run timer and send tick message.

        Args:
            timer_id: Timer ID
            delay_ms: Delay in milliseconds
        """
        try:
            while True:
                await asyncio.sleep(delay_ms / 1000.0)
                from rstn.msg import Tick

                await self.msg_sender.send(Tick())
        except asyncio.CancelledError:
            pass
        finally:
            if timer_id in self.timers:
                del self.timers[timer_id]

    async def _execute_stop_timer(self, effect: StopTimer) -> None:
        """Stop a timer.

        Args:
            effect: StopTimer effect
        """
        if effect.timer_id in self.timers:
            self.timers[effect.timer_id].cancel()
            del self.timers[effect.timer_id]

    async def _execute_cancel_workflow(self, effect: CancelWorkflow) -> None:
        """Cancel a workflow.

        Args:
            effect: CancelWorkflow effect
        """
        # Cancel associated agent if running
        if effect.workflow_id in self.active_tasks:
            task = self.active_tasks[effect.workflow_id]
            task.cancel()
            # Suppress CancelledError since we initiated the cancellation
            try:
                await task
            except asyncio.CancelledError:
                pass  # noqa: SIM105

        await self.msg_sender.send(WorkflowCancelled(workflow_id=effect.workflow_id))

    async def _execute_save_state(self, effect: SaveState) -> None:
        """Save application state to file.

        Args:
            effect: SaveState effect
        """
        try:
            effect.state.save_to_file(effect.path)
            await self.msg_sender.send(StateSaved(path=effect.path))
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to save state: {e}")
            )

    async def _execute_load_state(self, effect: LoadState) -> None:
        """Load application state from file.

        Args:
            effect: LoadState effect
        """
        try:
            from rstn.state import AppState

            # Load state from file
            AppState.load_from_file(effect.path)
            # In real implementation, would send StateLoaded message with state
            await self.msg_sender.send(
                EffectCompleted(effect_id="load_state", result=str(effect.path))
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to load state: {e}")
            )

    async def _execute_log_info(self, effect: LogInfo) -> None:
        """Log info message.

        Args:
            effect: LogInfo effect
        """
        # In real implementation, would use proper logging
        print(f"INFO: {effect.message}")

    async def _execute_log_error(self, effect: LogError) -> None:
        """Log error message.

        Args:
            effect: LogError effect
        """
        # In real implementation, would use proper logging
        print(f"ERROR: {effect.message}")

    async def _execute_log_debug(self, effect: LogDebug) -> None:
        """Log debug message.

        Args:
            effect: LogDebug effect
        """
        # In real implementation, would use proper logging
        print(f"DEBUG: {effect.message}")

    async def _execute_render(self, effect: Render) -> None:
        """Trigger UI render.

        Args:
            effect: Render effect
        """
        # In real implementation, would trigger TUI render
        # For now, this is a no-op in the executor
        pass

    async def _execute_quit_app(self, effect: QuitApp) -> None:
        """Quit application.

        Args:
            effect: QuitApp effect
        """
        from rstn.msg import Quit

        await self.msg_sender.send(Quit())

    async def _execute_copy_to_clipboard(self, effect: CopyToClipboard) -> None:
        """Copy content to system clipboard.

        Args:
            effect: CopyToClipboard effect
        """
        import platform
        import shutil

        try:
            system = platform.system()

            if system == "Darwin":  # macOS
                proc = subprocess.Popen(
                    ["pbcopy"],
                    stdin=subprocess.PIPE,
                )
                proc.communicate(effect.content.encode("utf-8"))
            elif system == "Linux":
                # Try xclip first, then xsel
                if shutil.which("xclip"):
                    proc = subprocess.Popen(
                        ["xclip", "-selection", "clipboard"],
                        stdin=subprocess.PIPE,
                    )
                    proc.communicate(effect.content.encode("utf-8"))
                elif shutil.which("xsel"):
                    proc = subprocess.Popen(
                        ["xsel", "--clipboard", "--input"],
                        stdin=subprocess.PIPE,
                    )
                    proc.communicate(effect.content.encode("utf-8"))
                else:
                    await self.msg_sender.send(
                        ErrorOccurred(
                            message="No clipboard tool found (xclip or xsel)"
                        )
                    )
                    return
            elif system == "Windows":
                # Use clip.exe on Windows
                proc = subprocess.Popen(
                    ["clip"],
                    stdin=subprocess.PIPE,
                )
                proc.communicate(effect.content.encode("utf-8"))
            else:
                await self.msg_sender.send(
                    ErrorOccurred(message=f"Unsupported platform: {system}")
                )
                return

            await self.msg_sender.send(
                EffectCompleted(
                    effect_id="copy_to_clipboard",
                    result=f"Copied {len(effect.content)} characters to clipboard",
                )
            )
        except Exception as e:
            await self.msg_sender.send(
                ErrorOccurred(message=f"Failed to copy to clipboard: {e}")
            )

    async def cleanup(self) -> None:
        """Cleanup all running tasks and timers."""
        # Cancel all active tasks
        for task in self.active_tasks.values():
            task.cancel()

        # Cancel all timers
        for task in self.timers.values():
            task.cancel()

        # Wait for all tasks to complete
        all_tasks = list(self.active_tasks.values()) + list(self.timers.values())
        if all_tasks:
            await asyncio.gather(*all_tasks, return_exceptions=True)

        self.active_tasks.clear()
        self.timers.clear()

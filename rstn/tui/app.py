"""Main TUI application using Textual framework.

This is the main entry point for the TUI. It implements the State-First MVI pattern:
1. Events are converted to AppMsg
2. reduce(state, msg) → (new_state, effects)
3. Effects are executed by EffectExecutor
4. UI is re-rendered based on new state
"""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import Any

from textual import events, work
from textual.app import App, ComposeResult
from textual.containers import Container, Vertical
from textual.widgets import Footer, Header, Label, Static

from rstn.effect import DefaultEffectExecutor, MessageSender
from rstn.msg import AppMsg, KeyModifiers, KeyPressed, MouseClicked, Quit, Tick
from rstn.reduce import reduce
from rstn.state import AppState


class RstnApp(App[None]):
    """Main rstn TUI application.

    Implements State-First MVI architecture:
    - All state in AppState (serializable)
    - Events → AppMsg → reduce() → (State, Effects)
    - Effects executed by EffectExecutor
    - UI = render(State)
    """

    CSS = """
    Screen {
        layout: vertical;
    }

    #main-container {
        layout: horizontal;
        height: 1fr;
    }

    #command-list {
        width: 30%;
        border: solid $primary;
    }

    #content-area {
        width: 70%;
        border: solid $secondary;
    }

    .status-bar {
        height: 1;
        background: $boost;
    }
    """

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("1", "switch_view('worktree')", "Worktree"),
        ("2", "switch_view('dashboard')", "Dashboard"),
        ("3", "switch_view('settings')", "Settings"),
        ("j", "next_command", "Next"),
        ("k", "prev_command", "Prev"),
    ]

    def __init__(
        self,
        state_file: Path | None = None,
        **kwargs: Any,
    ) -> None:
        """Initialize TUI app.

        Args:
            state_file: Optional path to saved state file
            **kwargs: Additional arguments for Textual App
        """
        super().__init__(**kwargs)

        # Load or create initial state
        if state_file and state_file.exists():
            self.state = AppState.load_from_file(state_file)
        else:
            self.state = AppState()

        # Message queue for executor feedback
        self._msg_queue: asyncio.Queue[AppMsg] = asyncio.Queue()

        # Effect executor
        self._executor = DefaultEffectExecutor(self._create_message_sender())

        # Timer task
        self._timer_task: asyncio.Task[None] | None = None

    def _create_message_sender(self) -> MessageSender:
        """Create message sender for executor feedback."""

        class QueueMessageSender:
            def __init__(self, queue: asyncio.Queue[AppMsg]) -> None:
                self.queue = queue

            async def send(self, msg: AppMsg) -> None:
                await self.queue.put(msg)

        return QueueMessageSender(self._msg_queue)

    def compose(self) -> ComposeResult:
        """Compose the UI layout."""
        yield Header()
        with Container(id="main-container"):
            with Vertical(id="command-list"):
                yield Static("Commands", classes="title")
                yield Label("No commands", id="command-label")
            with Vertical(id="content-area"):
                yield Static("Content", classes="title")
                yield Label("Empty", id="content-label")
        yield Static(
            self.state.error_message or "Ready", classes="status-bar", id="status-bar"
        )
        yield Footer()

    async def on_mount(self) -> None:
        """Handle mount event - start background tasks."""
        # Start message processing loop
        self.process_messages()

        # Start tick timer if needed
        self._start_tick_timer()

    def _start_tick_timer(self) -> None:
        """Start background tick timer."""

        async def tick_loop() -> None:
            while self.state.running:
                await asyncio.sleep(0.1)  # 10 FPS
                await self._msg_queue.put(Tick())

        self._timer_task = asyncio.create_task(tick_loop())

    @work(exclusive=True)
    async def process_messages(self) -> None:
        """Process messages from executor feedback queue."""
        while True:
            try:
                msg = await asyncio.wait_for(self._msg_queue.get(), timeout=0.1)
                await self._handle_message(msg)
            except TimeoutError:
                continue
            except Exception as e:
                self.log(f"Error processing message: {e}")

    async def _handle_message(self, msg: AppMsg) -> None:
        """Handle a message by running it through reduce.

        Args:
            msg: Message to handle
        """
        # Run through reducer
        new_state, effects = reduce(self.state, msg)

        # Update state
        old_state = self.state
        self.state = new_state

        # Execute effects
        for effect in effects:
            await self._executor.execute(effect)

        # Re-render if state changed
        if new_state != old_state:
            self._update_ui()

        # Handle quit
        if isinstance(msg, Quit) or not self.state.running:
            await self._cleanup()
            self.exit()

    def _update_ui(self) -> None:
        """Update UI based on current state."""
        try:
            # Update status bar
            status_bar = self.query_one("#status-bar", Static)
            status_bar.update(self.state.error_message or "Ready")

            # Update command list
            command_label = self.query_one("#command-label", Label)
            if self.state.worktree_view.commands:
                cmd_text = "\n".join(
                    f"{'>' if i == self.state.worktree_view.selected_command_index else ' '} {cmd.label}"
                    for i, cmd in enumerate(self.state.worktree_view.commands)
                )
                command_label.update(cmd_text)
            else:
                command_label.update("No commands")

            # Update content area
            content_label = self.query_one("#content-label", Label)
            content_type = self.state.worktree_view.content_type
            if content_type.value == "spec" and self.state.worktree_view.spec_content:
                content_label.update(self.state.worktree_view.spec_content)
            elif content_type.value == "plan" and self.state.worktree_view.plan_content:
                content_label.update(self.state.worktree_view.plan_content)
            elif self.state.worktree_view.workflow_output:
                content_label.update(self.state.worktree_view.workflow_output)
            else:
                content_label.update("Empty")

        except Exception as e:
            self.log(f"Error updating UI: {e}")

    async def _cleanup(self) -> None:
        """Cleanup resources before exit."""
        # Cancel timer
        if self._timer_task:
            self._timer_task.cancel()
            try:
                await self._timer_task
            except asyncio.CancelledError:
                pass

        # Cleanup executor
        await self._executor.cleanup()

    # Event handlers

    async def on_key(self, event: events.Key) -> None:
        """Handle key press events.

        Args:
            event: Key event
        """
        # Parse modifiers from key string (e.g., "ctrl+c" → ctrl=True, key="c")
        key_str = event.key
        modifiers = KeyModifiers(
            ctrl="ctrl+" in key_str.lower(),
            shift="shift+" in key_str.lower(),
            alt="alt+" in key_str.lower(),
        )

        # Remove modifier prefixes from key
        clean_key = key_str
        for prefix in ["ctrl+", "shift+", "alt+"]:
            clean_key = clean_key.lower().replace(prefix, "")

        # Create message
        msg = KeyPressed(key=clean_key, modifiers=modifiers)

        # Handle message
        await self._handle_message(msg)

    async def on_click(self, event: events.Click) -> None:
        """Handle mouse click events.

        Args:
            event: Click event
        """
        msg = MouseClicked(x=event.x, y=event.y)
        await self._handle_message(msg)

    # Actions (bound to keys)

    async def action_quit(self) -> None:
        """Quit the application."""
        await self._handle_message(Quit())

    async def action_switch_view(self, view: str) -> None:
        """Switch to a different view.

        Args:
            view: View name
        """
        from rstn.msg import SwitchView
        from rstn.state.types import ViewType

        view_map = {
            "worktree": ViewType.WORKTREE,
            "dashboard": ViewType.DASHBOARD,
            "settings": ViewType.SETTINGS,
        }

        if view in view_map:
            msg = SwitchView(view=view_map[view])
            await self._handle_message(msg)

    async def action_next_command(self) -> None:
        """Select next command."""
        from rstn.msg import SelectCommand

        current = self.state.worktree_view.selected_command_index
        count = len(self.state.worktree_view.commands)
        if count > 0:
            next_idx = min(current + 1, count - 1)
            msg = SelectCommand(index=next_idx)
            await self._handle_message(msg)

    async def action_prev_command(self) -> None:
        """Select previous command."""
        from rstn.msg import SelectCommand

        current = self.state.worktree_view.selected_command_index
        if current > 0:
            msg = SelectCommand(index=current - 1)
            await self._handle_message(msg)


def run_tui(state_file: Path | None = None) -> None:
    """Run the TUI application.

    Args:
        state_file: Optional path to saved state file
    """
    app = RstnApp(state_file=state_file)
    app.run()

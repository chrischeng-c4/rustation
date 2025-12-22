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
from textual.widgets import Footer, Header, Static

from rstn.effect import DefaultEffectExecutor, MessageSender
from rstn.logging import get_logger
from rstn.msg import AppMsg, KeyModifiers, KeyPressed, MouseClicked, Quit, Tick
from rstn.reduce import reduce
from rstn.state import AppState
from rstn.tui.render import render_app
from rstn.tui.render.widgets import (
    CommandListWidget,
    ContentAreaWidget,
    StatusBarWidget,
)

log = get_logger("rstn.tui")


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

    #command-panel {
        width: 30%;
        border: solid $primary;
    }

    #content-panel {
        width: 70%;
        border: solid $secondary;
    }

    #command-list {
        height: 100%;
    }

    #content-area {
        height: 100%;
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
        """Compose the UI layout.

        Uses custom widgets that accept render output from pure render functions.
        Layout: 30% command list | 70% content area
        """
        yield Header()
        with Container(id="main-container"):
            with Vertical(id="command-panel"):
                yield Static("Commands", classes="title")
                yield CommandListWidget(id="command-list")
            with Vertical(id="content-panel"):
                yield Static("Content", classes="title")
                yield ContentAreaWidget(id="content-area")
        yield StatusBarWidget(id="status-bar")
        yield Footer()

    async def on_mount(self) -> None:
        """Handle mount event - start background tasks."""
        log.info("TUI mounted")

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
                log.exception("Error processing message", error=str(e))

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
        """Update UI based on current state.

        Uses pure render functions to generate output, then applies to widgets.
        UI = render(State)
        """
        try:
            # Generate render output (pure function)
            render_output = render_app(self.state)

            # Apply to widgets
            command_list = self.query_one("#command-list", CommandListWidget)
            command_list.update_from_render(render_output.command_list)

            content_area = self.query_one("#content-area", ContentAreaWidget)
            content_area.update_from_render(render_output.content_area)

            status_bar = self.query_one("#status-bar", StatusBarWidget)
            status_bar.update_from_render(render_output.status_bar)

        except Exception as e:
            log.exception("Error updating UI", error=str(e))

    async def _cleanup(self) -> None:
        """Cleanup resources before exit."""
        log.info("TUI cleanup started")

        # Cancel timer
        if self._timer_task:
            self._timer_task.cancel()
            try:
                await self._timer_task
            except asyncio.CancelledError:
                pass

        # Cleanup executor
        await self._executor.cleanup()
        log.info("TUI cleanup complete")

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
    log.info("Starting TUI", state_file=str(state_file) if state_file else None)
    try:
        app = RstnApp(state_file=state_file)
        app.run()
        log.info("TUI exited normally")
    except Exception as e:
        log.exception("TUI crashed", error=str(e))
        raise

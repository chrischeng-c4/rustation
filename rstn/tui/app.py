"""Main TUI application using Textual framework.

This is the main entry point for the TUI. It implements the State-First MVI pattern:
1. Events are converted to AppMsg
2. reduce(state, msg) → (new_state, effects)
3. Effects are executed by EffectExecutor
4. UI is re-rendered based on new state
"""

from __future__ import annotations

import asyncio
import uuid
from pathlib import Path
from typing import Any

from textual import events, work
from textual.app import App, ComposeResult
from textual.containers import Container, Vertical
from textual.widgets import Header, Static

from rstn.effect import DefaultEffectExecutor, MessageSender
from rstn.logging import get_logger
from rstn.mcp import McpServer, McpServerConfig
from rstn.msg import AppMsg, KeyModifiers, KeyPressed, MouseClicked, Quit, Tick
from rstn.reduce import reduce
from rstn.state import AppState
from rstn.tui.render import render_app
from rstn.tui.render.widgets import (
    CommandListWidget,
    ContentAreaWidget,
    FooterWidget,
    StatusBarWidget,
    TabBarWidget,
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

    #tab-bar {
        height: 1;
        background: $surface;
        padding: 0 1;
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

    #custom-footer {
        height: 1;
        background: $boost;
        padding: 0 1;
        text-style: dim;
    }
    """

    # Note: Key bindings are handled through the reducer (reduce_key_pressed)
    # We don't use Textual's BINDINGS to avoid duplicate handling
    BINDINGS = []

    def __init__(
        self,
        state_file: Path | None = None,
        project_root: Path | None = None,
        enable_mcp: bool = True,
        **kwargs: Any,
    ) -> None:
        """Initialize TUI app.

        Args:
            state_file: Optional path to saved state file
            project_root: Project root directory (defaults to cwd)
            enable_mcp: Whether to start MCP server (default True)
            **kwargs: Additional arguments for Textual App
        """
        super().__init__(**kwargs)

        # Project root for MCP and spec operations
        self._project_root = project_root or Path.cwd()

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

        # Flag to prevent duplicate quit processing
        self._quitting = False

        # MCP server (optional)
        self._enable_mcp = enable_mcp
        self._mcp_server: McpServer | None = None
        self._mcp_session_id = str(uuid.uuid4())[:8]  # Short session ID

    def _create_message_sender(self) -> MessageSender:
        """Create message sender for executor feedback."""

        class QueueMessageSender:
            def __init__(self, queue: asyncio.Queue[AppMsg]) -> None:
                self.queue = queue

            async def send(self, msg: AppMsg) -> None:
                await self.queue.put(msg)

        return QueueMessageSender(self._msg_queue)

    async def _mcp_msg_sender(self, msg: AppMsg) -> None:
        """Send message from MCP server to TUI queue."""
        await self._msg_queue.put(msg)

    def _get_state(self) -> AppState:
        """Get current state (callback for MCP server)."""
        return self.state

    def compose(self) -> ComposeResult:
        """Compose the UI layout.

        Uses custom widgets that accept render output from pure render functions.
        Layout: Header | TabBar | (30% Sidebar | 70% Content) | StatusBar | Footer
        """
        yield Header()
        yield TabBarWidget(id="tab-bar")
        with Container(id="main-container"):
            with Vertical(id="command-panel"):
                yield Static("Commands", classes="title")
                yield CommandListWidget(id="command-list")
            with Vertical(id="content-panel"):
                yield Static("Content", classes="title")
                yield ContentAreaWidget(id="content-area")
        yield StatusBarWidget(id="status-bar")
        yield FooterWidget(id="custom-footer")

    async def on_mount(self) -> None:
        """Handle mount event - start background tasks."""
        log.info("TUI mounted")

        # Initial UI render
        self._update_ui()

        # Start message processing loop
        self.process_messages()

        # Start tick timer if needed
        self._start_tick_timer()

        # Start MCP server if enabled
        if self._enable_mcp:
            await self._start_mcp_server()

    def _start_tick_timer(self) -> None:
        """Start background tick timer."""

        async def tick_loop() -> None:
            while self.state.running:
                await asyncio.sleep(0.1)  # 10 FPS
                await self._msg_queue.put(Tick())

        self._timer_task = asyncio.create_task(tick_loop())

    async def _start_mcp_server(self) -> None:
        """Start the embedded MCP HTTP server."""
        from rstn.msg import McpServerStarted

        config = McpServerConfig(
            host="127.0.0.1",
            port=0,  # Dynamic port
            session_id=self._mcp_session_id,
        )

        self._mcp_server = McpServer(
            config=config,
            state_getter=self._get_state,
            msg_sender=self._mcp_msg_sender,
            project_root=self._project_root,
        )

        port = await self._mcp_server.start()
        log.info(
            "MCP server started",
            port=port,
            session_id=self._mcp_session_id,
        )

        # Notify TUI via message
        await self._msg_queue.put(
            McpServerStarted(port=port, session_id=self._mcp_session_id)
        )

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
        # Skip if already quitting
        if self._quitting:
            return

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
            self._quitting = True
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
            tab_bar = self.query_one("#tab-bar", TabBarWidget)
            tab_bar.update_from_render(render_output.tab_bar)

            command_list = self.query_one("#command-list", CommandListWidget)
            command_list.update_from_render(render_output.command_list)

            content_area = self.query_one("#content-area", ContentAreaWidget)
            content_area.update_from_render(render_output.content_area)

            status_bar = self.query_one("#status-bar", StatusBarWidget)
            status_bar.update_from_render(render_output.status_bar)

            footer = self.query_one("#custom-footer", FooterWidget)
            footer.update_from_render(render_output.footer)

        except Exception as e:
            log.exception("Error updating UI", error=str(e))

    async def _cleanup(self) -> None:
        """Cleanup resources before exit."""
        log.info("TUI cleanup started")

        # Stop MCP server
        if self._mcp_server:
            await self._mcp_server.stop()
            self._mcp_server = None

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

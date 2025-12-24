"""MCP HTTP server for rstn.

Embeds a FastAPI server within the TUI process to handle MCP tool requests
from Claude Code.
"""

from __future__ import annotations

import asyncio
import socket
from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import TYPE_CHECKING

from fastapi import FastAPI

from rstn.logging import get_logger
from rstn.mcp.types import McpServerConfig

if TYPE_CHECKING:
    from rstn.msg import AppMsg
    from rstn.state import AppState

log = get_logger("rstn.mcp.server")


class McpServer:
    """Embedded MCP HTTP server for Claude Code communication.

    Runs alongside the TUI event loop, providing HTTP endpoints
    for MCP protocol communication.
    """

    def __init__(
        self,
        config: McpServerConfig,
        state_getter: Callable[[], AppState],
        msg_sender: Callable[[AppMsg], Awaitable[None]],
        project_root: Path,
    ) -> None:
        """Initialize MCP server.

        Args:
            config: Server configuration
            state_getter: Callback to get current AppState
            msg_sender: Callback to send AppMsg to TUI queue
            project_root: Project root for spec/hook operations
        """
        self.config = config
        self.state_getter = state_getter
        self.msg_sender = msg_sender
        self.project_root = project_root

        self._app: FastAPI | None = None
        self._server_task: asyncio.Task | None = None
        self._actual_port: int = 0
        self._shutdown_event: asyncio.Event = asyncio.Event()

    @property
    def port(self) -> int:
        """Get the actual port the server is listening on."""
        return self._actual_port

    @property
    def app(self) -> FastAPI:
        """Get the FastAPI app instance."""
        if self._app is None:
            self._app = self._create_app()
        return self._app

    def _create_app(self) -> FastAPI:
        """Create and configure the FastAPI application."""
        from rstn.mcp.routes import create_router

        app = FastAPI(
            title=f"rstn-mcp-{self.config.session_id}",
            description="rstn MCP Server for Claude Code integration",
            version="0.1.0",
        )

        # Create router with dependencies injected
        router = create_router(
            state_getter=self.state_getter,
            msg_sender=self.msg_sender,
            project_root=self.project_root,
        )
        app.include_router(router)

        return app

    def _find_free_port(self) -> int:
        """Find a free port for the server."""
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(("", 0))
            s.listen(1)
            port = s.getsockname()[1]
        return port

    async def start(self) -> int:
        """Start the MCP HTTP server.

        Returns:
            The port the server is listening on
        """
        import uvicorn

        # Determine port
        if self.config.port == 0:
            self._actual_port = self._find_free_port()
        else:
            self._actual_port = self.config.port

        log.info(
            "Starting MCP server",
            host=self.config.host,
            port=self._actual_port,
            session_id=self.config.session_id,
        )

        # Configure uvicorn
        uvicorn_config = uvicorn.Config(
            app=self.app,
            host=self.config.host,
            port=self._actual_port,
            log_level="warning",  # Reduce uvicorn noise
            access_log=False,
        )

        server = uvicorn.Server(uvicorn_config)

        # Run server in background task
        self._server_task = asyncio.create_task(
            self._run_server(server),
            name=f"mcp-server-{self.config.session_id}",
        )

        # Wait a bit for server to start
        await asyncio.sleep(0.1)

        log.info("MCP server started", port=self._actual_port)
        return self._actual_port

    async def _run_server(self, server) -> None:
        """Run the uvicorn server until shutdown."""
        try:
            await server.serve()
        except asyncio.CancelledError:
            log.debug("MCP server task cancelled")
        except Exception as e:
            log.error("MCP server error", error=str(e))

    async def stop(self) -> None:
        """Stop the MCP server gracefully."""
        log.info("Stopping MCP server", port=self._actual_port)

        if self._server_task:
            self._server_task.cancel()
            try:
                await asyncio.wait_for(self._server_task, timeout=5.0)
            except (TimeoutError, asyncio.CancelledError):
                pass
            self._server_task = None

        log.info("MCP server stopped")

    def get_mcp_config_dict(self) -> dict:
        """Get MCP config dict for Claude CLI.

        Returns:
            Dict suitable for mcp-config.json
        """
        return {
            "mcpServers": {
                "rstn": {
                    "type": "http",
                    "url": f"http://{self.config.host}:{self._actual_port}/mcp",
                }
            }
        }

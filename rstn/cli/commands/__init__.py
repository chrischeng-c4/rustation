"""CLI commands for rstn.

Each command module implements a specific CLI command or command group.
All commands use the reduce() pattern to share logic with TUI.
"""

from rstn.cli.commands.clarify import clarify
from rstn.cli.commands.plan import plan
from rstn.cli.commands.prompt import prompt
from rstn.cli.commands.session import session
from rstn.cli.commands.specify import specify
from rstn.cli.commands.state import state
from rstn.cli.commands.tui import tui

__all__ = [
    "tui",
    "prompt",
    "specify",
    "plan",
    "clarify",
    "session",
    "state",
]

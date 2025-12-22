"""TUI launcher command.

Starts the rstn TUI application.
"""

from __future__ import annotations

from pathlib import Path

import click


@click.command()
@click.option(
    "--state-file",
    type=click.Path(path_type=Path),
    default=None,
    help="Path to state file to load on startup",
)
@click.pass_context
def tui(ctx: click.Context, state_file: Path | None) -> None:
    """Start the rstn TUI.

    Launches the interactive TUI for workflow management.

    Examples:

        rstn tui

        rstn tui --state-file ~/.rstn/state.json
    """
    from rstn.tui.app import run_tui

    # Use state file from context if not provided
    if state_file is None:
        state_file = ctx.obj.get("state_file")

    click.echo("Starting rstn TUI...")
    run_tui(state_file=state_file)

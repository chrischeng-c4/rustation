"""Session management commands.

Provides CLI commands for managing rstn sessions:
- list: List all sessions
- show: Show session details
- delete: Delete a session
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

console = Console()

# Default sessions directory
SESSIONS_DIR = Path.home() / ".rstn" / "sessions"


@click.group()
def session() -> None:
    """Manage rstn sessions.

    Sessions store workflow state and history.

    Examples:

        rstn session list

        rstn session show abc123

        rstn session delete abc123
    """
    pass


@session.command("list")
@click.option(
    "--all", "-a",
    is_flag=True,
    default=False,
    help="Show all sessions including completed",
)
@click.option(
    "--limit", "-n",
    type=int,
    default=20,
    help="Maximum number of sessions to show",
)
@click.pass_context
def list_sessions(ctx: click.Context, all: bool, limit: int) -> None:
    """List all sessions.

    Shows active sessions by default. Use --all to include completed.

    Examples:

        rstn session list

        rstn session list --all

        rstn session list -n 50
    """
    verbose: bool = ctx.obj.get("verbose", False)

    if verbose:
        console.print(f"[dim]Sessions dir: {SESSIONS_DIR}[/dim]")

    if not SESSIONS_DIR.exists():
        console.print("[yellow]No sessions found[/yellow]")
        return

    # Find session files
    session_files = sorted(
        SESSIONS_DIR.glob("*.json"),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )

    if not session_files:
        console.print("[yellow]No sessions found[/yellow]")
        return

    # Build table
    table = Table(title="Sessions")
    table.add_column("ID", style="cyan")
    table.add_column("Status", style="green")
    table.add_column("Created", style="dim")
    table.add_column("Workflows", justify="right")

    count = 0
    for session_file in session_files:
        if count >= limit:
            break

        session_data = _load_session(session_file)
        if session_data is None:
            continue

        status = str(session_data.get("status", "unknown"))
        if not all and status == "completed":
            continue

        session_id = session_file.stem
        created_at = session_data.get("created_at", "")
        created = _format_timestamp(str(created_at) if created_at else "")
        workflows = session_data.get("workflows", [])
        workflow_count = len(workflows) if isinstance(workflows, list) else 0

        table.add_row(
            session_id[:12] + "..." if len(session_id) > 15 else session_id,
            status,
            created,
            str(workflow_count),
        )
        count += 1

    console.print(table)
    console.print(f"\n[dim]Showing {count} of {len(session_files)} sessions[/dim]")


@session.command("show")
@click.argument("session_id")
@click.option(
    "--json", "-j",
    "output_json",
    is_flag=True,
    default=False,
    help="Output as JSON",
)
@click.pass_context
def show_session(ctx: click.Context, session_id: str, output_json: bool) -> None:
    """Show session details.

    Displays detailed information about a specific session.

    Examples:

        rstn session show abc123

        rstn session show abc123 --json
    """
    session_file = _find_session(session_id)

    if session_file is None:
        console.print(f"[red]Session not found: {session_id}[/red]")
        raise SystemExit(1) from None

    session_data = _load_session(session_file)
    if session_data is None:
        console.print(f"[red]Failed to load session: {session_id}[/red]")
        raise SystemExit(1) from None

    if output_json:
        import json
        console.print(json.dumps(session_data, indent=2, default=str))
        return

    # Display session details
    console.print(Panel(
        f"[bold]Session ID:[/bold] {session_file.stem}\n"
        f"[bold]Status:[/bold] {session_data.get('status', 'unknown')}\n"
        f"[bold]Created:[/bold] {session_data.get('created_at', 'unknown')}\n"
        f"[bold]Updated:[/bold] {session_data.get('updated_at', 'unknown')}",
        title="Session Details",
        border_style="blue",
    ))

    # Show workflows
    workflows = session_data.get("workflows", [])
    if isinstance(workflows, list) and workflows:
        console.print("\n[bold]Workflows:[/bold]")
        for wf in workflows:
            if isinstance(wf, dict):
                success = wf.get("success", False)
                wf_type = wf.get("type", "unknown")
                status_icon = "[green]OK[/green]" if success else "[red]FAIL[/red]"
                console.print(f"  - {wf_type}: {status_icon}")

    # Show state summary
    state = session_data.get("state", {})
    if isinstance(state, dict) and state:
        console.print("\n[bold]State:[/bold]")
        console.print(f"  Current view: {state.get('current_view', 'unknown')}")
        active_wfs = state.get("active_workflows", [])
        wf_count = len(active_wfs) if isinstance(active_wfs, list) else 0
        console.print(f"  Active workflows: {wf_count}")


@session.command("delete")
@click.argument("session_id")
@click.option(
    "--force", "-f",
    is_flag=True,
    default=False,
    help="Delete without confirmation",
)
@click.pass_context
def delete_session(ctx: click.Context, session_id: str, force: bool) -> None:
    """Delete a session.

    Permanently removes session data. Use --force to skip confirmation.

    Examples:

        rstn session delete abc123

        rstn session delete abc123 --force
    """
    session_file = _find_session(session_id)

    if session_file is None:
        console.print(f"[red]Session not found: {session_id}[/red]")
        raise SystemExit(1) from None

    if not force and not click.confirm(f"Delete session {session_id}?"):
        console.print("[yellow]Cancelled[/yellow]")
        return

    try:
        session_file.unlink()
        console.print(f"[green]Session deleted: {session_id}[/green]")
    except Exception as e:
        console.print(f"[red]Error deleting session: {e}[/red]")
        raise SystemExit(1) from None


@session.command("clean")
@click.option(
    "--older-than", "-o",
    type=int,
    default=7,
    help="Delete sessions older than N days (default: 7)",
)
@click.option(
    "--completed-only", "-c",
    is_flag=True,
    default=False,
    help="Only delete completed sessions",
)
@click.option(
    "--dry-run", "-n",
    is_flag=True,
    default=False,
    help="Show what would be deleted without deleting",
)
@click.pass_context
def clean_sessions(
    ctx: click.Context,
    older_than: int,
    completed_only: bool,
    dry_run: bool,
) -> None:
    """Clean up old sessions.

    Deletes sessions older than specified days.

    Examples:

        rstn session clean

        rstn session clean --older-than 30

        rstn session clean --completed-only --dry-run
    """
    import time

    if not SESSIONS_DIR.exists():
        console.print("[yellow]No sessions directory[/yellow]")
        return

    cutoff_time = time.time() - (older_than * 24 * 60 * 60)
    to_delete: list[Path] = []

    for session_file in SESSIONS_DIR.glob("*.json"):
        if session_file.stat().st_mtime > cutoff_time:
            continue

        if completed_only:
            session_data = _load_session(session_file)
            if session_data and session_data.get("status") != "completed":
                continue

        to_delete.append(session_file)

    if not to_delete:
        console.print("[green]No sessions to clean[/green]")
        return

    console.print(f"Found {len(to_delete)} sessions to delete:")
    for sf in to_delete[:10]:  # Show first 10
        console.print(f"  - {sf.stem}")
    if len(to_delete) > 10:
        console.print(f"  ... and {len(to_delete) - 10} more")

    if dry_run:
        console.print("\n[yellow]Dry run - no files deleted[/yellow]")
        return

    if not click.confirm("Proceed?"):
        console.print("[yellow]Cancelled[/yellow]")
        return

    deleted = 0
    for sf in to_delete:
        try:
            sf.unlink()
            deleted += 1
        except Exception as e:
            console.print(f"[red]Error deleting {sf.stem}: {e}[/red]")

    console.print(f"[green]Deleted {deleted} sessions[/green]")


# Helper functions


def _find_session(session_id: str) -> Path | None:
    """Find session file by ID (supports partial match).

    Args:
        session_id: Session ID or prefix

    Returns:
        Path to session file or None
    """
    if not SESSIONS_DIR.exists():
        return None

    # Try exact match first
    exact = SESSIONS_DIR / f"{session_id}.json"
    if exact.exists():
        return exact

    # Try prefix match
    matches = list(SESSIONS_DIR.glob(f"{session_id}*.json"))
    if len(matches) == 1:
        return matches[0]
    elif len(matches) > 1:
        console.print(f"[yellow]Multiple matches for '{session_id}':[/yellow]")
        for m in matches[:5]:
            console.print(f"  - {m.stem}")
        return None

    return None


def _load_session(path: Path) -> dict[str, object] | None:
    """Load session data from file.

    Args:
        path: Path to session file

    Returns:
        Session data dict or None
    """
    import json

    try:
        data: dict[str, object] = json.loads(path.read_text())
        return data
    except Exception:
        return None


def _format_timestamp(timestamp: str) -> str:
    """Format timestamp for display.

    Args:
        timestamp: ISO timestamp string

    Returns:
        Formatted string
    """
    if not timestamp:
        return "unknown"

    try:
        dt = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
        return dt.strftime("%Y-%m-%d %H:%M")
    except Exception:
        return timestamp[:16] if len(timestamp) > 16 else timestamp

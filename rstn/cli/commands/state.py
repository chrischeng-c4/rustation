"""State management commands.

Provides CLI commands for managing application state:
- save: Save current state to file
- load: Load state from file
- reset: Reset state to default
- show: Show current state
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.syntax import Syntax

from rstn.state import AppState

console = Console()

# Default state file location
DEFAULT_STATE_FILE = Path.home() / ".rstn" / "state.json"


@click.group()
def state() -> None:
    """Manage application state.

    State can be saved, loaded, and reset for debugging and persistence.

    Examples:

        rstn state show

        rstn state save ~/.rstn/backup.json

        rstn state reset
    """
    pass


@state.command("show")
@click.option(
    "--format", "-f",
    type=click.Choice(["json", "yaml", "summary"]),
    default="summary",
    help="Output format (default: summary)",
)
@click.pass_context
def show_state(ctx: click.Context, format: str) -> None:
    """Show current application state.

    Displays the current state in various formats.

    Examples:

        rstn state show

        rstn state show --format json

        rstn state show -f yaml
    """
    current_state: AppState = ctx.obj["state"]

    if format == "json":
        import json
        json_str = json.dumps(current_state.model_dump(), indent=2, default=str)
        syntax = Syntax(json_str, "json", theme="monokai", line_numbers=True)
        console.print(syntax)

    elif format == "yaml":
        import yaml
        yaml_str = yaml.dump(current_state.model_dump(), default_flow_style=False)
        syntax = Syntax(yaml_str, "yaml", theme="monokai", line_numbers=True)
        console.print(syntax)

    else:
        # Summary format
        console.print(Panel(
            f"[bold]Version:[/bold] {current_state.version}\n"
            f"[bold]Running:[/bold] {current_state.running}\n"
            f"[bold]Current View:[/bold] {current_state.current_view}\n"
            f"[bold]Session ID:[/bold] {current_state.session_id or 'None'}\n"
            f"[bold]Active Workflows:[/bold] {len(current_state.active_workflows)}\n"
            f"[bold]Mouse Enabled:[/bold] {current_state.mouse_enabled}",
            title="Application State",
            border_style="blue",
        ))

        # Show view states
        console.print("\n[bold]View States:[/bold]")

        # Worktree view
        wv = current_state.worktree_view
        console.print("  [cyan]Worktree View:[/cyan]")
        console.print(f"    Commands: {len(wv.commands)}")
        console.print(f"    Selected: {wv.selected_command_index}")

        # Dashboard view
        dv = current_state.dashboard_view
        console.print("  [cyan]Dashboard View:[/cyan]")
        console.print(f"    Recent workflows: {len(dv.recent_workflows)}")

        # Settings view
        sv = current_state.settings_view
        console.print("  [cyan]Settings View:[/cyan]")
        console.print(f"    Theme: {sv.theme}")


@state.command("save")
@click.argument(
    "path",
    type=click.Path(path_type=Path),
    default=str(DEFAULT_STATE_FILE),
)
@click.option(
    "--format", "-f",
    type=click.Choice(["json", "yaml"]),
    default="json",
    help="Output format (default: json)",
)
@click.pass_context
def save_state(ctx: click.Context, path: Path, format: str) -> None:
    """Save current state to file.

    Saves the application state for later restoration.

    Examples:

        rstn state save

        rstn state save ~/.rstn/backup.json

        rstn state save state.yaml --format yaml
    """
    current_state: AppState = ctx.obj["state"]

    # Ensure directory exists
    path.parent.mkdir(parents=True, exist_ok=True)

    # Adjust extension if needed
    if format == "yaml" and path.suffix not in [".yaml", ".yml"]:
        path = path.with_suffix(".yaml")
    elif format == "json" and path.suffix != ".json":
        path = path.with_suffix(".json")

    try:
        current_state.save_to_file(path)
        console.print(f"[green]State saved to:[/green] {path}")

        # Show file size
        size = path.stat().st_size
        console.print(f"[dim]Size: {size:,} bytes[/dim]")

    except Exception as e:
        console.print(f"[red]Error saving state: {e}[/red]")
        raise SystemExit(1) from None


@state.command("load")
@click.argument(
    "path",
    type=click.Path(exists=True, path_type=Path),
)
@click.option(
    "--dry-run", "-n",
    is_flag=True,
    default=False,
    help="Validate state without loading",
)
@click.pass_context
def load_state(ctx: click.Context, path: Path, dry_run: bool) -> None:
    """Load state from file.

    Restores application state from a previously saved file.

    Examples:

        rstn state load ~/.rstn/backup.json

        rstn state load state.yaml --dry-run
    """
    try:
        loaded = AppState.load_from_file(path)

        if dry_run:
            console.print(f"[green]State file is valid:[/green] {path}")
            console.print(f"[dim]Version: {loaded.version}[/dim]")
            console.print(f"[dim]Session: {loaded.session_id or 'None'}[/dim]")
            return

        # Update context state
        ctx.obj["state"] = loaded
        ctx.obj["state_file"] = path

        console.print(f"[green]State loaded from:[/green] {path}")
        console.print(f"[dim]Version: {loaded.version}[/dim]")
        console.print(f"[dim]Session: {loaded.session_id or 'None'}[/dim]")

    except Exception as e:
        console.print(f"[red]Error loading state: {e}[/red]")
        raise SystemExit(1) from None


@state.command("reset")
@click.option(
    "--force", "-f",
    is_flag=True,
    default=False,
    help="Reset without confirmation",
)
@click.pass_context
def reset_state(ctx: click.Context, force: bool) -> None:
    """Reset state to default.

    Creates a fresh application state with default values.

    Examples:

        rstn state reset

        rstn state reset --force
    """
    if not force and not click.confirm("Reset state to default values?"):
        console.print("[yellow]Cancelled[/yellow]")
        return

    ctx.obj["state"] = AppState()
    console.print("[green]State reset to default[/green]")


@state.command("diff")
@click.argument(
    "path",
    type=click.Path(exists=True, path_type=Path),
)
@click.pass_context
def diff_state(ctx: click.Context, path: Path) -> None:
    """Show differences between current and saved state.

    Compares the current state with a saved state file.

    Examples:

        rstn state diff ~/.rstn/backup.json
    """

    current_state: AppState = ctx.obj["state"]

    try:
        saved_state = AppState.load_from_file(path)
    except Exception as e:
        console.print(f"[red]Error loading state: {e}[/red]")
        raise SystemExit(1) from None

    current_dict = current_state.model_dump()
    saved_dict = saved_state.model_dump()

    differences = _find_differences(current_dict, saved_dict)

    if not differences:
        console.print("[green]States are identical[/green]")
        return

    console.print(Panel(
        f"Found {len(differences)} differences",
        title="State Diff",
        border_style="yellow",
    ))

    for path_str, (current_val, saved_val) in differences.items():
        console.print(f"\n[bold]{path_str}[/bold]")
        console.print(f"  [red]- {saved_val}[/red]")
        console.print(f"  [green]+ {current_val}[/green]")


@state.command("validate")
@click.argument(
    "path",
    type=click.Path(exists=True, path_type=Path),
)
def validate_state(path: Path) -> None:
    """Validate a state file.

    Checks that a state file is valid and can be loaded.

    Examples:

        rstn state validate ~/.rstn/state.json
    """
    try:
        loaded = AppState.load_from_file(path)

        # Run invariant checks
        loaded.assert_invariants()

        console.print(f"[green]State file is valid:[/green] {path}")
        console.print(f"[dim]Version: {loaded.version}[/dim]")
        console.print("[dim]Invariants: OK[/dim]")

    except Exception as e:
        console.print(f"[red]State validation failed: {e}[/red]")
        raise SystemExit(1) from None


def _find_differences(
    dict1: dict[str, object],
    dict2: dict[str, object],
    path: str = "",
) -> dict[str, tuple[object, object]]:
    """Find differences between two dictionaries.

    Args:
        dict1: First dictionary
        dict2: Second dictionary
        path: Current path in nested structure

    Returns:
        Dictionary of differences with paths as keys
    """
    differences: dict[str, tuple[object, object]] = {}

    all_keys = set(dict1.keys()) | set(dict2.keys())

    for key in all_keys:
        current_path = f"{path}.{key}" if path else key
        val1 = dict1.get(key)
        val2 = dict2.get(key)

        if val1 == val2:
            continue

        if isinstance(val1, dict) and isinstance(val2, dict):
            nested = _find_differences(val1, val2, current_path)
            differences.update(nested)
        else:
            differences[current_path] = (val1, val2)

    return differences

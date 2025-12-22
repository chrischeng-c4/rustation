"""Plan command for generating implementation plans.

Uses reduce() pattern to process plan generation workflow.
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel

from rstn.msg import PlanGenerationRequested
from rstn.reduce import reduce
from rstn.state import AppState

console = Console()


@click.command()
@click.argument("feature_name", required=False)
@click.option(
    "--spec-dir", "-s",
    type=click.Path(exists=True, path_type=Path),
    default=None,
    help="Path to spec directory (default: auto-detect from feature name)",
)
@click.option(
    "--output", "-o",
    type=click.Path(path_type=Path),
    default=None,
    help="Output path for plan file",
)
@click.option(
    "--context-files", "-c",
    multiple=True,
    type=click.Path(exists=True, path_type=Path),
    help="Additional context files to include",
)
@click.pass_context
def plan(
    ctx: click.Context,
    feature_name: str | None,
    spec_dir: Path | None,
    output: Path | None,
    context_files: tuple[Path, ...],
) -> None:
    """Generate an implementation plan.

    Creates a detailed implementation plan from a spec. If no feature name
    is provided, lists available specs to choose from.

    Examples:

        rstn plan 001-user-auth

        rstn plan --spec-dir specs/001-user-auth

        rstn plan 002-api -c src/api.rs -c src/types.rs
    """
    # Get state from context
    state: AppState = ctx.obj["state"]
    verbose: bool = ctx.obj.get("verbose", False)

    # If no feature name, list available specs
    if feature_name is None and spec_dir is None:
        _list_available_specs()
        feature_name = click.prompt("Enter feature name or number")

    # Resolve spec directory
    if spec_dir is None:
        spec_dir = _find_spec_dir(feature_name or "")

    if spec_dir is None or not spec_dir.exists():
        console.print(f"[red]Error: Spec directory not found for '{feature_name}'[/red]")
        raise SystemExit(1) from None

    # Get feature name from directory if not provided
    if feature_name is None:
        feature_name = spec_dir.name

    if verbose:
        console.print(f"[dim]Spec directory: {spec_dir}[/dim]")
        console.print(f"[dim]Context files: {len(context_files)}[/dim]")

    console.print(Panel(
        f"[bold]Generating plan for:[/bold]\n{feature_name}",
        title="Plan Workflow",
        border_style="green",
    ))

    # Create plan generation request message
    msg = PlanGenerationRequested(feature_name=feature_name)

    # Run through reducer
    new_state, effects = reduce(state, msg)

    # Execute the plan generation workflow
    _execute_plan_workflow(
        spec_dir=spec_dir,
        feature_name=feature_name,
        output=output,
        context_files=list(context_files),
        verbose=verbose,
    )


def _list_available_specs() -> None:
    """List available specs in the specs directory."""
    specs_dir = Path("specs")

    if not specs_dir.exists():
        console.print("[yellow]No specs directory found[/yellow]")
        return

    specs = sorted(specs_dir.glob("[0-9][0-9][0-9]-*"))
    if not specs:
        console.print("[yellow]No specs found in specs/[/yellow]")
        return

    console.print("\n[bold]Available specs:[/bold]")
    for spec in specs:
        has_plan = (spec / "plan.md").exists()
        status = "[green](has plan)[/green]" if has_plan else "[dim](no plan)[/dim]"
        console.print(f"  {spec.name} {status}")
    console.print()


def _find_spec_dir(feature_name: str) -> Path | None:
    """Find spec directory by feature name or number.

    Args:
        feature_name: Feature name or number to search for

    Returns:
        Path to spec directory or None if not found
    """
    specs_dir = Path("specs")

    if not specs_dir.exists():
        return None

    # Try exact match first
    exact = specs_dir / feature_name
    if exact.exists():
        return exact

    # Try matching by number prefix
    if feature_name.isdigit():
        pattern = f"{int(feature_name):03d}-*"
        matches = list(specs_dir.glob(pattern))
        if matches:
            return matches[0]

    # Try partial match
    for spec in specs_dir.iterdir():
        if feature_name.lower() in spec.name.lower():
            return spec

    return None


def _execute_plan_workflow(
    spec_dir: Path,
    feature_name: str,
    output: Path | None,
    context_files: list[Path],
    verbose: bool,
) -> None:
    """Execute the plan generation workflow.

    Args:
        spec_dir: Spec directory path
        feature_name: Feature name
        output: Output path for plan
        context_files: Additional context files
        verbose: Whether to show verbose output
    """
    # Use placeholder plan workflow
    # Domain module integration will be completed in Phase 7
    _create_placeholder_plan(spec_dir, feature_name, output, verbose)


def _create_placeholder_plan(
    spec_dir: Path,
    feature_name: str,
    output: Path | None,
    verbose: bool = False,
) -> None:
    """Create a placeholder plan when domain module is not available.

    Args:
        spec_dir: Spec directory
        feature_name: Feature name
        output: Optional output path
        verbose: Whether to show verbose output
    """
    plan_path = output or (spec_dir / "plan.md")

    # Read spec for context
    spec_file = spec_dir / "spec.md"
    spec_content = spec_file.read_text() if spec_file.exists() else ""

    content = f"""# Implementation Plan: {feature_name}

## Overview

This is a placeholder plan generated automatically.
Review and update before implementation.

## Source Spec

{spec_content[:500]}...

## Implementation Steps

1. [ ] Review spec requirements
2. [ ] Design architecture
3. [ ] Implement core functionality
4. [ ] Add tests
5. [ ] Documentation

## Files to Create/Modify

TBD

## Testing Strategy

TBD

## Notes

- Generated automatically
- Review and customize before use
"""

    plan_path.write_text(content)
    console.print(f"[green]Placeholder plan created:[/green] {plan_path}")

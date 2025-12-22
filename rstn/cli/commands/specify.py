"""Specify command for generating feature specifications.

Uses reduce() pattern to process spec generation workflow.
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel

from rstn.msg import (
    SpecGenerationRequested,
)
from rstn.reduce import reduce
from rstn.state import AppState

console = Console()


def _generate_workflow_id() -> str:
    """Generate a unique workflow ID."""
    import uuid
    return f"wf-{uuid.uuid4().hex[:12]}"


@click.command()
@click.argument("description", required=False)
@click.option(
    "--output-dir", "-o",
    type=click.Path(path_type=Path),
    default=None,
    help="Output directory for spec (default: specs/)",
)
@click.option(
    "--template", "-t",
    type=str,
    default="default",
    help="Spec template to use",
)
@click.option(
    "--interactive", "-i",
    is_flag=True,
    default=False,
    help="Run in interactive mode with clarify questions",
)
@click.pass_context
def specify(
    ctx: click.Context,
    description: str | None,
    output_dir: Path | None,
    template: str,
    interactive: bool,
) -> None:
    """Generate a feature specification.

    Creates a new spec in the specs/ directory with the given description.
    Optionally runs in interactive mode to ask clarifying questions.

    Examples:

        rstn specify "Add user authentication"

        rstn specify -i "Build payment integration"

        rstn specify --template rust "Add CLI option parser"
    """
    if description is None:
        description = click.prompt("Enter feature description")

    if not description:
        console.print("[red]Error: No description provided[/red]")
        raise SystemExit(1) from None

    # Get state from context
    state: AppState = ctx.obj["state"]
    verbose: bool = ctx.obj.get("verbose", False)

    # Generate workflow ID
    workflow_id = _generate_workflow_id()

    if verbose:
        console.print(f"[dim]Workflow ID: {workflow_id}[/dim]")
        console.print(f"[dim]Template: {template}[/dim]")

    console.print(Panel(
        f"[bold]Generating spec for:[/bold]\n{description}",
        title="Specify Workflow",
        border_style="blue",
    ))

    # Create spec generation request message
    msg = SpecGenerationRequested(description=description)

    # Run through reducer
    new_state, effects = reduce(state, msg)

    # Execute the spec generation workflow
    _execute_specify_workflow(
        description=description,
        output_dir=output_dir or Path("specs"),
        template=template,
        interactive=interactive,
        verbose=verbose,
    )


def _execute_specify_workflow(
    description: str,
    output_dir: Path,
    template: str,
    interactive: bool,
    verbose: bool,
) -> None:
    """Execute the specify workflow.

    This function orchestrates the spec generation process.

    Args:
        description: Feature description
        output_dir: Output directory for spec
        template: Template name
        interactive: Whether to run in interactive mode
        verbose: Whether to show verbose output
    """
    try:
        from rstn.domain.specify import generate_feature_name

        # Step 1: Get next feature number
        with console.status("[bold blue]Allocating feature number...[/bold blue]"):
            feature_number = _get_next_feature_number(output_dir)

        console.print(f"[green]Feature number:[/green] {feature_number}")

        # Step 2: Generate feature name
        with console.status("[bold blue]Generating feature name...[/bold blue]"):
            feature_name = generate_feature_name(description)

        console.print(f"[green]Feature name:[/green] {feature_name}")

        # Step 3: Create spec directory
        spec_dir = output_dir / f"{feature_number}-{feature_name}"
        spec_dir.mkdir(parents=True, exist_ok=True)

        console.print(f"[green]Spec directory:[/green] {spec_dir}")

        # Step 4: Generate spec.md content
        spec_path = spec_dir / "spec.md"
        _write_spec_file(spec_path, feature_number, feature_name, description)

        console.print("\n[bold green]Spec created successfully![/bold green]")
        console.print(f"[dim]Path: {spec_path}[/dim]")

        if interactive:
            console.print("\n[yellow]Interactive mode: Running clarify workflow...[/yellow]")
            # TODO: Integrate with clarify workflow

    except ImportError:
        # Domain module not yet fully implemented, use fallback
        _create_placeholder_spec(output_dir, description)

    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        if verbose:
            import traceback
            console.print(f"[dim]{traceback.format_exc()}[/dim]")
        raise SystemExit(1) from None


def _get_next_feature_number(output_dir: Path) -> str:
    """Get the next available feature number.

    Args:
        output_dir: Directory containing specs

    Returns:
        Next feature number as zero-padded string
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    existing = list(output_dir.glob("[0-9][0-9][0-9]-*"))
    if existing:
        numbers = [int(d.name.split("-")[0]) for d in existing]
        next_num = max(numbers) + 1
    else:
        next_num = 1
    return f"{next_num:03d}"


def _write_spec_file(
    path: Path,
    feature_number: str,
    feature_name: str,
    description: str,
) -> None:
    """Write the spec.md file.

    Args:
        path: Path to spec file
        feature_number: Feature number
        feature_name: Feature name
        description: Feature description
    """
    content = f"""# {feature_number}: {feature_name}

## Overview

{description}

## Requirements

- [ ] TBD

## Technical Design

### Architecture

TBD

### Implementation Notes

TBD

## Testing

- [ ] Unit tests
- [ ] Integration tests

## References

- Related specs: None
"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def _create_placeholder_spec(output_dir: Path, description: str) -> None:
    """Create a placeholder spec when domain module is not available.

    Args:
        output_dir: Output directory
        description: Feature description
    """
    import re

    # Generate simple feature name from description
    words = re.findall(r'\w+', description.lower())[:3]
    feature_name = "-".join(words) if words else "new-feature"

    # Find next number
    output_dir.mkdir(parents=True, exist_ok=True)
    existing = list(output_dir.glob("[0-9][0-9][0-9]-*"))
    if existing:
        numbers = [int(d.name.split("-")[0]) for d in existing]
        next_num = max(numbers) + 1
    else:
        next_num = 1

    feature_number = f"{next_num:03d}"
    spec_dir = output_dir / f"{feature_number}-{feature_name}"
    spec_path = spec_dir / "spec.md"

    _write_spec_file(spec_path, feature_number, feature_name, description)
    console.print(f"[green]Placeholder spec created:[/green] {spec_path}")

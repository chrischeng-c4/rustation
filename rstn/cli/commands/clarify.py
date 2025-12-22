"""Clarify command for running clarification sessions.

Uses reduce() pattern to process clarify workflow.
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt

from rstn.msg import (
    ClarifySessionRequested,
)
from rstn.reduce import reduce
from rstn.state import AppState

console = Console()


@click.command()
@click.argument("spec_path", required=False, type=click.Path(exists=True, path_type=Path))
@click.option(
    "--auto", "-a",
    is_flag=True,
    default=False,
    help="Auto-generate answers using Claude",
)
@click.option(
    "--max-questions", "-n",
    type=int,
    default=10,
    help="Maximum number of questions (default: 10)",
)
@click.pass_context
def clarify(
    ctx: click.Context,
    spec_path: Path | None,
    auto: bool,
    max_questions: int,
) -> None:
    """Run a clarification session on a spec.

    Asks clarifying questions about a spec and integrates answers
    back into the spec document.

    Examples:

        rstn clarify specs/001-user-auth/spec.md

        rstn clarify --auto specs/002-api/spec.md

        rstn clarify -n 5 specs/003-ui/spec.md
    """
    # Find spec if not provided
    if spec_path is None:
        spec_path = _select_spec()

    if spec_path is None:
        console.print("[red]Error: No spec selected[/red]")
        raise SystemExit(1) from None

    # Get state from context
    state: AppState = ctx.obj["state"]
    verbose: bool = ctx.obj.get("verbose", False)

    if verbose:
        console.print(f"[dim]Spec path: {spec_path}[/dim]")
        console.print(f"[dim]Max questions: {max_questions}[/dim]")
        console.print(f"[dim]Auto mode: {auto}[/dim]")

    console.print(Panel(
        f"[bold]Running clarify session on:[/bold]\n{spec_path}",
        title="Clarify Workflow",
        border_style="yellow",
    ))

    # Create clarify session request message
    msg = ClarifySessionRequested(spec_path=str(spec_path))

    # Run through reducer
    new_state, effects = reduce(state, msg)

    # Execute the clarify workflow
    _execute_clarify_workflow(
        spec_path=spec_path,
        auto=auto,
        max_questions=max_questions,
        verbose=verbose,
    )


def _select_spec() -> Path | None:
    """Prompt user to select a spec file.

    Returns:
        Path to selected spec or None
    """
    specs_dir = Path("specs")

    if not specs_dir.exists():
        console.print("[yellow]No specs directory found[/yellow]")
        return None

    specs = sorted(specs_dir.glob("[0-9][0-9][0-9]-*/spec.md"))
    if not specs:
        console.print("[yellow]No specs found[/yellow]")
        return None

    console.print("\n[bold]Available specs:[/bold]")
    for i, spec in enumerate(specs, 1):
        console.print(f"  {i}. {spec.parent.name}")

    console.print()
    selection = Prompt.ask(
        "Select spec",
        choices=[str(i) for i in range(1, len(specs) + 1)],
    )

    return specs[int(selection) - 1]


def _execute_clarify_workflow(
    spec_path: Path,
    auto: bool,
    max_questions: int,
    verbose: bool,
) -> None:
    """Execute the clarify workflow.

    Args:
        spec_path: Path to spec file
        auto: Whether to auto-generate answers
        max_questions: Maximum number of questions
        verbose: Whether to show verbose output
    """
    # Use placeholder clarify workflow
    # Domain module integration will be completed in Phase 7
    _run_placeholder_clarify(spec_path, max_questions, auto, verbose)


def _auto_answer_question(spec_content: str, question: str) -> str:
    """Auto-generate an answer using Claude.

    Args:
        spec_content: The spec content for context
        question: The question to answer

    Returns:
        Generated answer
    """
    import subprocess

    prompt = f"""Based on this spec:

{spec_content[:2000]}

Answer this question concisely:
{question}

Provide a brief, actionable answer."""

    try:
        result = subprocess.run(
            ["claude", "-p", prompt],
            capture_output=True,
            text=True,
            timeout=60,
        )
        return result.stdout.strip() if result.returncode == 0 else "Unable to generate answer"
    except Exception:
        return "Unable to generate answer"


def _run_placeholder_clarify(
    spec_path: Path,
    max_questions: int,
    auto: bool = False,
    verbose: bool = False,
) -> None:
    """Run placeholder clarify when domain module is not available.

    Args:
        spec_path: Path to spec file
        max_questions: Max questions
        auto: Whether to auto-generate answers
        verbose: Whether to show verbose output
    """
    # Generate simple placeholder questions
    placeholder_questions = [
        "What is the primary use case for this feature?",
        "Who are the target users?",
        "What are the acceptance criteria?",
        "Are there any security considerations?",
        "What is the expected timeline?",
    ]

    spec_content = spec_path.read_text()
    questions_to_ask = min(max_questions, len(placeholder_questions))

    console.print(f"\n[bold]Clarify session ({questions_to_ask} questions)[/bold]\n")

    answers: dict[int, str] = {}
    for i, question in enumerate(placeholder_questions[:max_questions], 1):
        console.print(Panel(question, title=f"Question {i}", border_style="yellow"))

        if auto:
            with console.status("[bold blue]Generating answer...[/bold blue]"):
                answer = _auto_answer_question(spec_content, question)
            console.print(f"[dim]Auto-answer: {answer}[/dim]")
        else:
            answer = Prompt.ask("Your answer", default="skip")

        if answer and answer != "skip":
            answers[i] = answer
        console.print()

    if answers:
        # Append answers as a clarifications section
        spec_content = spec_path.read_text()
        clarifications = "\n\n## Clarifications\n\n"
        for q_num, answer in answers.items():
            clarifications += f"**Q{q_num}**: {placeholder_questions[q_num-1]}\n"
            clarifications += f"**A**: {answer}\n\n"

        spec_path.write_text(spec_content + clarifications)
        console.print(f"[green]Added {len(answers)} clarifications to spec[/green]")
    else:
        console.print("[yellow]No answers provided[/yellow]")

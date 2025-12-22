"""Prompt command for running single prompts through Claude.

Uses reduce() pattern to process prompt execution.
"""

from __future__ import annotations

import sys
from pathlib import Path

import click
from rich.console import Console

from rstn.effect import RunClaudeCli
from rstn.msg import WorkflowStartRequested
from rstn.reduce import reduce
from rstn.state import AppState

console = Console()


def _generate_workflow_id() -> str:
    """Generate a unique workflow ID."""
    import uuid
    return f"wf-{uuid.uuid4().hex[:12]}"


@click.command()
@click.argument("message", required=False)
@click.option(
    "--output-format", "-f",
    type=click.Choice(["text", "json", "stream-json"]),
    default="text",
    help="Output format (default: text)",
)
@click.option(
    "--timeout", "-t",
    type=int,
    default=120,
    help="Timeout in seconds (default: 120)",
)
@click.option(
    "--system-prompt", "-s",
    type=click.Path(exists=True, path_type=Path),
    default=None,
    help="Path to system prompt file",
)
@click.option(
    "--cwd", "-C",
    type=click.Path(exists=True, path_type=Path),
    default=None,
    help="Working directory for Claude CLI",
)
@click.option(
    "--stdin",
    is_flag=True,
    default=False,
    help="Read prompt from stdin",
)
@click.pass_context
def prompt(
    ctx: click.Context,
    message: str | None,
    output_format: str,
    timeout: int,
    system_prompt: Path | None,
    cwd: Path | None,
    stdin: bool,
) -> None:
    """Run a single prompt through Claude.

    The prompt message can be provided as an argument, from stdin,
    or will be prompted interactively.

    Examples:

        rstn prompt "Explain this code"

        echo "What is 2+2?" | rstn prompt --stdin

        rstn prompt -f json "List files in JSON"
    """
    # Get prompt from various sources
    if stdin:
        if not sys.stdin.isatty():
            message = sys.stdin.read().strip()
        else:
            console.print("[red]Error: --stdin specified but stdin is a TTY[/red]")
            raise SystemExit(1) from None
    elif message is None:
        message = click.prompt("Enter prompt")

    if not message:
        console.print("[red]Error: No prompt provided[/red]")
        raise SystemExit(1) from None

    # Get state and handler from context
    state: AppState = ctx.obj["state"]
    verbose: bool = ctx.obj.get("verbose", False)

    # Generate workflow ID
    workflow_id = _generate_workflow_id()

    if verbose:
        console.print(f"[dim]Workflow ID: {workflow_id}[/dim]")
        console.print(f"[dim]Output format: {output_format}[/dim]")

    # Create workflow start message
    msg = WorkflowStartRequested(
        workflow_id=workflow_id,
        workflow_type="prompt",
        params=message,
    )

    # Run through reducer to update state
    new_state, effects = reduce(state, msg)

    # Create the Claude CLI effect
    work_dir = cwd or Path.cwd()
    claude_effect = RunClaudeCli(
        prompt=message,
        output_format=output_format,
        timeout_secs=timeout,
        system_prompt_file=system_prompt,
        cwd=work_dir,
        workflow_id=str(workflow_id),
    )

    # Execute the effect
    _execute_claude_prompt(claude_effect, output_format, verbose)


def _execute_claude_prompt(
    effect: RunClaudeCli,
    output_format: str,
    verbose: bool,
) -> None:
    """Execute Claude CLI prompt.

    Args:
        effect: The RunClaudeCli effect to execute
        output_format: Output format for display
        verbose: Whether to show verbose output
    """
    import subprocess

    # Build Claude CLI command
    cmd = ["claude", "-p", effect.prompt]

    if effect.output_format == "stream-json":
        cmd.extend(["--output-format", "stream-json", "--verbose"])
    elif effect.output_format == "json":
        cmd.extend(["--output-format", "json"])

    if effect.system_prompt_file:
        cmd.extend(["--system-prompt-file", str(effect.system_prompt_file)])

    if verbose:
        console.print(f"[dim]Running: {' '.join(cmd)}[/dim]")

    try:
        if output_format == "stream-json":
            # Stream output line by line
            process = subprocess.Popen(
                cmd,
                cwd=effect.cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            if process.stdout:
                for line in process.stdout:
                    # Parse JSONL and extract content
                    _handle_stream_line(line)
            process.wait()
            if process.returncode != 0 and process.stderr:
                console.print(f"[red]Error: {process.stderr.read()}[/red]")
        else:
            # Run and capture output
            result = subprocess.run(
                cmd,
                cwd=effect.cwd,
                capture_output=True,
                text=True,
                timeout=effect.timeout_secs,
            )
            if result.returncode == 0:
                console.print(result.stdout)
            else:
                console.print(f"[red]Error: {result.stderr}[/red]")
                raise SystemExit(result.returncode)

    except subprocess.TimeoutExpired:
        console.print(f"[red]Error: Command timed out after {effect.timeout_secs}s[/red]")
        raise SystemExit(1) from None
    except FileNotFoundError:
        console.print("[red]Error: Claude CLI not found. Install with: npm install -g @anthropic/claude-code[/red]")
        raise SystemExit(1) from None


def _handle_stream_line(line: str) -> None:
    """Handle a single line from stream-json output.

    Args:
        line: JSON line from Claude CLI
    """
    import json

    try:
        data = json.loads(line)
        if data.get("type") == "content_block_delta":
            delta = data.get("delta", {})
            if delta.get("type") == "text_delta":
                text = delta.get("text", "")
                console.print(text, end="")
        elif data.get("type") == "message_stop":
            console.print()  # Final newline
    except json.JSONDecodeError:
        # Non-JSON line, print as-is
        console.print(line.rstrip())

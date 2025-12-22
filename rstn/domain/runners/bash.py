"""Bash script runner.

Pure functions for parsing bash output.
Effect creators for bash execution.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.runners.types import RunnerResult, ScriptConfig
from rstn.effect import AppEffect, RunBashCommand, RunBashScript


def parse_bash_output(
    stdout: str,
    stderr: str,
    exit_code: int,
) -> RunnerResult:
    """Parse bash command output.

    Pure function - no I/O.

    Args:
        stdout: Standard output
        stderr: Standard error
        exit_code: Exit code

    Returns:
        Runner result
    """
    return RunnerResult(
        success=exit_code == 0,
        exit_code=exit_code,
        stdout=stdout,
        stderr=stderr,
    )


def create_bash_command_effects(
    command: str,
    config: ScriptConfig,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run a bash command.

    Effect creator - returns effects, doesn't execute.

    Args:
        command: Command to execute
        config: Script configuration
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    return [
        RunBashCommand(
            command=command,
            cwd=config.cwd,
            effect_id=effect_id,
            timeout_secs=config.timeout_secs,
        )
    ]


def create_bash_script_effects(
    script_path: Path,
    args: list[str] | None = None,
    config: ScriptConfig | None = None,
) -> list[AppEffect]:
    """Create effects to run a bash script.

    Effect creator - returns effects, doesn't execute.

    Args:
        script_path: Path to script file
        args: Script arguments
        config: Optional script configuration

    Returns:
        List of effects to execute
    """
    return [
        RunBashScript(
            script_path=script_path,
            args=args or [],
        )
    ]


def create_inline_script_effects(
    script_content: str,
    config: ScriptConfig,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run inline bash script.

    Effect creator - returns effects, doesn't execute.

    Args:
        script_content: Script content to execute
        config: Script configuration
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    # Use bash -c to execute inline script
    command = f"bash -c '{script_content}'"
    return [
        RunBashCommand(
            command=command,
            cwd=config.cwd,
            effect_id=effect_id,
            timeout_secs=config.timeout_secs,
        )
    ]

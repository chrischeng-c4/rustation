"""Cargo command runner.

Pure functions for parsing cargo output.
Effect creators for cargo execution.
"""

from __future__ import annotations

import json
from typing import Any

from rstn.domain.runners.types import RunnerResult, ScriptConfig
from rstn.effect import AppEffect, RunCargoCommand


def parse_cargo_json_output(
    stdout: str,
    stderr: str,
    exit_code: int,
) -> RunnerResult:
    """Parse cargo JSON output.

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


def extract_cargo_messages(output: str) -> list[dict[str, Any]]:
    """Extract JSON messages from cargo output.

    Pure function - no I/O.

    Args:
        output: Cargo output (may contain JSON lines)

    Returns:
        List of parsed JSON messages
    """
    messages: list[dict[str, Any]] = []

    for line in output.split("\n"):
        line = line.strip()
        if line.startswith("{"):
            try:
                msg = json.loads(line)
                messages.append(msg)
            except json.JSONDecodeError:
                pass

    return messages


def create_cargo_effects(
    subcommand: str,
    args: list[str] | None = None,
    config: ScriptConfig | None = None,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run a cargo command.

    Effect creator - returns effects, doesn't execute.

    Args:
        subcommand: Cargo subcommand (build, test, etc.)
        args: Additional arguments
        config: Script configuration
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    from pathlib import Path

    cwd = config.cwd if config else Path.cwd()

    return [
        RunCargoCommand(
            subcommand=subcommand,
            args=args or [],
            cwd=cwd,
            effect_id=effect_id,
        )
    ]


def create_cargo_build_run_effects(
    config: ScriptConfig,
    package: str | None = None,
    release: bool = False,
) -> list[AppEffect]:
    """Create effects to build and run a cargo project.

    Effect creator - returns effects, doesn't execute.

    Args:
        config: Script configuration
        package: Optional package name
        release: Whether to build in release mode

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Build first
    build_args: list[str] = []
    if package:
        build_args.extend(["--package", package])
    if release:
        build_args.append("--release")

    effects.append(
        RunCargoCommand(
            subcommand="build",
            args=build_args,
            cwd=config.cwd,
            effect_id="cargo_build",
        )
    )

    # Then run
    run_args: list[str] = []
    if package:
        run_args.extend(["--package", package])
    if release:
        run_args.append("--release")

    effects.append(
        RunCargoCommand(
            subcommand="run",
            args=run_args,
            cwd=config.cwd,
            effect_id="cargo_run",
        )
    )

    return effects

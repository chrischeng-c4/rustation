"""Python script runner.

Pure functions for parsing python output.
Effect creators for python execution.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.runners.types import RunnerResult, ScriptConfig
from rstn.effect import AppEffect, RunBashCommand


def parse_python_output(
    stdout: str,
    stderr: str,
    exit_code: int,
) -> RunnerResult:
    """Parse python script output.

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


def create_python_script_effects(
    script_path: Path,
    args: list[str] | None = None,
    config: ScriptConfig | None = None,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run a python script.

    Effect creator - returns effects, doesn't execute.

    Uses 'uv run python' for execution.

    Args:
        script_path: Path to python script
        args: Script arguments
        config: Script configuration
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    cmd_parts = ["uv", "run", "python", str(script_path)]
    if args:
        cmd_parts.extend(args)

    command = " ".join(cmd_parts)
    cwd = config.cwd if config else Path.cwd()
    timeout = config.timeout_secs if config else 120

    return [
        RunBashCommand(
            command=command,
            cwd=cwd,
            effect_id=effect_id,
            timeout_secs=timeout,
        )
    ]


def create_uv_run_effects(
    module_or_script: str,
    args: list[str] | None = None,
    config: ScriptConfig | None = None,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run with uv.

    Effect creator - returns effects, doesn't execute.

    Args:
        module_or_script: Module name or script to run
        args: Additional arguments
        config: Script configuration
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    cmd_parts = ["uv", "run", module_or_script]
    if args:
        cmd_parts.extend(args)

    command = " ".join(cmd_parts)
    cwd = config.cwd if config else Path.cwd()
    timeout = config.timeout_secs if config else 120

    return [
        RunBashCommand(
            command=command,
            cwd=cwd,
            effect_id=effect_id,
            timeout_secs=timeout,
        )
    ]


def create_pytest_run_effects(
    test_path: str | None = None,
    config: ScriptConfig | None = None,
    markers: list[str] | None = None,
    verbose: bool = True,
    effect_id: str = "",
) -> list[AppEffect]:
    """Create effects to run pytest.

    Effect creator - returns effects, doesn't execute.

    Args:
        test_path: Optional test path
        config: Script configuration
        markers: Pytest markers to filter
        verbose: Enable verbose output
        effect_id: Optional effect identifier

    Returns:
        List of effects to execute
    """
    cmd_parts = ["uv", "run", "pytest"]

    if test_path:
        cmd_parts.append(test_path)
    if verbose:
        cmd_parts.append("-v")
    if markers:
        for marker in markers:
            cmd_parts.extend(["-m", marker])

    command = " ".join(cmd_parts)
    cwd = config.cwd if config else Path.cwd()
    timeout = config.timeout_secs if config else 300  # 5 minutes for tests

    return [
        RunBashCommand(
            command=command,
            cwd=cwd,
            effect_id=effect_id,
            timeout_secs=timeout,
        )
    ]

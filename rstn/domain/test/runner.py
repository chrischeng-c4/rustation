"""Test runner operations.

Pure functions for parsing test output.
Effect creators for test commands.
"""

from __future__ import annotations

import re
from pathlib import Path

from rstn.domain.test.types import TestCase, TestResult, TestStatus, TestSuite
from rstn.effect import AppEffect, RunCargoCommand


def parse_cargo_test_output(stdout: str, stderr: str, exit_code: int) -> TestResult:
    """Parse cargo test output.

    Pure function - no I/O.

    Args:
        stdout: Standard output
        stderr: Standard error
        exit_code: Exit code

    Returns:
        Parsed test result
    """
    tests: list[TestCase] = []
    combined = stdout + "\n" + stderr

    # Parse test results
    # Format: test module::test_name ... ok/FAILED/ignored
    for line in combined.split("\n"):
        match = re.match(r"^test\s+(.+?)\s+\.\.\.\s+(\w+)", line)
        if match:
            name = match.group(1)
            result = match.group(2).lower()

            status_map = {
                "ok": TestStatus.PASSED,
                "failed": TestStatus.FAILED,
                "ignored": TestStatus.IGNORED,
            }
            status = status_map.get(result, TestStatus.SKIPPED)

            tests.append(
                TestCase(
                    name=name,
                    status=status,
                )
            )

    # Parse duration if available
    duration_ms = None
    duration_match = re.search(r"finished in (\d+\.?\d*)s", combined)
    if duration_match:
        duration_ms = float(duration_match.group(1)) * 1000

    suite = TestSuite(
        name="cargo test",
        tests=tests,
        duration_ms=duration_ms,
    )

    return TestResult(
        success=exit_code == 0,
        suites=[suite] if tests else [],
        total_duration_ms=duration_ms,
        stdout=stdout,
        stderr=stderr,
    )


def parse_pytest_output(stdout: str, stderr: str, exit_code: int) -> TestResult:
    """Parse pytest output.

    Pure function - no I/O.

    Args:
        stdout: Standard output
        stderr: Standard error
        exit_code: Exit code

    Returns:
        Parsed test result
    """
    tests: list[TestCase] = []
    combined = stdout + "\n" + stderr

    # Parse test results
    # Format: tests/test_foo.py::test_bar PASSED/FAILED/SKIPPED
    for line in combined.split("\n"):
        # Match pytest verbose output
        match = re.match(r"^(.+?::.+?)\s+(PASSED|FAILED|SKIPPED)", line)
        if match:
            name = match.group(1)
            result = match.group(2).lower()

            status_map = {
                "passed": TestStatus.PASSED,
                "failed": TestStatus.FAILED,
                "skipped": TestStatus.SKIPPED,
            }
            status = status_map.get(result, TestStatus.SKIPPED)

            tests.append(
                TestCase(
                    name=name,
                    status=status,
                )
            )

    # Parse summary line
    # Format: ===== 10 passed, 2 failed in 1.23s =====
    duration_ms = None
    summary_match = re.search(r"in (\d+\.?\d*)s", combined)
    if summary_match:
        duration_ms = float(summary_match.group(1)) * 1000

    suite = TestSuite(
        name="pytest",
        tests=tests,
        duration_ms=duration_ms,
    )

    return TestResult(
        success=exit_code == 0,
        suites=[suite] if tests else [],
        total_duration_ms=duration_ms,
        stdout=stdout,
        stderr=stderr,
    )


def create_cargo_test_effects(
    cwd: Path,
    package: str | None = None,
    test_name: str | None = None,
    no_capture: bool = False,
) -> list[AppEffect]:
    """Create effects to run cargo test.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory
        package: Specific package to test
        test_name: Specific test name pattern
        no_capture: Don't capture stdout

    Returns:
        List of effects to execute
    """
    args: list[str] = []
    if package:
        args.extend(["--package", package])
    if test_name:
        args.append(test_name)
    if no_capture:
        args.extend(["--", "--nocapture"])

    return [
        RunCargoCommand(
            subcommand="test",
            args=args,
            cwd=cwd,
            effect_id="cargo_test",
        )
    ]


def create_pytest_effects(
    cwd: Path,
    test_path: str | None = None,
    marker: str | None = None,
    verbose: bool = True,
    fail_fast: bool = False,
) -> list[AppEffect]:
    """Create effects to run pytest.

    Effect creator - returns effects, doesn't execute.

    Note: This uses Bash to run pytest since we don't have a dedicated
    Python runner effect type.

    Args:
        cwd: Working directory
        test_path: Specific test path
        marker: Pytest marker to filter
        verbose: Enable verbose output
        fail_fast: Stop on first failure

    Returns:
        List of effects to execute
    """
    from rstn.effect import RunBashCommand

    args = ["uv", "run", "pytest"]
    if test_path:
        args.append(test_path)
    if marker:
        args.extend(["-m", marker])
    if verbose:
        args.append("-v")
    if fail_fast:
        args.append("-x")

    return [
        RunBashCommand(
            command=" ".join(args),
            cwd=cwd,
            effect_id="pytest",
        )
    ]

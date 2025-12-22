"""Git commit workflow operations.

Pure functions for analyzing commit groups and building commit messages.
Effect creators for commit workflow orchestration.
"""

from __future__ import annotations

import re
from pathlib import Path

from rstn.domain.git.types import CommitGroup, CommitResult
from rstn.effect import AppEffect, RunGitCommand


def analyze_commit_groups(
    files: list[str],
    diff_stats: str,
) -> list[CommitGroup]:
    """Analyze changed files and group for commits.

    Pure function - no I/O.

    Groups files by:
    1. Type (source, test, doc, config)
    2. Directory proximity
    3. Related changes

    Args:
        files: List of changed file paths
        diff_stats: Output of git diff --stat

    Returns:
        List of commit groups
    """
    if not files:
        return []

    # Categorize files
    source_files: list[str] = []
    test_files: list[str] = []
    doc_files: list[str] = []
    config_files: list[str] = []

    for f in files:
        if _is_test_file(f):
            test_files.append(f)
        elif _is_doc_file(f):
            doc_files.append(f)
        elif _is_config_file(f):
            config_files.append(f)
        else:
            source_files.append(f)

    groups: list[CommitGroup] = []

    # Group source files by directory
    if source_files:
        dir_groups = _group_by_directory(source_files)
        for directory, dir_files in dir_groups.items():
            scope = _extract_scope(directory)
            groups.append(
                CommitGroup(
                    files=dir_files,
                    message=f"Update {scope}",
                    category="feat",
                    scope=scope,
                )
            )

    # Group test files
    if test_files:
        groups.append(
            CommitGroup(
                files=test_files,
                message="Add/update tests",
                category="test",
                scope=None,
            )
        )

    # Group doc files
    if doc_files:
        groups.append(
            CommitGroup(
                files=doc_files,
                message="Update documentation",
                category="docs",
                scope=None,
            )
        )

    # Group config files
    if config_files:
        groups.append(
            CommitGroup(
                files=config_files,
                message="Update configuration",
                category="chore",
                scope=None,
            )
        )

    return groups


def _is_test_file(path: str) -> bool:
    """Check if path is a test file."""
    return (
        "/test" in path
        or "/tests" in path
        or path.endswith("_test.py")
        or path.endswith("_test.rs")
        or path.startswith("test_")
    )


def _is_doc_file(path: str) -> bool:
    """Check if path is a documentation file."""
    return (
        path.endswith(".md")
        or path.endswith(".rst")
        or path.endswith(".txt")
        or "/docs/" in path
        or "/doc/" in path
    )


def _is_config_file(path: str) -> bool:
    """Check if path is a configuration file."""
    config_names = {
        "Cargo.toml",
        "pyproject.toml",
        "package.json",
        ".gitignore",
        ".env.example",
        "Makefile",
        "justfile",
    }
    return (
        Path(path).name in config_names
        or path.endswith(".toml")
        or path.endswith(".yaml")
        or path.endswith(".yml")
        or path.endswith(".json")
    )


def _group_by_directory(files: list[str]) -> dict[str, list[str]]:
    """Group files by their parent directory."""
    groups: dict[str, list[str]] = {}
    for f in files:
        parent = str(Path(f).parent)
        if parent not in groups:
            groups[parent] = []
        groups[parent].append(f)
    return groups


def _extract_scope(directory: str) -> str:
    """Extract a scope name from directory path."""
    parts = directory.split("/")
    # Find meaningful part (skip src, lib, etc.)
    skip = {"src", "lib", ".", ""}
    for part in reversed(parts):
        if part not in skip:
            return part
    return "core"


def build_commit_message(
    category: str,
    scope: str | None,
    description: str,
    body: str | None = None,
    breaking: bool = False,
) -> str:
    """Build conventional commit message.

    Pure function - no I/O.

    Format: category(scope)!: description

    Args:
        category: Commit category (feat, fix, docs, etc.)
        scope: Optional scope
        description: Short description
        body: Optional longer description
        breaking: Whether this is a breaking change

    Returns:
        Formatted commit message
    """
    # Build header
    header = category
    if scope:
        header += f"({scope})"
    if breaking:
        header += "!"
    header += f": {description}"

    # Add body if provided
    if body:
        return f"{header}\n\n{body}"

    return header


def parse_commit_message(message: str) -> dict[str, str | None]:
    """Parse conventional commit message.

    Pure function - no I/O.

    Args:
        message: Commit message to parse

    Returns:
        Dict with category, scope, breaking, description, body
    """
    lines = message.strip().split("\n")
    header = lines[0]

    # Parse header: category(scope)!: description
    match = re.match(r"^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$", header)
    if not match:
        return {
            "category": None,
            "scope": None,
            "breaking": None,
            "description": header,
            "body": None,
        }

    body = None
    if len(lines) > 2:
        body = "\n".join(lines[2:]).strip()

    return {
        "category": match.group(1),
        "scope": match.group(2),
        "breaking": "true" if match.group(3) else None,
        "description": match.group(4),
        "body": body,
    }


def create_commit_effects(
    cwd: Path,
    message: str,
    files: list[str] | None = None,
) -> list[AppEffect]:
    """Create effects to make a commit.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory
        message: Commit message
        files: Specific files to commit (None = all staged)

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Stage specific files if provided
    if files:
        effects.append(
            RunGitCommand(
                args=["add", "--"] + files,
                cwd=cwd,
                effect_id="commit_stage",
            )
        )

    # Create commit
    effects.append(
        RunGitCommand(
            args=["commit", "-m", message],
            cwd=cwd,
            effect_id="commit_create",
        )
    )

    return effects


def create_staged_files_effects(cwd: Path) -> list[AppEffect]:
    """Create effects to get list of staged files.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    return [
        RunGitCommand(
            args=["diff", "--cached", "--name-only"],
            cwd=cwd,
            effect_id="staged_files",
        )
    ]


def create_diff_stat_effects(cwd: Path) -> list[AppEffect]:
    """Create effects to get diff statistics.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    return [
        RunGitCommand(
            args=["diff", "--cached", "--stat"],
            cwd=cwd,
            effect_id="diff_stat",
        )
    ]


def build_commit_result(
    blocked: bool,
    security_result: object | None = None,
    groups: list[CommitGroup] | None = None,
    suggested_message: str | None = None,
) -> CommitResult:
    """Build commit result from analysis.

    Pure function - no I/O.

    Args:
        blocked: Whether commit is blocked
        security_result: Optional security scan result
        groups: Optional commit groups
        suggested_message: Optional suggested message

    Returns:
        Complete commit result
    """
    from rstn.domain.git.types import SecurityScanResult

    sec_result = None
    if isinstance(security_result, SecurityScanResult):
        sec_result = security_result

    return CommitResult(
        blocked=blocked,
        security_result=sec_result,
        groups=groups or [],
        suggested_message=suggested_message,
    )

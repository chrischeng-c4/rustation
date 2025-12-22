"""Git worktree management.

Pure functions for parsing worktree output and extracting feature info.
Effect creators for worktree operations.
"""

from __future__ import annotations

import re
from pathlib import Path

from rstn.domain.git.types import FeatureInfo, WorktreeInfo
from rstn.effect import AppEffect, RunGitCommand


def parse_worktrees(output: str) -> list[WorktreeInfo]:
    """Parse git worktree list --porcelain output.

    Pure function - no I/O.

    Args:
        output: Output from `git worktree list --porcelain`

    Returns:
        List of worktree info
    """
    worktrees: list[WorktreeInfo] = []
    current: dict[str, str | bool] = {}

    for line in output.strip().split("\n"):
        if not line:
            # Empty line separates worktrees
            if current and "path" in current:
                worktrees.append(_build_worktree_info(current))
            current = {}
            continue

        if line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("HEAD "):
            current["head"] = line[5:]
        elif line.startswith("branch "):
            # Format: refs/heads/branch-name
            branch = line[7:]
            if branch.startswith("refs/heads/"):
                branch = branch[11:]
            current["branch"] = branch
        elif line == "bare":
            current["is_bare"] = True
        elif line == "detached":
            current["is_detached"] = True
        elif line == "locked":
            current["is_locked"] = True
        elif line == "prunable":
            current["prunable"] = True

    # Handle last worktree
    if current and "path" in current:
        worktrees.append(_build_worktree_info(current))

    return worktrees


def _build_worktree_info(data: dict[str, str | bool]) -> WorktreeInfo:
    """Build WorktreeInfo from parsed data."""
    path_str = data.get("path", "")
    if not isinstance(path_str, str):
        path_str = ""

    head_str = data.get("head", "")
    if not isinstance(head_str, str):
        head_str = ""

    branch_val = data.get("branch")
    branch_str = branch_val if isinstance(branch_val, str) else None

    return WorktreeInfo(
        path=Path(path_str),
        head=head_str,
        branch=branch_str,
        is_bare=bool(data.get("is_bare", False)),
        is_detached=bool(data.get("is_detached", False)),
        is_locked=bool(data.get("is_locked", False)),
        prunable=bool(data.get("prunable", False)),
    )


def extract_feature_info(branch_name: str) -> FeatureInfo | None:
    """Extract feature info from branch name.

    Pure function - no I/O.

    Expected format: NNN-feature-name (e.g., 042-worktree-management)

    Args:
        branch_name: Git branch name

    Returns:
        FeatureInfo if valid format, None otherwise
    """
    # Match NNN-name pattern
    match = re.match(r"^(\d{3})-(.+)$", branch_name)
    if not match:
        return None

    number = match.group(1)
    name = match.group(2)

    return FeatureInfo(
        number=number,
        name=name,
        full_name=branch_name,
    )


def find_worktree_by_feature(
    worktrees: list[WorktreeInfo],
    feature_number: str,
) -> WorktreeInfo | None:
    """Find worktree by feature number.

    Pure function - no I/O.

    Args:
        worktrees: List of worktrees
        feature_number: Feature number to find (e.g., "042")

    Returns:
        Matching worktree or None
    """
    for wt in worktrees:
        if wt.branch:
            info = extract_feature_info(wt.branch)
            if info and info.number == feature_number:
                return wt
    return None


def create_worktree_list_effects(cwd: Path) -> list[AppEffect]:
    """Create effects to list worktrees.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    return [
        RunGitCommand(
            args=["worktree", "list", "--porcelain"],
            cwd=cwd,
            effect_id="worktree_list",
        )
    ]


def create_worktree_add_effects(
    cwd: Path,
    path: Path,
    branch: str,
    create_branch: bool = True,
) -> list[AppEffect]:
    """Create effects to add a new worktree.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory
        path: Path for new worktree
        branch: Branch name
        create_branch: Whether to create the branch

    Returns:
        List of effects to execute
    """
    args = ["worktree", "add"]
    if create_branch:
        args.extend(["-b", branch])
    else:
        args.append(branch)
    args.append(str(path))

    return [
        RunGitCommand(
            args=args,
            cwd=cwd,
            effect_id="worktree_add",
        )
    ]


def create_worktree_remove_effects(
    cwd: Path,
    path: Path,
    force: bool = False,
) -> list[AppEffect]:
    """Create effects to remove a worktree.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory
        path: Worktree path to remove
        force: Force removal even if dirty

    Returns:
        List of effects to execute
    """
    args = ["worktree", "remove"]
    if force:
        args.append("--force")
    args.append(str(path))

    return [
        RunGitCommand(
            args=args,
            cwd=cwd,
            effect_id="worktree_remove",
        )
    ]

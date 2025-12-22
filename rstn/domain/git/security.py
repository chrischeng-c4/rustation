"""Security scanning for git staged changes.

Pure functions for analyzing diffs and filenames for secrets.
Effect creators for orchestrating security scans.
"""

from __future__ import annotations

import re
from pathlib import Path

from rstn.domain.git.patterns import SECRET_PATTERNS, SENSITIVE_FILES
from rstn.domain.git.types import (
    SecurityScanResult,
    SecurityWarning,
    SensitiveFile,
    Severity,
)
from rstn.effect import AppEffect, RunGitCommand


def analyze_diff_for_secrets(diff: str) -> list[SecurityWarning]:
    """Analyze diff content for potential secrets.

    Pure function - no I/O.

    Args:
        diff: Git diff output

    Returns:
        List of security warnings found
    """
    warnings: list[SecurityWarning] = []
    current_file = ""
    current_line = 0

    for line in diff.split("\n"):
        # Track current file from diff headers
        if line.startswith("+++ b/"):
            current_file = line[6:]
            current_line = 0
            continue

        # Track line numbers from @@ headers
        if line.startswith("@@"):
            # Parse @@ -old,count +new,count @@
            match = re.search(r"\+(\d+)", line)
            if match:
                current_line = int(match.group(1)) - 1
            continue

        # Only check added lines (starting with +)
        if not line.startswith("+") or line.startswith("+++"):
            if not line.startswith("-"):
                current_line += 1
            continue

        current_line += 1
        content = line[1:]  # Remove the + prefix

        # Check against all secret patterns
        for pattern, description, severity in SECRET_PATTERNS:
            if re.search(pattern, content):
                warnings.append(
                    SecurityWarning(
                        file_path=current_file,
                        line_number=current_line,
                        pattern_matched=description,
                        severity=severity,
                        message=f"{description} in {current_file}:{current_line}",
                    )
                )

    return warnings


def analyze_sensitive_filenames(files: list[str]) -> list[SensitiveFile]:
    """Check filenames against sensitive file patterns.

    Pure function - no I/O.

    Args:
        files: List of file paths to check

    Returns:
        List of sensitive files found
    """
    sensitive: list[SensitiveFile] = []

    for file_path in files:
        filename = Path(file_path).name

        for pattern, reason in SENSITIVE_FILES:
            # Handle glob patterns
            if pattern.startswith("*."):
                if filename.endswith(pattern[1:]):
                    sensitive.append(
                        SensitiveFile(
                            path=file_path,
                            reason=reason,
                            suggest_gitignore=True,
                        )
                    )
                    break
            # Handle prefix patterns
            elif pattern.endswith("."):
                if filename.startswith(pattern[:-1]):
                    sensitive.append(
                        SensitiveFile(
                            path=file_path,
                            reason=reason,
                            suggest_gitignore=True,
                        )
                    )
                    break
            # Exact match
            elif filename == pattern:
                sensitive.append(
                    SensitiveFile(
                        path=file_path,
                        reason=reason,
                        suggest_gitignore=True,
                    )
                )
                break

    return sensitive


def build_security_scan_result(
    diff_warnings: list[SecurityWarning],
    sensitive_files: list[SensitiveFile],
) -> SecurityScanResult:
    """Build security scan result from analysis.

    Pure function - no I/O.

    Args:
        diff_warnings: Warnings from diff analysis
        sensitive_files: Sensitive files found

    Returns:
        Complete security scan result
    """
    # Block if any critical warnings
    blocked = any(w.severity == Severity.CRITICAL for w in diff_warnings)

    return SecurityScanResult(
        blocked=blocked,
        warnings=diff_warnings,
        sensitive_files=sensitive_files,
    )


def create_security_scan_effects(cwd: Path, scan_all: bool = False) -> list[AppEffect]:
    """Create effects to run security scan.

    Effect creator - returns effects, doesn't execute.

    Args:
        cwd: Working directory
        scan_all: If True, scan all changes; if False, only staged

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Get list of staged files
    if scan_all:
        effects.append(
            RunGitCommand(
                args=["diff", "--name-only"],
                cwd=cwd,
                effect_id="security_scan_files",
            )
        )
        effects.append(
            RunGitCommand(
                args=["diff"],
                cwd=cwd,
                effect_id="security_scan_diff",
            )
        )
    else:
        effects.append(
            RunGitCommand(
                args=["diff", "--cached", "--name-only"],
                cwd=cwd,
                effect_id="security_scan_files",
            )
        )
        effects.append(
            RunGitCommand(
                args=["diff", "--cached"],
                cwd=cwd,
                effect_id="security_scan_diff",
            )
        )

    return effects

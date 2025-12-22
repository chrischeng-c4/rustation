"""Git domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field


class Severity(str, Enum):
    """Severity level of a security issue."""

    CRITICAL = "critical"  # Blocks commit (private keys, etc.)
    HIGH = "high"  # Warns but allows (API keys, passwords)
    MEDIUM = "medium"  # Info only (base64, long hex)


class SecurityWarning(BaseModel):
    """A security warning found during scanning."""

    model_config = {"frozen": True}

    file_path: str = Field(description="File containing the issue")
    line_number: int = Field(ge=0, description="Line number (0 if unknown)")
    pattern_matched: str = Field(description="Pattern that matched")
    severity: Severity = Field(description="Severity level")
    message: str = Field(description="Human-readable warning message")


class SensitiveFile(BaseModel):
    """A sensitive file found in staged changes."""

    model_config = {"frozen": True}

    path: str = Field(description="File path")
    reason: str = Field(description="Why this file is sensitive")
    suggest_gitignore: bool = Field(description="Whether to suggest adding to .gitignore")


class SecurityScanResult(BaseModel):
    """Security scan result for staged changes."""

    model_config = {"frozen": True}

    blocked: bool = Field(description="Whether commit should be blocked")
    warnings: list[SecurityWarning] = Field(default_factory=list, description="Warnings found")
    sensitive_files: list[SensitiveFile] = Field(
        default_factory=list, description="Sensitive files found"
    )

    @property
    def critical_count(self) -> int:
        """Count of critical severity warnings."""
        return sum(1 for w in self.warnings if w.severity == Severity.CRITICAL)

    @property
    def high_count(self) -> int:
        """Count of high severity warnings."""
        return sum(1 for w in self.warnings if w.severity == Severity.HIGH)

    @property
    def medium_count(self) -> int:
        """Count of medium severity warnings."""
        return sum(1 for w in self.warnings if w.severity == Severity.MEDIUM)


class WorktreeInfo(BaseModel):
    """Information about a git worktree."""

    model_config = {"frozen": True}

    path: Path = Field(description="Worktree path")
    head: str = Field(description="HEAD commit hash")
    branch: str | None = Field(default=None, description="Branch name if any")
    is_bare: bool = Field(default=False, description="Whether this is bare worktree")
    is_detached: bool = Field(default=False, description="Whether HEAD is detached")
    is_locked: bool = Field(default=False, description="Whether worktree is locked")
    prunable: bool = Field(default=False, description="Whether worktree is prunable")


class FeatureInfo(BaseModel):
    """Feature information extracted from branch name."""

    model_config = {"frozen": True}

    number: str = Field(description="Feature number (e.g., '042')")
    name: str = Field(description="Feature name (e.g., 'worktree-management')")
    full_name: str = Field(description="Full feature name (e.g., '042-worktree-management')")


class CommitGroup(BaseModel):
    """A group of related files to commit together."""

    model_config = {"frozen": True}

    files: list[str] = Field(description="Files in this commit group")
    message: str = Field(description="Commit message")
    category: str = Field(description="Category (feat, fix, refactor, etc.)")
    scope: str | None = Field(default=None, description="Optional scope")


class CommitResult(BaseModel):
    """Result of commit preparation."""

    model_config = {"frozen": True}

    blocked: bool = Field(description="Whether commit was blocked by security scan")
    security_result: SecurityScanResult | None = Field(
        default=None, description="Security scan result"
    )
    groups: list[CommitGroup] = Field(default_factory=list, description="Commit groups")
    suggested_message: str | None = Field(default=None, description="Suggested commit message")

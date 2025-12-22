"""Git domain operations for rstn.

Provides git operations including:
- Worktree management
- Security scanning for secrets
- Commit workflow orchestration

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.git.commit import (
    analyze_commit_groups,
    build_commit_message,
    build_commit_result,
    create_commit_effects,
    create_diff_stat_effects,
    create_staged_files_effects,
    parse_commit_message,
)
from rstn.domain.git.security import (
    analyze_diff_for_secrets,
    analyze_sensitive_filenames,
    build_security_scan_result,
    create_security_scan_effects,
)
from rstn.domain.git.types import (
    CommitGroup,
    CommitResult,
    FeatureInfo,
    SecurityScanResult,
    SecurityWarning,
    SensitiveFile,
    Severity,
    WorktreeInfo,
)
from rstn.domain.git.worktree import (
    create_worktree_add_effects,
    create_worktree_list_effects,
    create_worktree_remove_effects,
    extract_feature_info,
    find_worktree_by_feature,
    parse_worktrees,
)

__all__ = [
    # Types
    "CommitGroup",
    "CommitResult",
    "FeatureInfo",
    "SecurityScanResult",
    "SecurityWarning",
    "SensitiveFile",
    "Severity",
    "WorktreeInfo",
    # Security functions
    "analyze_diff_for_secrets",
    "analyze_sensitive_filenames",
    "build_security_scan_result",
    "create_security_scan_effects",
    # Worktree functions
    "parse_worktrees",
    "extract_feature_info",
    "find_worktree_by_feature",
    "create_worktree_list_effects",
    "create_worktree_add_effects",
    "create_worktree_remove_effects",
    # Commit functions
    "analyze_commit_groups",
    "build_commit_message",
    "parse_commit_message",
    "create_commit_effects",
    "create_staged_files_effects",
    "create_diff_stat_effects",
    "build_commit_result",
]

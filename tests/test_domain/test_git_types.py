"""Tests for git domain types."""

from __future__ import annotations

from pathlib import Path

import pytest
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


class TestSeverity:
    """Tests for Severity enum."""

    def test_severity_values(self) -> None:
        """Test all severity values."""
        assert Severity.CRITICAL.value == "critical"
        assert Severity.HIGH.value == "high"
        assert Severity.MEDIUM.value == "medium"

    def test_severity_is_string_enum(self) -> None:
        """Test Severity is a string enum."""
        for severity in Severity:
            assert isinstance(severity.value, str)


class TestSecurityWarning:
    """Tests for SecurityWarning model."""

    def test_warning_creation(self) -> None:
        """Test creating security warning."""
        warning = SecurityWarning(
            file_path="config.py",
            line_number=42,
            pattern_matched="AWS_SECRET_KEY",
            severity=Severity.CRITICAL,
            message="Potential AWS secret key detected",
        )
        assert warning.file_path == "config.py"
        assert warning.line_number == 42
        assert warning.pattern_matched == "AWS_SECRET_KEY"
        assert warning.severity == Severity.CRITICAL
        assert warning.message == "Potential AWS secret key detected"

    def test_warning_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        warning = SecurityWarning(
            file_path="secret.txt",
            line_number=1,
            pattern_matched="password",
            severity=Severity.HIGH,
            message="Password found",
        )
        json_str = warning.model_dump_json()
        restored = SecurityWarning.model_validate_json(json_str)
        assert restored == warning

    def test_warning_immutable(self) -> None:
        """Test warning is immutable (frozen)."""
        warning = SecurityWarning(
            file_path="test.py",
            line_number=0,
            pattern_matched="test",
            severity=Severity.MEDIUM,
            message="Test",
        )
        with pytest.raises(Exception):
            warning.file_path = "other.py"  # type: ignore

    def test_warning_line_number_zero(self) -> None:
        """Test line number can be zero (unknown)."""
        warning = SecurityWarning(
            file_path="test.py",
            line_number=0,
            pattern_matched="test",
            severity=Severity.MEDIUM,
            message="Line unknown",
        )
        assert warning.line_number == 0


class TestSensitiveFile:
    """Tests for SensitiveFile model."""

    def test_sensitive_file_creation(self) -> None:
        """Test creating sensitive file."""
        sf = SensitiveFile(
            path=".env",
            reason="Environment variables may contain secrets",
            suggest_gitignore=True,
        )
        assert sf.path == ".env"
        assert sf.reason == "Environment variables may contain secrets"
        assert sf.suggest_gitignore is True

    def test_sensitive_file_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        sf = SensitiveFile(
            path="credentials.json",
            reason="Credential file",
            suggest_gitignore=True,
        )
        json_str = sf.model_dump_json()
        restored = SensitiveFile.model_validate_json(json_str)
        assert restored == sf


class TestSecurityScanResult:
    """Tests for SecurityScanResult model."""

    def test_clean_scan(self) -> None:
        """Test clean security scan."""
        result = SecurityScanResult(blocked=False)
        assert result.blocked is False
        assert result.warnings == []
        assert result.sensitive_files == []
        assert result.critical_count == 0
        assert result.high_count == 0
        assert result.medium_count == 0

    def test_scan_with_warnings(self) -> None:
        """Test scan with warnings."""
        warnings = [
            SecurityWarning(
                file_path="a.py",
                line_number=1,
                pattern_matched="secret",
                severity=Severity.CRITICAL,
                message="Critical",
            ),
            SecurityWarning(
                file_path="b.py",
                line_number=2,
                pattern_matched="password",
                severity=Severity.HIGH,
                message="High",
            ),
            SecurityWarning(
                file_path="c.py",
                line_number=3,
                pattern_matched="base64",
                severity=Severity.MEDIUM,
                message="Medium",
            ),
            SecurityWarning(
                file_path="d.py",
                line_number=4,
                pattern_matched="key",
                severity=Severity.HIGH,
                message="High 2",
            ),
        ]
        result = SecurityScanResult(blocked=True, warnings=warnings)
        assert result.blocked is True
        assert result.critical_count == 1
        assert result.high_count == 2
        assert result.medium_count == 1

    def test_scan_with_sensitive_files(self) -> None:
        """Test scan with sensitive files."""
        files = [
            SensitiveFile(path=".env", reason="Env file", suggest_gitignore=True),
            SensitiveFile(path="id_rsa", reason="SSH key", suggest_gitignore=True),
        ]
        result = SecurityScanResult(blocked=True, sensitive_files=files)
        assert len(result.sensitive_files) == 2

    def test_scan_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        warnings = [
            SecurityWarning(
                file_path="test.py",
                line_number=1,
                pattern_matched="test",
                severity=Severity.HIGH,
                message="Test",
            ),
        ]
        result = SecurityScanResult(blocked=True, warnings=warnings)
        json_str = result.model_dump_json()
        restored = SecurityScanResult.model_validate_json(json_str)
        assert restored.blocked == result.blocked
        assert restored.critical_count == result.critical_count


class TestWorktreeInfo:
    """Tests for WorktreeInfo model."""

    def test_worktree_info_creation(self) -> None:
        """Test creating worktree info."""
        info = WorktreeInfo(
            path=Path("/project/.git/worktrees/feature-001"),
            head="abc123def456",
            branch="feature/001-user-auth",
        )
        assert info.path == Path("/project/.git/worktrees/feature-001")
        assert info.head == "abc123def456"
        assert info.branch == "feature/001-user-auth"
        assert info.is_bare is False
        assert info.is_detached is False
        assert info.is_locked is False
        assert info.prunable is False

    def test_worktree_info_detached(self) -> None:
        """Test detached HEAD worktree."""
        info = WorktreeInfo(
            path=Path("/project/worktree"),
            head="abc123",
            is_detached=True,
        )
        assert info.branch is None
        assert info.is_detached is True

    def test_worktree_info_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        info = WorktreeInfo(
            path=Path("/project"),
            head="abc123",
            branch="main",
            is_locked=True,
        )
        data = info.model_dump(mode="json")
        restored = WorktreeInfo.model_validate(data)
        assert restored.is_locked == info.is_locked


class TestFeatureInfo:
    """Tests for FeatureInfo model."""

    def test_feature_info_creation(self) -> None:
        """Test creating feature info."""
        info = FeatureInfo(
            number="042",
            name="worktree-management",
            full_name="042-worktree-management",
        )
        assert info.number == "042"
        assert info.name == "worktree-management"
        assert info.full_name == "042-worktree-management"

    def test_feature_info_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        info = FeatureInfo(
            number="001",
            name="test",
            full_name="001-test",
        )
        json_str = info.model_dump_json()
        restored = FeatureInfo.model_validate_json(json_str)
        assert restored == info


class TestCommitGroup:
    """Tests for CommitGroup model."""

    def test_commit_group_creation(self) -> None:
        """Test creating commit group."""
        group = CommitGroup(
            files=["src/main.rs", "src/lib.rs"],
            message="feat(core): add main functionality",
            category="feat",
            scope="core",
        )
        assert group.files == ["src/main.rs", "src/lib.rs"]
        assert group.message == "feat(core): add main functionality"
        assert group.category == "feat"
        assert group.scope == "core"

    def test_commit_group_no_scope(self) -> None:
        """Test commit group without scope."""
        group = CommitGroup(
            files=["README.md"],
            message="docs: update readme",
            category="docs",
        )
        assert group.scope is None

    def test_commit_group_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        group = CommitGroup(
            files=["test.py"],
            message="test: add tests",
            category="test",
        )
        json_str = group.model_dump_json()
        restored = CommitGroup.model_validate_json(json_str)
        assert restored == group


class TestCommitResult:
    """Tests for CommitResult model."""

    def test_commit_result_blocked(self) -> None:
        """Test blocked commit result."""
        scan = SecurityScanResult(blocked=True)
        result = CommitResult(
            blocked=True,
            security_result=scan,
        )
        assert result.blocked is True
        assert result.security_result is not None
        assert result.groups == []
        assert result.suggested_message is None

    def test_commit_result_with_groups(self) -> None:
        """Test commit result with groups."""
        groups = [
            CommitGroup(
                files=["src/main.rs"],
                message="feat: add main",
                category="feat",
            ),
            CommitGroup(
                files=["tests/test.rs"],
                message="test: add tests",
                category="test",
            ),
        ]
        result = CommitResult(
            blocked=False,
            groups=groups,
            suggested_message="feat: add main with tests",
        )
        assert result.blocked is False
        assert len(result.groups) == 2
        assert result.suggested_message == "feat: add main with tests"

    def test_commit_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        result = CommitResult(
            blocked=False,
            suggested_message="Test commit",
        )
        json_str = result.model_dump_json()
        restored = CommitResult.model_validate_json(json_str)
        assert restored.blocked == result.blocked
        assert restored.suggested_message == result.suggested_message

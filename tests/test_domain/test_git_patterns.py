"""Tests for git security patterns."""

from __future__ import annotations

import re

import pytest
from rstn.domain.git.patterns import (
    SECRET_PATTERNS,
    SENSITIVE_FILES,
)
from rstn.domain.git.types import Severity


class TestSecretPatterns:
    """Tests for secret detection patterns."""

    def test_patterns_are_valid_regex(self) -> None:
        """Test all patterns compile as valid regex."""
        for pattern, _description, _severity in SECRET_PATTERNS:
            try:
                re.compile(pattern)
            except re.error as e:
                pytest.fail(f"Invalid regex pattern '{pattern}': {e}")

    def test_patterns_have_valid_severity(self) -> None:
        """Test all patterns have valid severity."""
        for _pattern, _description, severity in SECRET_PATTERNS:
            assert severity in Severity
            assert isinstance(severity, Severity)

    def test_patterns_have_descriptions(self) -> None:
        """Test all patterns have non-empty descriptions."""
        for _pattern, description, _severity in SECRET_PATTERNS:
            assert description
            assert isinstance(description, str)

    def test_critical_patterns_exist(self) -> None:
        """Test critical severity patterns exist."""
        critical = [p for p in SECRET_PATTERNS if p[2] == Severity.CRITICAL]
        assert len(critical) > 0

    def test_high_patterns_exist(self) -> None:
        """Test high severity patterns exist."""
        high = [p for p in SECRET_PATTERNS if p[2] == Severity.HIGH]
        assert len(high) > 0

    def test_private_key_detection(self) -> None:
        """Test private key patterns match correctly."""
        private_key_patterns = [
            p[0] for p in SECRET_PATTERNS if "PRIVATE KEY" in p[0]
        ]
        assert len(private_key_patterns) > 0

        # Test RSA private key header
        test_content = "-----BEGIN RSA PRIVATE KEY-----"
        for pattern in private_key_patterns:
            if re.search(pattern, test_content):
                break
        else:
            pytest.fail("No pattern matched RSA private key")

    def test_api_key_detection(self) -> None:
        """Test API key patterns match correctly."""
        api_key_patterns = [
            (p[0], p[1]) for p in SECRET_PATTERNS if "API" in p[1].upper() or "api" in p[0]
        ]
        assert len(api_key_patterns) > 0

    def test_github_token_detection(self) -> None:
        """Test GitHub token pattern."""
        gh_patterns = [p[0] for p in SECRET_PATTERNS if "GitHub" in p[1]]
        assert len(gh_patterns) > 0

        # Test GitHub token format
        test_token = "ghp_1234567890abcdefghijklmnopqrstuvwxyz"
        for pattern in gh_patterns:
            if re.search(pattern, test_token):
                break
        else:
            pytest.fail("No pattern matched GitHub token")

    def test_openai_key_detection(self) -> None:
        """Test OpenAI/Anthropic API key pattern."""
        sk_patterns = [p[0] for p in SECRET_PATTERNS if "OpenAI" in p[1] or "sk-" in p[0]]
        assert len(sk_patterns) > 0

        # Test sk- format
        test_key = "sk-abc123def456ghi789jkl012mno345"
        for pattern in sk_patterns:
            if re.search(pattern, test_key):
                break
        else:
            pytest.fail("No pattern matched OpenAI key")


class TestSensitiveFiles:
    """Tests for sensitive file patterns."""

    def test_sensitive_files_not_empty(self) -> None:
        """Test sensitive files list is not empty."""
        assert len(SENSITIVE_FILES) > 0

    def test_sensitive_files_have_reasons(self) -> None:
        """Test all sensitive files have reasons."""
        for pattern, reason in SENSITIVE_FILES:
            assert pattern
            assert reason
            assert isinstance(pattern, str)
            assert isinstance(reason, str)

    def test_env_files_included(self) -> None:
        """Test .env files are in sensitive list."""
        env_patterns = [p[0] for p in SENSITIVE_FILES if ".env" in p[0]]
        assert len(env_patterns) > 0

    def test_key_files_included(self) -> None:
        """Test key files are in sensitive list."""
        key_patterns = [
            p[0] for p in SENSITIVE_FILES
            if ".key" in p[0] or ".pem" in p[0] or "id_rsa" in p[0]
        ]
        assert len(key_patterns) > 0

    def test_credentials_included(self) -> None:
        """Test credentials files are in sensitive list."""
        cred_patterns = [
            p[0] for p in SENSITIVE_FILES
            if "credential" in p[0].lower() or "secret" in p[0].lower()
        ]
        assert len(cred_patterns) > 0

    def test_ssh_keys_included(self) -> None:
        """Test SSH key files are in sensitive list."""
        ssh_patterns = [
            p[0] for p in SENSITIVE_FILES
            if "id_rsa" in p[0] or "id_ed25519" in p[0] or "id_ecdsa" in p[0]
        ]
        assert len(ssh_patterns) >= 3  # At least RSA, ED25519, ECDSA

    def test_npm_pypi_configs_included(self) -> None:
        """Test npm and pypi config files are in sensitive list."""
        config_patterns = [
            p[0] for p in SENSITIVE_FILES
            if ".npmrc" in p[0] or ".pypirc" in p[0]
        ]
        assert len(config_patterns) >= 2


class TestPatternIntegration:
    """Integration tests for pattern usage."""

    def test_can_iterate_all_patterns(self) -> None:
        """Test all patterns can be iterated."""
        count = 0
        for pattern, description, severity in SECRET_PATTERNS:
            count += 1
            # Just ensure we can access all tuple elements
            assert pattern is not None
            assert description is not None
            assert severity is not None
        assert count == len(SECRET_PATTERNS)

    def test_can_iterate_all_files(self) -> None:
        """Test all file patterns can be iterated."""
        count = 0
        for pattern, reason in SENSITIVE_FILES:
            count += 1
            assert pattern is not None
            assert reason is not None
        assert count == len(SENSITIVE_FILES)

    def test_severity_distribution(self) -> None:
        """Test patterns cover multiple severities."""
        severities = {p[2] for p in SECRET_PATTERNS}
        # Should have at least critical and high
        assert Severity.CRITICAL in severities
        assert Severity.HIGH in severities

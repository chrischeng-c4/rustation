"""Tests for feature name generator."""

from __future__ import annotations

from rstn.domain.specify.name_generator import (
    build_full_feature_name,
    generate_feature_name,
    normalize_feature_name,
    validate_feature_name,
)


class TestNormalizeFeatureName:
    """Tests for normalize_feature_name function."""

    def test_lowercase(self) -> None:
        """Test converting to lowercase."""
        assert normalize_feature_name("TestFeature") == "testfeature"
        assert normalize_feature_name("TEST") == "test"

    def test_spaces_to_hyphens(self) -> None:
        """Test converting spaces to hyphens."""
        assert normalize_feature_name("my feature") == "my-feature"
        assert normalize_feature_name("a b c") == "a-b-c"

    def test_underscores_to_hyphens(self) -> None:
        """Test converting underscores to hyphens."""
        assert normalize_feature_name("my_feature") == "my-feature"
        assert normalize_feature_name("a_b_c") == "a-b-c"

    def test_special_chars_removed(self) -> None:
        """Test removing special characters."""
        assert normalize_feature_name("test@feature!") == "testfeature"
        assert normalize_feature_name("feature#1") == "feature1"

    def test_collapse_hyphens(self) -> None:
        """Test collapsing multiple hyphens."""
        assert normalize_feature_name("test--feature") == "test-feature"
        assert normalize_feature_name("a---b----c") == "a-b-c"

    def test_strip_leading_trailing_hyphens(self) -> None:
        """Test removing leading/trailing hyphens."""
        assert normalize_feature_name("-test-") == "test"
        assert normalize_feature_name("---feature---") == "feature"

    def test_mixed_input(self) -> None:
        """Test mixed input."""
        assert normalize_feature_name("Add User_Authentication!") == "add-user-authentication"
        assert normalize_feature_name("  Test  Feature  ") == "test-feature"

    def test_empty_input(self) -> None:
        """Test empty input."""
        assert normalize_feature_name("") == ""
        assert normalize_feature_name("   ") == ""

    def test_already_normalized(self) -> None:
        """Test already normalized input."""
        assert normalize_feature_name("my-feature") == "my-feature"
        assert normalize_feature_name("test-feature-name") == "test-feature-name"


class TestGenerateFeatureName:
    """Tests for generate_feature_name function."""

    def test_basic_generation(self) -> None:
        """Test basic name generation."""
        name = generate_feature_name("Add user authentication")
        assert name == "add-user-authentication"

    def test_max_words_default(self) -> None:
        """Test default max words (4)."""
        name = generate_feature_name("Add a new user authentication system with OAuth support")
        # Stop words removed, limited to 4 words
        assert len(name.split("-")) <= 4

    def test_max_words_custom(self) -> None:
        """Test custom max words."""
        name = generate_feature_name("Add user auth system", max_words=2)
        assert len(name.split("-")) <= 2

    def test_stop_words_removed(self) -> None:
        """Test stop words are removed."""
        name = generate_feature_name("Add the user to the system")
        assert "the" not in name.split("-")
        assert "to" not in name.split("-")

    def test_preserves_key_words(self) -> None:
        """Test key words are preserved."""
        name = generate_feature_name("Add user authentication")
        assert "add" in name
        assert "user" in name
        assert "authentication" in name

    def test_short_description(self) -> None:
        """Test short description."""
        name = generate_feature_name("Login")
        assert name == "login"

    def test_empty_after_stop_words(self) -> None:
        """Test when all words are stop words."""
        name = generate_feature_name("a the to for")
        assert name == ""


class TestBuildFullFeatureName:
    """Tests for build_full_feature_name function."""

    def test_basic_build(self) -> None:
        """Test basic full name building."""
        full_name = build_full_feature_name("042", "worktree-management")
        assert full_name == "042-worktree-management"

    def test_with_leading_zeros(self) -> None:
        """Test with leading zeros."""
        full_name = build_full_feature_name("001", "first-feature")
        assert full_name == "001-first-feature"

    def test_three_digit_number(self) -> None:
        """Test three-digit number."""
        full_name = build_full_feature_name("100", "test")
        assert full_name == "100-test"


class TestValidateFeatureName:
    """Tests for validate_feature_name function."""

    def test_valid_simple(self) -> None:
        """Test valid simple name."""
        assert validate_feature_name("feature") is True
        assert validate_feature_name("test") is True

    def test_valid_with_hyphens(self) -> None:
        """Test valid name with hyphens."""
        assert validate_feature_name("my-feature") is True
        assert validate_feature_name("a-b-c") is True

    def test_valid_with_numbers(self) -> None:
        """Test valid name with numbers."""
        assert validate_feature_name("feature1") is True
        assert validate_feature_name("v2-auth") is True
        assert validate_feature_name("oauth2") is True

    def test_invalid_empty(self) -> None:
        """Test empty name is invalid."""
        assert validate_feature_name("") is False

    def test_invalid_uppercase(self) -> None:
        """Test uppercase is invalid."""
        assert validate_feature_name("Feature") is False
        assert validate_feature_name("TEST") is False

    def test_invalid_spaces(self) -> None:
        """Test spaces are invalid."""
        assert validate_feature_name("my feature") is False

    def test_invalid_underscores(self) -> None:
        """Test underscores are invalid."""
        assert validate_feature_name("my_feature") is False

    def test_invalid_leading_hyphen(self) -> None:
        """Test leading hyphen is invalid."""
        assert validate_feature_name("-feature") is False

    def test_invalid_trailing_hyphen(self) -> None:
        """Test trailing hyphen is invalid."""
        assert validate_feature_name("feature-") is False

    def test_invalid_consecutive_hyphens(self) -> None:
        """Test consecutive hyphens are invalid."""
        assert validate_feature_name("my--feature") is False

    def test_invalid_special_chars(self) -> None:
        """Test special characters are invalid."""
        assert validate_feature_name("feature!") is False
        assert validate_feature_name("test@name") is False


class TestIntegration:
    """Integration tests for name generator."""

    def test_normalize_then_validate(self) -> None:
        """Test normalizing then validating."""
        raw = "Add User_Authentication!"
        normalized = normalize_feature_name(raw)
        assert validate_feature_name(normalized) is True

    def test_generate_then_validate(self) -> None:
        """Test generating then validating."""
        description = "Implement OAuth 2.0 Authentication Flow"
        name = generate_feature_name(description)
        # Should be valid if not empty
        if name:
            assert validate_feature_name(name) is True

    def test_full_workflow(self) -> None:
        """Test full name generation workflow."""
        description = "Add worktree management feature"
        number = "042"

        # Generate name
        name = generate_feature_name(description)
        assert name  # Not empty

        # Validate
        assert validate_feature_name(name) is True

        # Build full name
        full_name = build_full_feature_name(number, name)
        assert full_name.startswith("042-")
        assert name in full_name

"""Tests for feature number allocator."""

from __future__ import annotations

import pytest
from rstn.domain.specify.number_allocator import (
    allocate_feature_number,
    format_feature_number,
    parse_feature_number,
    validate_feature_number,
)
from rstn.domain.specify.types import CatalogEntry, FeaturesCatalog


class TestFormatFeatureNumber:
    """Tests for format_feature_number function."""

    def test_single_digit(self) -> None:
        """Test formatting single digit numbers."""
        assert format_feature_number(1) == "001"
        assert format_feature_number(5) == "005"
        assert format_feature_number(9) == "009"

    def test_double_digit(self) -> None:
        """Test formatting double digit numbers."""
        assert format_feature_number(10) == "010"
        assert format_feature_number(42) == "042"
        assert format_feature_number(99) == "099"

    def test_triple_digit(self) -> None:
        """Test formatting triple digit numbers."""
        assert format_feature_number(100) == "100"
        assert format_feature_number(123) == "123"
        assert format_feature_number(999) == "999"

    def test_large_numbers(self) -> None:
        """Test formatting numbers > 999."""
        # Should work but not be zero-padded to 3
        assert format_feature_number(1000) == "1000"
        assert format_feature_number(9999) == "9999"

    def test_zero(self) -> None:
        """Test formatting zero."""
        assert format_feature_number(0) == "000"


class TestParseFeatureNumber:
    """Tests for parse_feature_number function."""

    def test_parse_zero_padded(self) -> None:
        """Test parsing zero-padded numbers."""
        assert parse_feature_number("001") == 1
        assert parse_feature_number("042") == 42
        assert parse_feature_number("100") == 100

    def test_parse_unpadded(self) -> None:
        """Test parsing unpadded numbers."""
        assert parse_feature_number("1") == 1
        assert parse_feature_number("42") == 42

    def test_parse_invalid_raises(self) -> None:
        """Test parsing invalid strings raises ValueError."""
        with pytest.raises(ValueError):
            parse_feature_number("abc")

        with pytest.raises(ValueError):
            parse_feature_number("")

        with pytest.raises(ValueError):
            parse_feature_number("12.5")


class TestAllocateFeatureNumber:
    """Tests for allocate_feature_number function."""

    def test_empty_catalog(self) -> None:
        """Test allocation with empty catalog."""
        catalog = FeaturesCatalog()
        number = allocate_feature_number(catalog)
        assert number == "001"

    def test_with_existing_features(self) -> None:
        """Test allocation with existing features."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="first"),
                CatalogEntry(number="002", name="second"),
            ]
        )
        number = allocate_feature_number(catalog)
        assert number == "003"

    def test_with_gaps(self) -> None:
        """Test allocation uses max+1, not filling gaps."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="first"),
                CatalogEntry(number="005", name="fifth"),
            ]
        )
        number = allocate_feature_number(catalog)
        assert number == "006"  # Not "002"

    def test_returns_formatted_string(self) -> None:
        """Test allocation returns zero-padded string."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number=f"{i:03d}", name=f"f{i}") for i in range(1, 10)]
        )
        number = allocate_feature_number(catalog)
        assert number == "010"
        assert len(number) == 3


class TestValidateFeatureNumber:
    """Tests for validate_feature_number function."""

    def test_valid_and_available(self) -> None:
        """Test valid number that's available."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="first")]
        )
        assert validate_feature_number("002", catalog) is True
        assert validate_feature_number("100", catalog) is True

    def test_already_used(self) -> None:
        """Test number that's already used."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="first")]
        )
        assert validate_feature_number("001", catalog) is False

    def test_invalid_format(self) -> None:
        """Test invalid number formats."""
        catalog = FeaturesCatalog()
        assert validate_feature_number("abc", catalog) is False
        assert validate_feature_number("", catalog) is False
        assert validate_feature_number("12.5", catalog) is False

    def test_zero_or_negative(self) -> None:
        """Test zero and negative numbers are invalid."""
        catalog = FeaturesCatalog()
        assert validate_feature_number("000", catalog) is False
        assert validate_feature_number("-1", catalog) is False

    def test_empty_catalog(self) -> None:
        """Test validation with empty catalog."""
        catalog = FeaturesCatalog()
        assert validate_feature_number("001", catalog) is True
        assert validate_feature_number("999", catalog) is True


class TestNumberAllocatorIntegration:
    """Integration tests for number allocator."""

    def test_allocation_workflow(self) -> None:
        """Test full allocation workflow."""
        catalog = FeaturesCatalog()

        # Allocate first number
        num1 = allocate_feature_number(catalog)
        assert num1 == "001"
        assert validate_feature_number(num1, catalog) is True

        # Add to catalog
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number=num1, name="first")]
        )

        # Allocate second number
        num2 = allocate_feature_number(catalog)
        assert num2 == "002"
        assert validate_feature_number(num2, catalog) is True

        # First number should no longer be valid
        assert validate_feature_number(num1, catalog) is False

    def test_format_parse_roundtrip(self) -> None:
        """Test format and parse are inverses."""
        for n in [1, 10, 42, 99, 100, 999]:
            formatted = format_feature_number(n)
            parsed = parse_feature_number(formatted)
            assert parsed == n

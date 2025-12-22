"""Test domain operations for rstn.

Provides test operations including:
- Test command effect creators
- Test output parsing
- Test result aggregation

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.test.runner import (
    create_cargo_test_effects,
    create_pytest_effects,
    parse_cargo_test_output,
    parse_pytest_output,
)
from rstn.domain.test.types import TestCase, TestResult, TestStatus, TestSuite

__all__ = [
    # Types
    "TestCase",
    "TestResult",
    "TestStatus",
    "TestSuite",
    # Runner functions
    "create_cargo_test_effects",
    "create_pytest_effects",
    "parse_cargo_test_output",
    "parse_pytest_output",
]

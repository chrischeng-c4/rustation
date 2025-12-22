"""Test domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class TestStatus(str, Enum):
    """Status of a test case."""

    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"
    IGNORED = "ignored"


class TestCase(BaseModel):
    """A single test case."""

    model_config = {"frozen": True}

    name: str = Field(description="Test name")
    status: TestStatus = Field(description="Test status")
    duration_ms: float | None = Field(default=None, description="Duration in ms")
    message: str | None = Field(default=None, description="Failure message if any")
    file_path: str | None = Field(default=None, description="Test file path")
    line: int | None = Field(default=None, description="Line number")


class TestSuite(BaseModel):
    """A collection of test cases."""

    model_config = {"frozen": True}

    name: str = Field(description="Suite name")
    tests: list[TestCase] = Field(default_factory=list, description="Test cases")
    duration_ms: float | None = Field(default=None, description="Total duration in ms")

    @property
    def passed_count(self) -> int:
        """Count of passed tests."""
        return sum(1 for t in self.tests if t.status == TestStatus.PASSED)

    @property
    def failed_count(self) -> int:
        """Count of failed tests."""
        return sum(1 for t in self.tests if t.status == TestStatus.FAILED)

    @property
    def skipped_count(self) -> int:
        """Count of skipped tests."""
        return sum(1 for t in self.tests if t.status == TestStatus.SKIPPED)


class TestResult(BaseModel):
    """Result of a test run."""

    model_config = {"frozen": True}

    success: bool = Field(description="Whether all tests passed")
    suites: list[TestSuite] = Field(default_factory=list, description="Test suites")
    total_duration_ms: float | None = Field(default=None, description="Total duration")
    stdout: str = Field(default="", description="Standard output")
    stderr: str = Field(default="", description="Standard error")

    @property
    def total_tests(self) -> int:
        """Total number of tests."""
        return sum(len(s.tests) for s in self.suites)

    @property
    def total_passed(self) -> int:
        """Total passed tests."""
        return sum(s.passed_count for s in self.suites)

    @property
    def total_failed(self) -> int:
        """Total failed tests."""
        return sum(s.failed_count for s in self.suites)

    @property
    def total_skipped(self) -> int:
        """Total skipped tests."""
        return sum(s.skipped_count for s in self.suites)

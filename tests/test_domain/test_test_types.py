"""Tests for test domain types."""

from __future__ import annotations

import pytest
from rstn.domain.test.types import (
    TestCase,
    TestResult,
    TestStatus,
    TestSuite,
)


class TestTestStatus:
    """Tests for TestStatus enum."""

    def test_status_values(self) -> None:
        """Test all status values."""
        assert TestStatus.PASSED.value == "passed"
        assert TestStatus.FAILED.value == "failed"
        assert TestStatus.SKIPPED.value == "skipped"
        assert TestStatus.IGNORED.value == "ignored"

    def test_status_is_string_enum(self) -> None:
        """Test TestStatus is a string enum."""
        for status in TestStatus:
            assert isinstance(status.value, str)


class TestTestCase:
    """Tests for TestCase model."""

    def test_test_case_passed(self) -> None:
        """Test passed test case."""
        case = TestCase(
            name="test_add",
            status=TestStatus.PASSED,
            duration_ms=10.5,
        )
        assert case.name == "test_add"
        assert case.status == TestStatus.PASSED
        assert case.duration_ms == 10.5
        assert case.message is None
        assert case.file_path is None
        assert case.line is None

    def test_test_case_failed(self) -> None:
        """Test failed test case."""
        case = TestCase(
            name="test_divide",
            status=TestStatus.FAILED,
            message="assertion failed: expected 2, got 3",
            file_path="tests/test_math.rs",
            line=42,
        )
        assert case.status == TestStatus.FAILED
        assert case.message == "assertion failed: expected 2, got 3"
        assert case.file_path == "tests/test_math.rs"
        assert case.line == 42

    def test_test_case_skipped(self) -> None:
        """Test skipped test case."""
        case = TestCase(
            name="test_feature",
            status=TestStatus.SKIPPED,
        )
        assert case.status == TestStatus.SKIPPED

    def test_test_case_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        case = TestCase(
            name="test_something",
            status=TestStatus.PASSED,
            duration_ms=5.0,
        )
        json_str = case.model_dump_json()
        restored = TestCase.model_validate_json(json_str)
        assert restored == case

    def test_test_case_immutable(self) -> None:
        """Test case is immutable (frozen)."""
        case = TestCase(name="test", status=TestStatus.PASSED)
        with pytest.raises(Exception):
            case.status = TestStatus.FAILED  # type: ignore


class TestTestSuite:
    """Tests for TestSuite model."""

    def test_empty_suite(self) -> None:
        """Test empty test suite."""
        suite = TestSuite(name="empty")
        assert suite.name == "empty"
        assert suite.tests == []
        assert suite.duration_ms is None
        assert suite.passed_count == 0
        assert suite.failed_count == 0
        assert suite.skipped_count == 0

    def test_suite_with_tests(self) -> None:
        """Test suite with various tests."""
        tests = [
            TestCase(name="test1", status=TestStatus.PASSED),
            TestCase(name="test2", status=TestStatus.PASSED),
            TestCase(name="test3", status=TestStatus.FAILED, message="Error"),
            TestCase(name="test4", status=TestStatus.SKIPPED),
            TestCase(name="test5", status=TestStatus.IGNORED),
        ]
        suite = TestSuite(name="my_suite", tests=tests, duration_ms=100.0)

        assert len(suite.tests) == 5
        assert suite.duration_ms == 100.0
        assert suite.passed_count == 2
        assert suite.failed_count == 1
        assert suite.skipped_count == 1

    def test_suite_all_passed(self) -> None:
        """Test suite with all passed tests."""
        tests = [
            TestCase(name=f"test{i}", status=TestStatus.PASSED) for i in range(5)
        ]
        suite = TestSuite(name="all_passed", tests=tests)

        assert suite.passed_count == 5
        assert suite.failed_count == 0

    def test_suite_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        tests = [
            TestCase(name="test1", status=TestStatus.PASSED),
        ]
        suite = TestSuite(name="suite", tests=tests, duration_ms=50.0)
        json_str = suite.model_dump_json()
        restored = TestSuite.model_validate_json(json_str)
        assert restored.name == suite.name
        assert restored.passed_count == suite.passed_count

    def test_suite_immutable(self) -> None:
        """Test suite is immutable (frozen)."""
        suite = TestSuite(name="test")
        with pytest.raises(Exception):
            suite.name = "new_name"  # type: ignore


class TestTestResult:
    """Tests for TestResult model."""

    def test_empty_result(self) -> None:
        """Test empty test result."""
        result = TestResult(success=True)
        assert result.success is True
        assert result.suites == []
        assert result.total_duration_ms is None
        assert result.stdout == ""
        assert result.stderr == ""
        assert result.total_tests == 0
        assert result.total_passed == 0
        assert result.total_failed == 0
        assert result.total_skipped == 0

    def test_result_with_suites(self) -> None:
        """Test result with multiple suites."""
        suites = [
            TestSuite(
                name="unit",
                tests=[
                    TestCase(name="test1", status=TestStatus.PASSED),
                    TestCase(name="test2", status=TestStatus.PASSED),
                ],
            ),
            TestSuite(
                name="integration",
                tests=[
                    TestCase(name="test3", status=TestStatus.PASSED),
                    TestCase(name="test4", status=TestStatus.FAILED, message="Error"),
                    TestCase(name="test5", status=TestStatus.SKIPPED),
                ],
            ),
        ]
        result = TestResult(
            success=False,
            suites=suites,
            total_duration_ms=500.0,
            stderr="Error in test4",
        )

        assert result.success is False
        assert result.total_duration_ms == 500.0
        assert result.total_tests == 5
        assert result.total_passed == 3
        assert result.total_failed == 1
        assert result.total_skipped == 1

    def test_result_success_all_passed(self) -> None:
        """Test successful result with all passed."""
        suites = [
            TestSuite(
                name="tests",
                tests=[
                    TestCase(name=f"test{i}", status=TestStatus.PASSED)
                    for i in range(10)
                ],
            ),
        ]
        result = TestResult(
            success=True,
            suites=suites,
            stdout="All tests passed!",
        )

        assert result.success is True
        assert result.total_tests == 10
        assert result.total_passed == 10
        assert result.total_failed == 0

    def test_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        suites = [
            TestSuite(
                name="tests",
                tests=[TestCase(name="test1", status=TestStatus.PASSED)],
            ),
        ]
        result = TestResult(success=True, suites=suites)
        json_str = result.model_dump_json()
        restored = TestResult.model_validate_json(json_str)
        assert restored.success == result.success
        assert restored.total_tests == result.total_tests

    def test_result_immutable(self) -> None:
        """Test result is immutable (frozen)."""
        result = TestResult(success=True)
        with pytest.raises(Exception):
            result.success = False  # type: ignore


class TestTestResultIntegration:
    """Integration tests for test result types."""

    def test_calculate_pass_rate(self) -> None:
        """Test calculating pass rate from result."""
        suites = [
            TestSuite(
                name="tests",
                tests=[
                    TestCase(name=f"pass{i}", status=TestStatus.PASSED)
                    for i in range(8)
                ]
                + [
                    TestCase(name=f"fail{i}", status=TestStatus.FAILED, message="Error")
                    for i in range(2)
                ],
            ),
        ]
        result = TestResult(success=False, suites=suites)

        pass_rate = result.total_passed / result.total_tests if result.total_tests > 0 else 0.0
        assert pass_rate == 0.8  # 80%

    def test_filter_failed_tests(self) -> None:
        """Test filtering failed tests from result."""
        suites = [
            TestSuite(
                name="suite1",
                tests=[
                    TestCase(name="test1", status=TestStatus.PASSED),
                    TestCase(name="test2", status=TestStatus.FAILED, message="Error1"),
                ],
            ),
            TestSuite(
                name="suite2",
                tests=[
                    TestCase(name="test3", status=TestStatus.FAILED, message="Error2"),
                ],
            ),
        ]
        result = TestResult(success=False, suites=suites)

        failed_tests = [
            (suite.name, test.name, test.message)
            for suite in result.suites
            for test in suite.tests
            if test.status == TestStatus.FAILED
        ]

        assert len(failed_tests) == 2
        assert ("suite1", "test2", "Error1") in failed_tests
        assert ("suite2", "test3", "Error2") in failed_tests

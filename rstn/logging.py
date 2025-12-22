"""Structured logging for rstn.

Uses JSON format for structured logs written to ~/.rstn/logs/.
"""

from __future__ import annotations

import json
import logging
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


class JsonFormatter(logging.Formatter):
    """JSON formatter for structured logging."""

    def format(self, record: logging.LogRecord) -> str:
        """Format log record as JSON."""
        log_entry: dict[str, Any] = {
            "timestamp": datetime.now(UTC).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }

        # Add extra fields if present
        if hasattr(record, "data") and record.data:
            log_entry["data"] = record.data

        # Add exception info if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)

        # Add source location
        log_entry["source"] = {
            "file": record.filename,
            "line": record.lineno,
            "function": record.funcName,
        }

        return json.dumps(log_entry, default=str)


class StructuredLogger:
    """Structured logger wrapper."""

    def __init__(self, logger: logging.Logger) -> None:
        self._logger = logger

    def debug(self, msg: str, **data: Any) -> None:
        """Log debug message with structured data."""
        self._log(logging.DEBUG, msg, data)

    def info(self, msg: str, **data: Any) -> None:
        """Log info message with structured data."""
        self._log(logging.INFO, msg, data)

    def warning(self, msg: str, **data: Any) -> None:
        """Log warning message with structured data."""
        self._log(logging.WARNING, msg, data)

    def error(self, msg: str, **data: Any) -> None:
        """Log error message with structured data."""
        self._log(logging.ERROR, msg, data)

    def exception(self, msg: str, **data: Any) -> None:
        """Log exception with structured data."""
        self._logger.exception(msg, extra={"data": data} if data else {})

    def _log(self, level: int, msg: str, data: dict[str, Any]) -> None:
        """Internal log method."""
        extra = {"data": data} if data else {}
        self._logger.log(level, msg, extra=extra)


def get_log_dir() -> Path:
    """Get the log directory path."""
    log_dir = Path.home() / ".rstn" / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    return log_dir


def setup_logging(verbose: bool = False) -> Path:
    """Setup structured logging.

    Args:
        verbose: If True, also log to stderr

    Returns:
        Path to the log file
    """
    # Generate log filename with timestamp
    timestamp = datetime.now().strftime("%Y-%m-%d-%H%M%S")
    log_file = get_log_dir() / f"rstn-py.{timestamp}.log"

    # Configure root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)

    # Clear existing handlers
    root_logger.handlers.clear()

    # File handler with JSON format
    file_handler = logging.FileHandler(log_file, encoding="utf-8")
    file_handler.setLevel(logging.DEBUG)
    file_handler.setFormatter(JsonFormatter())
    root_logger.addHandler(file_handler)

    # Console handler - human readable, only if verbose
    if verbose:
        console_handler = logging.StreamHandler(sys.stderr)
        console_handler.setLevel(logging.INFO)
        console_handler.setFormatter(
            logging.Formatter("%(levelname)s [%(name)s] %(message)s")
        )
        root_logger.addHandler(console_handler)

    # Log startup
    logger = get_logger("rstn")
    logger.info("rstn started", log_file=str(log_file), verbose=verbose)

    return log_file


def get_logger(name: str) -> StructuredLogger:
    """Get a structured logger.

    Args:
        name: Logger name (e.g., "rstn.cli", "rstn.tui")

    Returns:
        StructuredLogger instance
    """
    return StructuredLogger(logging.getLogger(name))

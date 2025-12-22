"""Security patterns for git scanning.

Regex patterns for detecting secrets and sensitive files.
"""

from __future__ import annotations

from rstn.domain.git.types import Severity

# Secret detection patterns
# Format: (regex_pattern, description, severity)
SECRET_PATTERNS: list[tuple[str, str, Severity]] = [
    # Critical (block commit)
    (
        r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
        "Private key detected",
        Severity.CRITICAL,
    ),
    (
        r"-----BEGIN CERTIFICATE-----",
        "Certificate detected (may contain private key)",
        Severity.CRITICAL,
    ),
    # High (warn but allow)
    (
        r'(?i)api[_-]?key["\']?\s*[:=]\s*["\'][^"\']{10,}["\']',
        "API key pattern",
        Severity.HIGH,
    ),
    (
        r'(?i)secret[_-]?key["\']?\s*[:=]\s*["\'][^"\']{10,}["\']',
        "Secret key pattern",
        Severity.HIGH,
    ),
    (
        r'(?i)password["\']?\s*[:=]\s*["\'][^"\']{8,}["\']',
        "Password pattern",
        Severity.HIGH,
    ),
    (
        r'(?i)token["\']?\s*[:=]\s*["\'][^"\']{20,}["\']',
        "Token pattern",
        Severity.HIGH,
    ),
    (
        r'(?i)auth[_-]?token["\']?\s*[:=]\s*["\'][^"\']{20,}["\']',
        "Auth token pattern",
        Severity.HIGH,
    ),
    (r"gh[ps]_[a-zA-Z0-9]{36,}", "GitHub token", Severity.HIGH),
    (r"sk-[a-zA-Z0-9]{20,}", "OpenAI/Anthropic API key", Severity.HIGH),
    (r"AIza[0-9A-Za-z\\-_]{35}", "Google API key", Severity.HIGH),
    (
        r'(?i)aws[_-]?access[_-]?key[_-]?id["\']?\s*[:=]\s*["\'][^"\']{16,}["\']',
        "AWS Access Key",
        Severity.HIGH,
    ),
    # Medium (info only)
    (
        r"[a-zA-Z0-9+/]{40,}={0,2}",
        "Base64 string (possible secret)",
        Severity.MEDIUM,
    ),
    (
        r"[0-9a-fA-F]{64,}",
        "Long hex string (possible key)",
        Severity.MEDIUM,
    ),
]

# Sensitive filename patterns
# Format: (pattern, reason)
SENSITIVE_FILES: list[tuple[str, str]] = [
    (".env", "Environment file with secrets"),
    (".env.", "Environment file variant"),
    ("credentials.json", "Credentials file"),
    ("secrets.yaml", "Secrets configuration"),
    ("secrets.yml", "Secrets configuration"),
    ("*.pem", "Private key file"),
    ("*.key", "Private key file"),
    ("id_rsa", "SSH private key"),
    ("id_ed25519", "SSH private key"),
    ("id_ecdsa", "SSH private key"),
    ("*.pfx", "Certificate file"),
    ("*.p12", "Certificate file"),
    (".npmrc", "NPM credentials"),
    (".pypirc", "PyPI credentials"),
]

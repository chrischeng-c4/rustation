#!/bin/bash
# Parse Gemini output and create OpenSpec files
# Expected input: Gemini output with FILE markers via stdin

set -euo pipefail

CHANGE_ID="${1:-}"

if [[ -z "$CHANGE_ID" ]]; then
    echo "Error: change-id required" >&2
    echo "Usage: cat gemini-output.txt | $0 <change-id>" >&2
    exit 1
fi

PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
CHANGE_DIR="${PROJECT_ROOT}/openspec/changes/${CHANGE_ID}"

# Ensure change directory exists
mkdir -p "${CHANGE_DIR}/specs"

# Parse FILE markers and create files
awk -v change_dir="${CHANGE_DIR}" '
BEGIN {
    in_file = 0
    file_path = ""
    content = ""
}

/^=== FILE: / {
    # Save previous file
    if (in_file && file_path != "") {
        full_path = change_dir "/" file_path
        # Create directory if needed
        dir = full_path
        sub(/[^/]*$/, "", dir)
        system("mkdir -p " dir)
        # Write content
        print content > full_path
        close(full_path)
        print "[Created] " file_path
    }

    # Start new file
    in_file = 1
    # Extract path after "FILE:"
    file_path = $0
    sub(/^=== FILE: */, "", file_path)
    sub(/ *=== *$/, "", file_path)
    content = ""
    next
}

/^=== END FILE ===/ {
    # Save current file
    if (in_file && file_path != "") {
        full_path = change_dir "/" file_path
        # Create directory if needed
        dir = full_path
        sub(/[^/]*$/, "", dir)
        system("mkdir -p " dir)
        # Write content
        print content > full_path
        close(full_path)
        print "[Created] " file_path
    }

    in_file = 0
    file_path = ""
    content = ""
    next
}

in_file {
    if (content != "") {
        content = content "\n" $0
    } else {
        content = $0
    }
}
'

echo ""
echo "Files created in: ${CHANGE_DIR}"

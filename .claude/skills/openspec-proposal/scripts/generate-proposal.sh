#!/bin/bash
# OpenSpec Proposal Generator using Gemini CLI
# Called by openspec-proposal skill

set -euo pipefail

# Arguments
CHANGE_ID="${1:-}"
USER_PROMPT="${2:-}"

if [[ -z "$CHANGE_ID" || -z "$USER_PROMPT" ]]; then
    echo "Error: Missing required arguments" >&2
    echo "Usage: $0 <change-id> <user-prompt>" >&2
    exit 1
fi

# Change to project root (gemini CLI looks for GEMINI.md here)
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$PROJECT_ROOT"

# Ensure change directory exists
CHANGE_DIR="openspec/changes/${CHANGE_ID}"
mkdir -p "${CHANGE_DIR}/specs"

# Call Gemini CLI
# Note: Gemini automatically reads GEMINI.md from project root as system prompt
gemini -p "${USER_PROMPT}" -o text

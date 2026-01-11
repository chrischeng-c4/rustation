#!/bin/bash
# Run Gemini CLI for codebase exploration
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Usage information
usage() {
    cat << EOF
Usage: $0 "<user-question>"

Arguments:
  user-question   Question about the codebase (e.g., "How does chat work?")

Example:
  $0 "Where is Docker orchestration implemented?"

Environment:
  PROJECT_ROOT   Project root directory (default: git root or current directory)
EOF
    exit 1
}

# Validate arguments
if [[ $# -lt 1 ]]; then
    echo -e "${RED}Error: Missing user question${NC}" >&2
    usage
fi

USER_QUESTION="$1"

# Find project root
PROJECT_ROOT="${PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$PROJECT_ROOT"

echo -e "${GREEN}🔍 Exploring codebase...${NC}"
echo "Question: ${USER_QUESTION}"
echo ""

# Check if Gemini CLI is available
if ! command -v gemini &> /dev/null; then
    echo -e "${RED}Error: Gemini CLI not found${NC}" >&2
    echo "Install from: https://geminicli.com" >&2
    exit 1
fi

# Build comprehensive prompt
GEMINI_PROMPT=$(cat << EOF
## User Question
${USER_QUESTION}

## Instructions
You are exploring the rustation codebase to answer the user's question.

1. **Start with KB** (avoid blind searching):
   - Read /GEMINI.md for architecture context and exploration strategy
   - Read dev-docs/architecture/ if relevant
   - Read openspec/specs/ for feature specifications

2. **Understand State Structure**:
   - Read packages/core/src/app_state.rs to see full state tree
   - Identify which part of state is relevant

3. **Find Existing Patterns**:
   - Search packages/core/src/reducer/ for similar features
   - Search desktop/src/renderer/src/features/ for UI examples
   - Look for test files to understand expected behavior

4. **Map Data Flow**:
   - Frontend: Component → dispatch(action) → IPC
   - Bridge: window.api.* → @rstn/core
   - Backend: action → reducer → new state → notify frontend
   - Frontend: useAppState() → re-render

5. **Provide findings** in this format:

## Architecture Understanding
[Explain how the relevant parts work]

## Key Files
- path/to/file.rs:123 - [what this code does]
- path/to/component.tsx:45 - [component purpose]
(Use exact file paths with line numbers)

## Data Flow
[Show how data moves through the system]
Frontend → IPC → Backend → State → Frontend

## Implementation Recommendations
[Best practices to follow, existing patterns to reuse]

## Risks & Considerations
[Edge cases, performance concerns, security issues]

**CRITICAL:**
- DO NOT write implementation code (.rs, .ts, .tsx files)
- DO provide file paths with line numbers
- DO explain data flow clearly
- DO reference existing patterns
EOF
)

# Log file for debugging
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="/tmp/gemini-explore-${TIMESTAMP}.jsonl"

# Call Gemini CLI with streaming output
echo -e "${YELLOW}Calling Gemini CLI...${NC}"
echo "Log: ${LOG_FILE}"
echo ""

if echo "$GEMINI_PROMPT" | gemini --output-format stream-json 2>&1 | tee "$LOG_FILE"; then
    echo ""
    echo -e "${GREEN}✅ Exploration completed${NC}"
    echo ""
    echo "Debug log: cat ${LOG_FILE}"
else
    echo ""
    echo -e "${RED}❌ Exploration failed${NC}" >&2
    echo ""
    echo "Troubleshooting:" >&2
    echo "  - Check log: cat ${LOG_FILE}" >&2
    echo "  - Verify GEMINI.md exists and has architecture reference" >&2
    echo "  - Ensure Gemini CLI is properly configured" >&2
    exit 1
fi

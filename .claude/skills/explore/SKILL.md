---
name: explore
description: Deep codebase exploration and planning using Gemini CLI (2M context). Use for understanding architecture, finding patterns, or planning implementations.
user-invocable: true
category: Development
tags: [explore, plan, analysis, architecture]
---

# Explore Skill

Use Gemini CLI's 2M token context to deeply explore the codebase and generate implementation plans.

## When to Use This Skill

Automatically use when the user:
- Asks "**how does X work?**" or "**where is Y implemented?**"
- Wants to **understand architecture** or **find patterns**
- Needs a **plan** for implementing a feature (outside OpenSpec workflow)
- Requests **analysis** of existing code
- Says "**explore**", "**analyze**", or "**plan this**"

**Do NOT use for:**
- Large features requiring OpenSpec specs (use `openspec-proposal` instead)
- Direct implementation (Claude should implement directly)
- Simple questions answerable by reading 1-3 files (Claude can read directly)

---

## Instructions

### Guardrails

- This skill uses **Gemini CLI** for exploration (2M context window)
- Gemini will **analyze code and generate plans**, but **NOT write implementation code**
- System prompt is in `/GEMINI.md` (Architecture Reference + Exploration Strategy)
- Ask clarifying questions before calling Gemini if request is vague
- Keep exploration focused on user's specific question

### Steps

1. **Preparation**
   - Understand user's question/request
   - Identify what needs to be explored:
     - Architecture understanding ("how does X work?")
     - Pattern finding ("where is Y implemented?")
     - Implementation planning ("how should I add Z?")
     - Code analysis ("analyze feature W")

2. **Call Gemini via helper script**
   ```bash
   .claude/skills/explore/scripts/run-explore.sh "<user-question>"
   ```

   The script will:
   - Call Gemini CLI with the user's question
   - Gemini reads GEMINI.md for architecture map and strategy
   - Gemini explores codebase using read/search tools
   - Gemini generates findings and recommendations
   - Output streams to console in real-time
   - Log saved to `/tmp/gemini-explore-TIMESTAMP.jsonl`

3. **Review Gemini's findings**
   - Read the exploration results
   - Verify the analysis makes sense
   - Check that key files are identified
   - Ensure recommendations are actionable

4. **Present results to user**
   Format:
   ```
   ## Exploration Results

   ### Architecture Understanding
   [Gemini's findings about how X works]

   ### Key Files
   - path/to/file.rs:123 - [purpose and relevant code]
   - path/to/file.tsx:45 - [component description]

   ### Data Flow
   [How data moves: Frontend → IPC → Backend → State → Frontend]

   ### Implementation Recommendations
   [Best practices, patterns to follow, existing code to reuse]

   ### Risks & Considerations
   [Edge cases, performance concerns, security issues]

   ---

   **Next Steps:**
   Based on this analysis, I can [describe what Claude will do next].
   [If user wants implementation, Claude implements directly]
   ```

5. **Follow-up implementation** (if applicable)
   - If exploration reveals complexity > 500 LOC, suggest `openspec-proposal`
   - If exploration shows simple approach < 500 LOC, Claude implements directly
   - If user wants to refine exploration, call skill again with more specific question

---

## Examples

### Example 1: Understanding Architecture

**User:** "How does the chat feature work?"

**Your response:**
1. Call explore skill:
   ```bash
   .claude/skills/explore/scripts/run-explore.sh "How is the chat feature implemented? What are the key components and data flow?"
   ```

2. Gemini analyzes:
   - `packages/core/src/reducer/chat.rs` (state transitions)
   - `packages/core/src/app_state.rs` (ChatState definition)
   - `desktop/src/renderer/src/features/chat/` (UI components)
   - Data flow: ChatPage → dispatch(SendChatMessage) → reducer → state update → re-render

3. Present findings:
   ```
   ## Chat Feature Architecture

   ### How It Works
   The chat feature uses the State-First architecture:
   1. User types message in ChatPage.tsx
   2. Component dispatches SendChatMessage action via window.stateApi
   3. Rust reducer (chat.rs) adds user message to state
   4. Reducer triggers AI response generation
   5. State updates stream back to frontend
   6. ChatPage re-renders with new messages

   ### Key Files
   - packages/core/src/app_state.rs:234 - ChatState definition
   - packages/core/src/reducer/chat.rs:18 - SendChatMessage handler
   - desktop/src/renderer/src/features/chat/ChatPage.tsx:67 - UI component
   - desktop/src/preload/index.ts:123 - IPC bridge for dispatch

   ### Data Flow
   Frontend (ChatPage) → window.stateApi.dispatch(SendChatMessage)
   → IPC Bridge (preload) → Rust Backend (reducer/chat.rs)
   → State Update → Notify Frontend → useAppState() → Re-render

   ---

   **Next:** What would you like to modify about the chat feature?
   ```

### Example 2: Planning Implementation

**User:** "I want to add markdown support to chat messages. How should I do this?"

**Your response:**
1. Call explore skill:
   ```bash
   .claude/skills/explore/scripts/run-explore.sh "Analyze chat message rendering. How should I add markdown support? What libraries or patterns exist?"
   ```

2. Gemini finds:
   - Current: Plain text rendering in ChatMessage component
   - Existing markdown: FilePreview uses react-markdown
   - Pattern: SourceCodeViewer uses syntax highlighting

3. Present plan:
   ```
   ## Markdown Support Implementation Plan

   ### Current Implementation
   - ChatMessage.tsx renders plain text (line 45: `{message.content}`)
   - No markdown parsing currently

   ### Existing Patterns
   - FilePreview.tsx:89 uses `react-markdown` library
   - Already in package.json: `react-markdown: ^9.0.0`

   ### Recommended Approach

   1. **Update ChatMessage Component** (Simple, < 50 LOC)
      - Import ReactMarkdown from 'react-markdown'
      - Replace plain text with <ReactMarkdown>{message.content}</ReactMarkdown>
      - Add CSS for markdown styles (headings, lists, code blocks)

   2. **Add Syntax Highlighting** (Optional, +30 LOC)
      - Use rehype-highlight plugin (already used in FilePreview)
      - Import: `import rehypeHighlight from 'rehype-highlight'`
      - Pass as plugin to ReactMarkdown

   3. **Test** (< 10 LOC)
      - Add test case in ChatPage.test.tsx
      - Verify markdown renders correctly

   ### Estimated Complexity
   **Simple** (Total ~90 LOC, 1 file)

   ---

   **Next:** I'll implement this directly. Starting with ChatMessage.tsx...
   ```

### Example 3: Finding Patterns

**User:** "Where do we handle Docker operations?"

**Your response:**
1. Call explore skill:
   ```bash
   .claude/skills/explore/scripts/run-explore.sh "Find all Docker-related code. Where are operations implemented?"
   ```

2. Gemini searches:
   - `packages/core/src/docker.rs` (backend operations)
   - `packages/core/src/reducer/docker.rs` (state transitions)
   - `desktop/src/renderer/src/features/dockers/` (UI)
   - `desktop/src/preload/index.ts` (IPC bridge)

3. Present findings:
   ```
   ## Docker Operations Architecture

   ### Backend (Rust)
   - packages/core/src/docker.rs:23 - `DockerManager` struct
   - packages/core/src/docker.rs:145 - `list_containers()` function
   - packages/core/src/docker.rs:201 - `start_container()` function
   - packages/core/src/reducer/docker.rs:15 - State transitions

   ### Frontend (React)
   - desktop/src/renderer/src/features/dockers/DockersPage.tsx - Main UI
   - desktop/src/renderer/src/features/dockers/DockerServiceCard.tsx - Container card
   - desktop/src/renderer/src/features/dockers/AddDbDialog.tsx - Create database

   ### IPC Bridge
   - desktop/src/preload/index.ts:234 - dockerApi exposed to renderer

   ### Data Flow
   1. User clicks "Start" button
   2. DockerServiceCard dispatches StartContainer action
   3. IPC sends to Rust backend
   4. docker.rs calls Docker API
   5. Reducer updates state
   6. Frontend re-renders with new container status

   ---

   **Next:** What Docker operation would you like to modify?
   ```

---

## Reference

- Gemini will use `/GEMINI.md` for architecture context
- Exploration strategy: KB → State → Patterns → Data Flow
- Output includes file paths with line numbers (format: `path/to/file.rs:123`)
- Logs saved to `/tmp/gemini-explore-TIMESTAMP.jsonl` for debugging

---

## After Completion

Based on exploration results:
1. **Simple task (< 500 LOC)** → Claude implements directly
2. **Complex task (> 500 LOC)** → Suggest using `openspec-proposal` skill
3. **Need more detail** → Call explore again with refined question
4. **Need plan approval** → Present plan and wait for user confirmation

# Workflow System Architecture

> **Status**: Design Draft
> **Last Updated**: 2024-12-29

## Overview

Workflow System 是 rstn 的核心功能，提供狀態機驅動的引導式工作流程。每個 Workflow 由多個 Node 組成，Node 可以調用 Claude Code 作為執行引擎。

### 設計原則

1. **Workflow = State Machine** — 每個 workflow 是明確定義的狀態機
2. **Claude as Node Capability** — Claude Code 不是獨立功能，而是 node 的執行能力
3. **Separation of Concerns** — Workflows (引導式) vs Tasks (簡單執行) 清楚分離

### Tab 結構

```
Sidebar Navigation:
├── Workflows    → State machine workflows (guided, multi-step)
├── Tasks        → Justfile commands only (simple, fire-and-forget)
├── Dockers      → Container management
└── Settings     → Configuration
```

---

## Workflow Architecture

### Conceptual Model

```
┌─────────────────────────────────────────────────────────┐
│                    Workflow Definition                   │
│  (YAML/JSON metadata + node graph)                      │
├─────────────────────────────────────────────────────────┤
│                    Workflow Runtime                      │
│  - State machine executor                               │
│  - Node dispatcher                                      │
│  - Progress tracking                                    │
├─────────────────────────────────────────────────────────┤
│                    Node Executors                        │
│  ├── ClaudeCodeNode  (invoke Claude CLI)                │
│  ├── SystemCheckNode (check env/files)                  │
│  ├── FileWriteNode   (write output files)               │
│  ├── UserInputNode   (wait for user input)              │
│  └── BranchNode      (conditional routing)              │
└─────────────────────────────────────────────────────────┘
```

### State Machine Pattern

每個 Workflow 的生命週期：

```
                    ┌──────────────┐
                    │    Idle      │
                    └──────┬───────┘
                           │ start()
                           ▼
                    ┌──────────────┐
            ┌───────│   Running    │◄─────────┐
            │       └──────┬───────┘          │
            │              │                  │
            │   ┌──────────┼──────────┐       │
            │   ▼          ▼          ▼       │
         ┌──────────┐ ┌──────────┐ ┌──────────┐
         │ Waiting  │ │Executing │ │ Paused   │
         │  Input   │ │  Node    │ │          │
         └────┬─────┘ └────┬─────┘ └────┬─────┘
              │            │            │
              └────────────┴────────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
     ┌──────────┐   ┌──────────┐   ┌──────────┐
     │ Complete │   │  Failed  │   │ Cancelled│
     └──────────┘   └──────────┘   └──────────┘
```

---

## Node Types

### 1. ClaudeCodeNode

調用 Claude CLI 執行 AI 任務。

```yaml
node:
  id: generate-constitution
  type: claude-code
  config:
    prompt_template: |
      Analyze the codebase and generate a constitution file.
      Languages detected: {{detected_languages}}
    streaming: true
    output_variable: constitution_content
  transitions:
    on_success: save-file
    on_error: handle-error
```

**執行流程**：
1. 準備 prompt（模板變數替換）
2. 調用 Claude CLI（streaming mode）
3. 將輸出寫入 workflow context
4. 觸發 transition

### 2. SystemCheckNode

檢查系統狀態或檔案存在。

```yaml
node:
  id: check-prerequisites
  type: system-check
  config:
    checks:
      - type: file-exists
        path: .rstn/constitutions/
        store_as: constitution_exists
      - type: command-exists
        command: claude
        store_as: claude_available
  transitions:
    on_success: next-node
    on_failure: show-error
```

### 3. FileWriteNode

寫入檔案到指定位置。

```yaml
node:
  id: save-constitution
  type: file-write
  config:
    path: .rstn/constitutions/custom.md
    content_from: constitution_content  # 從 context 取值
    create_dirs: true
  transitions:
    on_success: verify-result
    on_error: handle-error
```

### 4. UserInputNode

等待使用者輸入。

```yaml
node:
  id: collect-project-info
  type: user-input
  config:
    fields:
      - id: project_name
        label: "Project Name"
        type: text
        required: true
      - id: use_typescript
        label: "Use TypeScript?"
        type: boolean
        default: true
  transitions:
    on_submit: process-input
    on_cancel: cancelled
```

### 5. BranchNode

條件分支路由。

```yaml
node:
  id: check-existing
  type: branch
  config:
    condition: constitution_exists
    branches:
      - when: true
        goto: show-existing-options
      - when: false
        goto: start-generation
```

### 6. OptionsNode

顯示選項讓使用者選擇。

```yaml
node:
  id: choose-method
  type: options
  config:
    title: "How would you like to proceed?"
    options:
      - id: default
        label: "Use Default Template"
        description: "Auto-detect languages and apply optimized rules"
        goto: apply-default
      - id: custom
        label: "Custom Q&A"
        description: "Answer questions to generate custom constitution"
        goto: start-qa
```

---

## Workflow Definition Format

Workflow 使用 YAML 定義，存放於 `.rstn/workflows/` 或內建於應用程式。

```yaml
# .rstn/workflows/constitution-management.yaml
workflow:
  id: constitution-management
  name: "Constitution Management"
  description: "Initialize or update project constitution"
  version: "1.0.0"
  icon: "scroll"  # Lucide icon name

context:
  # Workflow-level variables
  detected_languages: []
  constitution_exists: false
  constitution_content: ""

nodes:
  - id: start
    type: system-check
    config:
      checks:
        - type: file-exists
          path: .rstn/constitution.md
          store_as: constitution_exists
    transitions:
      on_success: check-existing

  - id: check-existing
    type: branch
    config:
      condition: constitution_exists
      branches:
        - when: true
          goto: existing-options
        - when: false
          goto: choose-method

  - id: existing-options
    type: options
    config:
      title: "Constitution already exists"
      options:
        - id: regenerate
          label: "Regenerate"
          goto: choose-method
        - id: keep
          label: "Keep existing"
          goto: complete

  - id: choose-method
    type: options
    config:
      title: "Choose generation method"
      options:
        - id: default
          label: "Default Template (Recommended)"
          description: "Auto-detect and apply optimized rules"
          goto: detect-languages
        - id: qa
          label: "Q&A Workflow"
          description: "Answer questions for custom generation"
          goto: qa-start

  - id: detect-languages
    type: claude-code
    config:
      prompt_template: |
        Analyze the project structure and list detected languages.
        Return as JSON array: ["rust", "typescript", ...]
      output_variable: detected_languages
    transitions:
      on_success: apply-template
      on_error: handle-error

  - id: apply-template
    type: file-write
    config:
      path: .rstn/constitution.md
      template: default-constitution
      variables:
        languages: detected_languages
    transitions:
      on_success: complete

  - id: complete
    type: terminal
    config:
      status: success
      message: "Constitution setup complete!"

  - id: handle-error
    type: terminal
    config:
      status: error
      message: "Failed to complete workflow"
      allow_retry: true

entry_point: start
```

---

## Backend State Design

### WorkflowState (Rust)

```pseudo
struct WorkflowState {
    // Available workflows
    available_workflows: Vec<WorkflowDefinition>,

    // Currently active workflow instance
    active_workflow: Option<WorkflowInstance>,

    // Execution history (for debugging)
    execution_log: Vec<WorkflowLogEntry>,
}

struct WorkflowInstance {
    workflow_id: String,
    current_node_id: String,
    status: WorkflowStatus,  // Idle | Running | WaitingInput | Complete | Failed
    context: HashMap<String, Value>,  // Workflow variables
    started_at: DateTime,
    node_history: Vec<NodeExecution>,
}

struct NodeExecution {
    node_id: String,
    started_at: DateTime,
    completed_at: Option<DateTime>,
    status: NodeStatus,
    output: Option<Value>,
    error: Option<String>,
}

enum WorkflowStatus {
    Idle,
    Running,
    WaitingInput { node_id: String, input_schema: Value },
    Paused,
    Complete,
    Failed { error: String, recoverable: bool },
    Cancelled,
}
```

### Actions

```pseudo
enum WorkflowAction {
    // Workflow lifecycle
    StartWorkflow { workflow_id: String },
    CancelWorkflow,
    RetryWorkflow,

    // Node execution
    ExecuteNode { node_id: String },
    SkipNode { node_id: String },

    // User input
    SubmitNodeInput { node_id: String, input: Value },

    // Options selection
    SelectOption { node_id: String, option_id: String },

    // Internal (from backend)
    SetNodeStatus { node_id: String, status: NodeStatus },
    AppendNodeOutput { node_id: String, content: String },
    TransitionToNode { node_id: String },
    CompleteWorkflow { status: WorkflowStatus },
}
```

---

## UI Components

### WorkflowsPage Layout

```
┌─────────────────────────────────────────────────────────┐
│  Workflows                                              │
├───────────────────────┬─────────────────────────────────┤
│                       │                                 │
│  ┌─────────────────┐  │  ┌───────────────────────────┐  │
│  │ Constitution    │  │  │                           │  │
│  │ Setup        ▶  │  │  │   Workflow Execution      │  │
│  └─────────────────┘  │  │   Panel                   │  │
│                       │  │                           │  │
│  ┌─────────────────┐  │  │   - Current node status   │  │
│  │ Project         │  │  │   - Progress indicator    │  │
│  │ Bootstrap    ▶  │  │  │   - Streaming output      │  │
│  └─────────────────┘  │  │   - User input forms      │  │
│                       │  │   - Option selection      │  │
│  ┌─────────────────┐  │  │                           │  │
│  │ Migration       │  │  │                           │  │
│  │ Wizard       ▶  │  │  │                           │  │
│  └─────────────────┘  │  └───────────────────────────┘  │
│                       │                                 │
└───────────────────────┴─────────────────────────────────┘
     Workflow List              Execution Panel
```

### Workflow Execution Panel States

根據 `WorkflowStatus` 顯示不同內容：

| Status | Panel Content |
|--------|---------------|
| `Idle` | Workflow description + "Start" button |
| `Running` | Progress bar + current node + streaming output |
| `WaitingInput` | Input form based on node schema |
| `Complete` | Success message + summary |
| `Failed` | Error message + retry option |

### Progress Visualization

```
Node Progress Bar:
[✓ Check] → [✓ Detect] → [● Generate] → [○ Save] → [○ Verify]
                              ▲
                         Current Node
```

---

## Communication Protocol

### Events (Backend → Frontend)

```pseudo
// High-frequency: streaming output
Event::WorkflowNodeOutput {
    workflow_id: String,
    node_id: String,
    delta: String
}

// State changes
Event::WorkflowStateUpdate {
    workflow_id: String,
    status: WorkflowStatus,
    current_node: String,
    context: HashMap<String, Value>
}

// Node transitions
Event::WorkflowNodeTransition {
    workflow_id: String,
    from_node: String,
    to_node: String,
    reason: String
}
```

### Commands (Frontend → Backend)

```pseudo
Command::StartWorkflow { workflow_id: String }
Command::SubmitWorkflowInput { node_id: String, input: Value }
Command::SelectWorkflowOption { node_id: String, option_id: String }
Command::CancelWorkflow
Command::RetryWorkflow
```

---

## Migration from Current Constitution

### Current State (to be deprecated)

```
TasksState {
    constitution_workflow: ConstitutionWorkflow | null
    constitution_exists: boolean | null
}
```

### Target State

```
WorkflowState {
    active_workflow: WorkflowInstance | null  // Generic
    available_workflows: WorkflowDefinition[]
}
```

### Migration Steps

1. **Phase 1**: 新增 `WorkflowState` 並行運作
2. **Phase 2**: 將 Constitution 重構為 Workflow Definition
3. **Phase 3**: 移除舊的 `constitution_workflow` 相關程式碼
4. **Phase 4**: Tasks tab 只保留 Justfile commands

---

## Future Workflows

| Workflow | Description | Priority |
|----------|-------------|----------|
| `constitution-management` | Initialize project constitution | P0 (migrate existing) |
| `project-bootstrap` | Setup new project structure | P1 |
| `worktree-setup` | Configure new worktree | P2 |
| `migration-wizard` | Version upgrade assistance | P2 |
| `debug-session` | Guided debugging workflow | P3 |

---

## References

- `kb/architecture/09-workflow-prompt-claude.md` — Streaming & state machine patterns
- `kb/architecture/10-constitution-system.md` — Current constitution design
- `kb/architecture/00-overview.md` — Three pillars architecture

# Worktree View State Machine

**Created**: 2025-12-18
**Status**: Current Implementation (Before Redesign)
**Purpose**: Document how the state machine works today

---

## State Hierarchy

The WorktreeView has **3 levels** of state:

1. **Focus State** - Which pane has focus (Commands / Content / Output)
2. **Content State** - What's shown in content pane
3. **Sub-state** - Workflow-specific states (Specify/Implement/Commit)

---

## State Diagram: Full State Machine

```mermaid
stateDiagram-v2
    [*] --> Commands_Focus: App starts

    state "Focus Layer" as FocusLayer {
        Commands_Focus: Commands Pane
        Content_Focus: Content Pane
        Output_Focus: Output Pane

        Commands_Focus --> Content_Focus: → (Right arrow)
        Content_Focus --> Output_Focus: → (Right arrow)
        Output_Focus --> Commands_Focus: → (Right arrow, wraps)

        Commands_Focus --> Output_Focus: ← (Left arrow)
        Output_Focus --> Content_Focus: ← (Left arrow)
        Content_Focus --> Commands_Focus: ← (Left arrow, wraps)
    }

    state "Content Type Layer" as ContentLayer {
        Spec: Showing spec.md
        Plan: Showing plan.md
        Tasks: Showing tasks.md
        SpecifyInput: Input feature description
        SpecifyReview: Review generated spec
        CommitReview: Review commit groups
        TaskExecution: Execute tasks (Implement)
    }

    state "Command Execution" as CommandLayer {
        SDD_Specify: Specify Phase
        SDD_Clarify: Clarify Phase
        SDD_Plan: Plan Phase
        SDD_Tasks: Tasks Phase
        SDD_Implement: Implement Phase
        Git_Commit: Intelligent Commit
        Git_Other: Push/Status/AddAll
    }
```

---

## Enter Key State Transitions

### When Focus = Commands Pane

```mermaid
stateDiagram-v2
    [*] --> Commands_Pane: Focus on Commands

    Commands_Pane --> Check_Selected: Press Enter

    state Check_Selected <<choice>>
    Check_Selected --> SDD_Command: Selected SDD Phase
    Check_Selected --> Git_Command: Selected Git Action
    Check_Selected --> [*]: No selection / Header

    state "SDD Phase Selected" as SDD_Command {
        [*] --> Check_Phase: Determine phase

        state Check_Phase <<choice>>
        Check_Phase --> Specify_Flow: Specify
        Check_Phase --> Implement_Flow: Implement
        Check_Phase --> Interactive_Flow: Clarify/Plan/Tasks/etc.

        state "Specify Flow" as Specify_Flow {
            [*] --> SpecifyInput_State: start_specify_input()
            note right of SpecifyInput_State
                ContentType = SpecifyInput
                Show input textarea
                Focus on input
            end note

            SpecifyInput_State --> Generating: Enter (submit)
            note right of Generating
                Validate input (min 10 chars)
                Return GenerateSpec action
            end note

            Generating --> SpecifyReview_State: Generation complete
            note right of SpecifyReview_State
                ContentType = SpecifyReview
                Show generated spec
                Can edit or save
            end note

            SpecifyReview_State --> Edit_Mode: Press 'e'
            SpecifyReview_State --> Save: Press Enter
            SpecifyReview_State --> Cancel: Press Esc

            Edit_Mode --> SpecifyReview_State: Press Esc (save edits)
            Save --> [*]: Return SaveSpec action
            Cancel --> [*]: Reset to Spec view
        }

        state "Implement Flow" as Implement_Flow {
            [*] --> TaskList_State: start_implement_mode()
            note right of TaskList_State
                Load tasks from tasks.md
                Show task list
                Allow selection
            end note

            TaskList_State --> Execute_Task: Enter on task
            TaskList_State --> Cancel: Esc
            Execute_Task --> TaskList_State: Task complete
            Cancel --> [*]
        }

        state "Interactive Flow" as Interactive_Flow {
            [*] --> Input_State: start_interactive_phase()
            note right of Input_State
                ContentType = SpecifyInput
                current_phase = selected phase
                Show input for notes
            end note

            Input_State --> Generating_Phase: Enter (submit)
            Generating_Phase --> Review_State: Generation complete
            Review_State --> Save_Phase: Enter
            Review_State --> Cancel_Phase: Esc
            Save_Phase --> [*]
            Cancel_Phase --> [*]
        }
    }

    state "Git Action Selected" as Git_Command {
        [*] --> Check_Git_Type: Determine git command

        state Check_Git_Type <<choice>>
        Check_Git_Type --> Intelligent_Commit: Commit
        Check_Git_Type --> Run_Command: Push/Status/AddAll
        Check_Git_Type --> Request_Input: Rebase

        Intelligent_Commit --> [*]: Return RunIntelligentCommit
        note right of Intelligent_Commit
            Triggers security scan
            Groups changes
            Shows commit review
        end note

        Run_Command --> [*]: Return RunCommand{git, [args]}
        note right of Run_Command
            Directly execute git command
        end note

        Request_Input --> [*]: Return RequestInput
        note right of Request_Input
            Show input dialog
            Store pending_git_command
        end note
    }
```

---

## Detailed State: Specify Workflow

```mermaid
stateDiagram-v2
    [*] --> Normal: Idle

    Normal --> SpecifyInput: User selects Specify + Enter
    note right of Normal
        ContentType = Spec
        specify_state.input_buffer = ""
        is_generating = false
    end note

    state "Input Description" as SpecifyInput {
        [*] --> Typing: Cursor in textarea

        Typing --> Typing: Character input
        Typing --> Typing: Ctrl+Enter (newline)
        Typing --> Typing: Backspace/Delete
        Typing --> Typing: Arrow keys (move cursor)
        Typing --> Validate: Enter (submit)
        Typing --> Cancel: Esc

        Validate --> Generating: Valid (>= 10 chars)
        Validate --> Typing: Invalid (show error)

        Cancel --> [*]: Reset to Normal
    }

    state "Generating Spec" as Generating {
        [*] --> Spawning: Call Claude CLI
        note right of Spawning
            is_generating = true
            Show spinner
        end note

        Spawning --> Streaming: Process started
        Streaming --> Streaming: Stream chunks arrive
        Streaming --> Complete: Stream ends
        Streaming --> Error: Process fails

        Complete --> [*]: Generated spec ready
        Error --> [*]: Show error message
    }

    Generating --> SpecifyReview: Generation complete

    state "Review Generated Spec" as SpecifyReview {
        [*] --> Reading: Show generated content

        Reading --> Reading: Scroll up/down
        Reading --> EditMode: Press 'e'
        Reading --> Save: Press Enter
        Reading --> Cancel: Press Esc

        state "Edit Mode" as EditMode {
            [*] --> Editing: Multi-line editor
            Editing --> Editing: Character input
            Editing --> Editing: Enter (newline)
            Editing --> Editing: Arrow keys
            Editing --> [*]: Esc (save changes)
        }

        EditMode --> Reading: Exit edit mode

        Save --> [*]: Return SaveSpec action
        note right of Save
            Writes to spec.md
            Updates features.json
        end note

        Cancel --> [*]: Discard and reset
    }

    SpecifyReview --> Normal: Save or Cancel
```

---

## Detailed State: Commit Review Workflow

```mermaid
stateDiagram-v2
    [*] --> Trigger: User selects Commit + Enter

    Trigger --> Scanning: RunIntelligentCommit
    note right of Scanning
        Security scan (rstn_scan_security)
        Group changes (rstn_group_changes)
        Generate messages (rstn_generate_message)
    end note

    Scanning --> Review: Groups ready

    state "Commit Review" as Review {
        [*] --> ShowGroup: Display group N/M
        note right of ShowGroup
            ContentType = CommitReview
            commit_groups = vec![...]
            current_commit_index = 0
        end note

        ShowGroup --> ShowGroup: Edit message (type chars)
        ShowGroup --> ShowGroup: Backspace
        ShowGroup --> ShowGroup: Left/Right (move cursor)
        ShowGroup --> ShowGroup: j/k (prev/next group)

        ShowGroup --> Validate: Enter (submit)

        Validate --> NextGroup: Valid + more groups
        Validate --> Complete: Valid + last group
        Validate --> ShowGroup: Invalid (show error)

        NextGroup --> ShowGroup: current_commit_index++

        ShowGroup --> Cancel: Esc
        Cancel --> [*]: Reset commit workflow
    }

    Review --> Done: All groups committed
    Done --> [*]: Reset to normal
```

---

## Detailed State: Implement (Task Execution)

```mermaid
stateDiagram-v2
    [*] --> Load: User selects Implement + Enter

    Load --> TaskList: Parse tasks.md
    note right of TaskList
        task_list_state = parsed tasks
        Show task list with checkboxes
    end note

    state "Task List" as TaskList {
        [*] --> Selecting: Navigate tasks

        Selecting --> Selecting: j/k (move selection)
        Selecting --> Selecting: Shift+J/K (reorder)
        Selecting --> Selecting: 'a' (toggle auto-advance)
        Selecting --> Execute: Enter (execute selected)
        Selecting --> Cancel: Esc

        Execute --> Running: Spawn Claude CLI
        note right of Running
            Call ExecuteTask action
            task_id, description, feature
        end note

        Running --> Complete_Task: Task succeeds
        Running --> Error_Task: Task fails

        Complete_Task --> Next_Task: auto_advance = true
        Complete_Task --> Selecting: auto_advance = false

        Error_Task --> Selecting: Show error, stay on task

        Next_Task --> Execute: More tasks
        Next_Task --> Done: All tasks done

        Cancel --> [*]: Exit implement mode
        Done --> [*]: All tasks complete
    }
```

---

## Focus Navigation

```mermaid
stateDiagram-v2
    [*] --> Commands: Default focus

    Commands --> Content: → (Right arrow)
    Content --> Output: → (Right arrow)
    Output --> Commands: → (Right arrow, wraps)

    Commands --> Output: ← (Left arrow, reverse)
    Output --> Content: ← (Left arrow)
    Content --> Commands: ← (Left arrow, wraps)

    note right of Commands
        WorktreeFocus::Commands
        Left pane
        Command list (SDD + Git)
    end note

    note right of Content
        WorktreeFocus::Content
        Middle pane
        Shows content based on ContentType
    end note

    note right of Output
        WorktreeFocus::Output
        Right pane
        Logs and command output
    end note
```

---

## Key State Variables

### WorktreeView State Fields

```rust
pub struct WorktreeView {
    // Focus state
    pub focus: WorktreeFocus,  // Commands | Content | Output

    // Content state
    pub content_type: ContentType,  // Spec | Plan | Tasks | SpecifyInput | SpecifyReview | CommitReview

    // Specify workflow state
    pub specify_state: SpecifyState {
        input_buffer: String,           // User's input
        input_cursor: usize,            // Cursor position
        is_generating: bool,            // Is Claude generating?
        generated_spec: Option<String>, // Generated content
        current_phase: SpecPhase,       // Which phase (Specify/Clarify/etc.)
        validation_error: Option<String>,
        auto_advance: bool,             // Auto-execute next task?
        edit_text_input: Option<TextInput>, // Edit mode state
        task_list_state: Option<TaskListState>, // Implement mode
    },

    // Commit workflow state
    pub commit_groups: Option<Vec<CommitGroup>>,
    pub current_commit_index: usize,
    pub commit_message_input: String,
    pub commit_message_cursor: usize,
    pub commit_validation_error: Option<String>,

    // Command state
    pub commands: Vec<Command>,     // SDD phases + Git actions
    pub command_state: ListState,   // Selected index

    // Output state
    pub log_buffer: LogBuffer,
    pub is_running: bool,
    pub running_phase: Option<String>,
}
```

---

## State Transition Table

| Current State | Key Press | Condition | Next State | Action |
|--------------|-----------|-----------|------------|--------|
| Commands Focus | Enter | SDD: Specify selected | SpecifyInput | start_specify_input() |
| Commands Focus | Enter | SDD: Implement selected | TaskList | start_implement_mode() |
| Commands Focus | Enter | SDD: Other selected | Interactive Input | start_interactive_phase() |
| Commands Focus | Enter | Git: Commit selected | (no state change) | RunIntelligentCommit |
| Commands Focus | Enter | Git: Push selected | (no state change) | RunCommand |
| Commands Focus | Enter | Git: Rebase selected | (no state change) | RequestInput |
| Commands Focus | → | Any | Content Focus | focus_right() |
| Commands Focus | ← | Any | Output Focus | focus_left() |
| SpecifyInput | Enter | Valid input | Generating | Return GenerateSpec |
| SpecifyInput | Enter | Invalid input | SpecifyInput | Show validation error |
| SpecifyInput | Esc | Any | Normal | cancel_specify() |
| SpecifyReview | Enter | Any | Normal | Return SaveSpec |
| SpecifyReview | 'e' | Any | Edit Mode | toggle_specify_edit_mode() |
| SpecifyReview | Esc | Any | Normal | cancel_specify() |
| Edit Mode | Esc | Any | SpecifyReview | Save edits |
| TaskList | Enter | Task selected | Running | Return ExecuteTask |
| TaskList | j/k | Any | TaskList | Navigate tasks |
| TaskList | Esc | Any | Normal | cancel_specify() |
| CommitReview | Enter | Valid message | Next/Done | SubmitCommitGroup |
| CommitReview | Enter | Invalid message | CommitReview | Show error |
| CommitReview | Esc | Any | Normal | cancel_commit_review() |

---

## Problems with Current State Machine

### 1. **State Explosion**
- 54+ fields in WorktreeView to track all states
- Hard to understand which fields affect which states
- Risk of inconsistent state

### 2. **Nested Sub-states**
- SpecifyState has 8+ fields
- Commit workflow has 7+ fields
- Each workflow adds more fields

### 3. **Complex State Checking**
```rust
// Current code has many conditionals:
if self.content_type == ContentType::SpecifyInput && !self.specify_state.is_generating {
    // Handle input
} else if self.specify_state.edit_text_input.is_some() {
    // Handle edit mode
} else if self.specify_state.task_list_state.is_some() {
    // Handle task list
}
```

### 4. **Unclear State Ownership**
- Who can modify specify_state? (multiple methods)
- When is commit_groups set/cleared? (multiple places)
- Hard to trace state mutations

---

## Proposed Improvements (For Redesign)

### 1. **Explicit State Enum**
```rust
enum WorktreeState {
    Idle {
        focus: Focus,
        content: ViewContent,
    },
    PromptInput {
        buffer: String,
        cursor: usize,
    },
    CommandRunning {
        command: String,
        progress: f32,
    },
    ShowingResult {
        content: String,
    },
}
```

### 2. **State Machine Pattern**
```rust
impl WorktreeView {
    fn transition(&mut self, event: Event) -> Action {
        match (&self.state, event) {
            (State::Idle, Event::SelectCommand(cmd)) => {
                self.state = State::PromptInput::default();
                Action::None
            }
            (State::PromptInput, Event::Submit(text)) => {
                self.state = State::CommandRunning { ... };
                Action::RunCommand(text)
            }
            _ => Action::None
        }
    }
}
```

### 3. **Reduce Field Count**
- Target: <20 fields in WorktreeView
- Group related state into sub-structs
- Make state transitions explicit

---

## Related Documents

- [Worktree View Redesign](worktree-view-redesign.md) - New three-column design
- [Integration Flow](rstn-integration-flow.md) - rstn ↔ Claude ↔ MCP flow
- [Technical Debt](../03-complexity-analysis/technical-debt.md) - Current issues

---

## Changelog

- 2025-12-18: Initial state machine documentation created

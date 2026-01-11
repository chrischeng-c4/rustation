# Architecture Diagrams

## 1. State Diagram (Explorer Expansion)
```mermaid
stateDiagram-v2
    [*] --> Collapsed
    Collapsed --> Loading: ExpandDirectory Action
    Loading --> Expanded: Directory Read Success
    Loading --> Error: Directory Read Failure
    Error --> Collapsed: CollapseDirectory Action
    Expanded --> Collapsed: CollapseDirectory Action
    
    state Expanded {
        [*] --> Cached
        Cached --> Refreshing: ExpandDirectory (force)
        Refreshing --> Cached
    }
```

## 2. Flow Chart (Chat Submission)
```mermaid
flowchart TD
    A[User types message] -->|Click Send| B[Dispatch SubmitChatMessage]
    B --> C{Backend Reducer}
    C -->|1. Update State| D[Add User Message to History]
    C -->|2. Trigger Effect| E[Call AI Provider]
    D --> F[UI Re-renders (User Msg)]
    E -->|Stream Chunks| G[Dispatch UpdateChatMessage]
    G --> H[UI Re-renders (Streaming)]
    E -->|Complete| I[Dispatch CompleteChatMessage]
```

## 3. Sequence Diagram (Context Validation)
```mermaid
sequenceDiagram
    participant UI as ContextFilesInput
    participant Bridge as IPC Bridge
    participant Backend as Rust Reducer
    participant FS as File System

    UI->>Bridge: dispatch(ValidateContextFile, path)
    Bridge->>Backend: Action::ValidateContextFile
    Backend->>FS: read_metadata(path)
    alt Valid File
        FS-->>Backend: Metadata (size, type)
        Backend->>Backend: Update changes.validation_result = Valid
        Backend-->>UI: State Update (Valid)
    else Invalid
        FS-->>Backend: Error
        Backend->>Backend: Update changes.validation_result = Error
        Backend-->>UI: State Update (Error)
    end
```

## 4. UI Layout Diagram (Explorer)
```mermaid
graph TD
    subgraph Sidebar
        Tabs[Project Tabs]
        Explorer[File Explorer]
    end
    
    subgraph MainContent
        Editor[Source Code Viewer]
    end
    
    Explorer -->|Select File| Editor
    Explorer -->|Expand Folder| Explorer
```

```text
┌───────────────────┬──────────────────────────────────────────────┐
│  PROJECT TABS     │  src/main.rs                        [x]      │
├───────────────────┼──────────────────────────────────────────────┤
│ 📂 src            │                                              │
│ │  📂 features    │  fn main() {                                 │
│ │  │  📄 chat.rs  │      println!("Hello World");                │
│ │  └─ 📄 utils.rs │  }                                           │
│ 📄 Cargo.toml     │                                              │
│                   │                                              │
│                   │                                              │
└───────────────────┴──────────────────────────────────────────────┘
```

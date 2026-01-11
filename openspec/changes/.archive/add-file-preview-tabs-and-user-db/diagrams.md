# Architecture Diagrams

## 1. State Diagram (Tab Lifecycle)

```mermaid
stateDiagram-v2
    [*] --> NoTabs
    
    NoTabs --> PreviewTab: Single Click File
    
    PreviewTab --> PreviewTab: Single Click Other File (Replace)
    PreviewTab --> PinnedTab: Double Click File / Tab
    PreviewTab --> PinnedTab: Edit File (Implicit Pin)
    
    PinnedTab --> PinnedTab: Click Other (Switch Active)
    PinnedTab --> PreviewTab: Single Click New File
    
    PinnedTab --> NoTabs: Close Last Tab
    PreviewTab --> NoTabs: Close Tab
```

## 2. Flow Chart (File Selection Logic)

```mermaid
flowchart TD
    A[User Clicks File] --> B{Click Type?}
    B -- Single --> C{File Already Open?}
    B -- Double --> D[Mark Tab as Pinned]
    
    C -- Yes --> E[Set Tab Active]
    C -- No --> F{Has Preview Tab?}
    
    F -- Yes --> G[Replace Preview Tab Content]
    F -- No --> H[Create New Preview Tab]
    
    G --> I[Set Active]
    H --> I
    D --> I
    
    I --> J[Render DetailPanel]
```

## 3. Sequence Diagram (Database Access)

```mermaid
sequenceDiagram
    participant React as Frontend
    participant IPC as Bridge
    participant Backend as Rust Core
    participant DB as SQLite (~/.rstn/state.db)
    
    Note over Backend, DB: Initialization
    Backend->>DB: Open Connection
    Backend->>DB: Run Migrations (Create Tables + project_id)
    
    Note over React, DB: Add Comment
    React->>IPC: dispatch(AddFileComment)
    IPC->>Backend: AddFileComment(path, content)
    Backend->>Backend: Derive project_id (hash)
    Backend->>DB: INSERT INTO file_comments (..., project_id)
    DB-->>Backend: Success
    Backend-->>React: State Update (Comments)
```

## 4. UI Layout Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Sidebar   │  File Explorer                                              │
│            │                                                             │
│ ┌────────┐ │ ┌───────────────────────────────────────────────────────┐   │
│ │ Docs   │ │ │  _README.md_  │  src/main.rs  │  package.json  │      │   │
│ │        │ │ └───────┬───────────────────────────────────────────────┘   │
│ │ Src    │ │         │                                                   │
│ │ └─ main│ │  # README                                               │   │
│ │        │ │                                                         │   │
│ └────────┘ │  Welcome to Rustation...                                │   │
│            │                                                             │
│            │                                                             │
│            │                                                             │
│            │                                                             │
│            └───────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────┘
Legend:
- _Italic_: Preview Tab (Temporary)
- Normal: Pinned Tab (Persistent)
```

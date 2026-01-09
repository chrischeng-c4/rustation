# Architecture Diagrams

## 1. State Diagram (File Explorer)

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading: ExploreDir / Navigate
    Loading --> Browsing: Entries Loaded
    
    state Browsing {
        [*] --> List
        List --> Selected: Click File
        Selected --> Previewing: File Type Supported
        Selected --> Commenting: Click Comment Tab
    }

    Browsing --> Loading: Change Directory
```

## 2. Flow Chart (Add Comment)

```mermaid
flowchart TD
    A[User Types Comment] -->|Click Submit| B(Dispatch AddFileComment)
    B --> C{Backend Handler}
    C -->|Persist| D[(SQLite DB)]
    D -->|Success| E[Update AppState]
    E -->|Refresh| F[Reload Comments List]
    F --> G[Update UI]
```

## 3. Sequence Diagram (Session Restore)

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Backend
    participant DB

    User->>Frontend: Open App
    Frontend->>Backend: Init / Load State
    Backend->>DB: Read GlobalSettings
    DB-->>Backend: Return last_active_project_id
    Backend->>Backend: Load Project State
    Backend-->>Frontend: Push State (Active Project)
    Frontend-->>User: Show Last Project View
```

## 4. UI Layout Diagram

```
┌───────────────────────────────────────────────────────────────┐
│  Sidebar   │  Main Content Area                              │
│  (Nav)     │  (File Explorer)                                │
│            │                                                 │
│            │  ┌──────────────┐ ┌──────────────────────────┐  │
│            │  │  File Tree   │ │      File Preview        │  │
│            │  │              │ │                          │  │
│            │  │  src/        │ │  pub fn main() {         │  │
│            │  │  ├─ main.rs  │ │      println!("Hi");     │  │
│            │  │  └─ lib.rs   │ │  }                       │  │
│            │  │              │ │                          │  │
│            │  │              │ │  [Comments Tab]          │  │
│            │  │ (Fixed/Resize) │ (Flex Grow)              │  │
│            │  └──────────────┘ └──────────────────────────┘  │
│            │                                                 │
└────────────┴─────────────────────────────────────────────────┘
```

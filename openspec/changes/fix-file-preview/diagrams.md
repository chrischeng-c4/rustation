# Architecture Diagrams

## 1. State Diagram (Comment Lifecycle)

```mermaid
stateDiagram-v2
    [*] --> ViewingFile
    ViewingFile --> AddingInlineComment : Click Line Number
    AddingInlineComment --> SavingComment : Enter Text & Submit
    AddingInlineComment --> ViewingFile : Cancel
    SavingComment --> ViewingFile : Success
    
    state ViewingFile {
        [*] --> Idle
        Idle --> Scrolling : User Scrolls
        Idle --> SelectingLine : Hover Line
    }
```

## 2. Flow Chart (Render Logic)

```mermaid
flowchart TD
    A[Start Render] --> B{Has Content?}
    B -- No --> C[Show Loading/Empty]
    B -- Yes --> D[Split Lines]
    D --> E[Loop Lines]
    E --> F{Has Comments?}
    F -- Yes --> G[Render Line + Comment Block]
    F -- No --> H[Render Line Only]
    G --> I[Next Line]
    H --> I
    I --> J{End of File?}
    J -- No --> E
    J -- Yes --> K[End Render]
```

## 3. Sequence Diagram (Add Comment)

```mermaid
sequenceDiagram
    participant User
    participant UI as DetailPanel/SourceViewer
    participant Rust as Backend (Action)
    participant DB as SQLite
    
    User->>UI: Click Line Number (10)
    UI->>User: Show Comment Input
    User->>UI: Type "Fix this" & Submit
    UI->>Rust: dispatch(AddFileComment { path, line: 10, content })
    Rust->>DB: INSERT into file_comments
    DB-->>Rust: Success (ID)
    Rust->>UI: State Update (New Comment)
    UI->>User: Show Comment Inline
```

## 4. UI Layout Diagram

```
┌───────────────────────────────────────────────────────────────┐
│ File Explorer                                                 │
├──────────────────────┬────────────────────────────────────────┤
│ ┌──────────────────┐ │ ┌────────────────────────────────────┐ │
│ │ File List        │ │ │ Detail Panel                       │ │
│ │ 📄 main.rs       │ │ │ ┌────────────────────────────────┐ │ │
│ │ 📄 lib.rs        │ │ │ │ Info | Preview | Comments      │ │ │
│ │                  │ │ │ ├────────────────────────────────┤ │ │
│ │                  │ │ │ │ 1  fn main() {                 │ │ │
│ │                  │ │ │ │ 2      println!("Hello");      │ │ │
│ │                  │ │ │ │    ┌──────────────────────┐    │ │ │
│ │                  │ │ │ │    │ 💬 User: Nice code!  │    │ │ │
│ │                  │ │ │ │    └──────────────────────┘    │ │ │
│ │                  │ │ │ │ 3  }                           │ │ │
│ │                  │ │ │ │                                │ │ │
│ │                  │ │ │ │                                │ │ │
│ │                  │ │ │ └────────────────────────────────┘ │ │
│ │                  │ │ └────────────────────────────────────┘ │
│ └──────────────────┘ └────────────────────────────────────────┘
└───────────────────────────────────────────────────────────────┘
```

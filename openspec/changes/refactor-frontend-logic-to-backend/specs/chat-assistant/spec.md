## ADDED Requirements
### Requirement: Chat Interaction
The system SHALL provide an AI chat interface grounded in the project context.

#### Scenario: Submit message
- **WHEN** user sends a message
- **THEN** dispatch `SubmitChatMessage` action
- **AND** backend adds message to history with "user" role
- **AND** backend triggers AI response generation

#### Scenario: AI Response
- **WHEN** AI response is received
- **THEN** backend streams chunks via `UpdateChatMessage`
- **AND** backend finalizes message via `CompleteChatMessage`

### Requirement: Chat History
The system SHALL persist chat history per project.

#### Scenario: Restore history
- **WHEN** project is opened
- **THEN** load previous chat messages from persisted state

#### Scenario: Clear history
- **WHEN** user clears chat
- **THEN** remove all messages from state and persistence

use crate::actions::{Action, ChatRoleData};
use crate::app_state::AppState;
use uuid::Uuid;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::SendChatMessage { text } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.is_typing = true;
                    worktree.chat.error = None;

                    // Add user message
                    let user_msg = crate::app_state::ChatMessage {
                        id: Uuid::new_v4().to_string(),
                        role: crate::app_state::ChatRole::User,
                        content: text,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        is_streaming: false,
                    };
                    worktree.chat.add_message(user_msg);
                }
            }
        }

        Action::AddChatMessage { message } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let chat_message = crate::app_state::ChatMessage {
                        id: message.id,
                        role: match message.role {
                            ChatRoleData::User => crate::app_state::ChatRole::User,
                            ChatRoleData::Assistant => crate::app_state::ChatRole::Assistant,
                            ChatRoleData::System => crate::app_state::ChatRole::System,
                        },
                        content: message.content,
                        timestamp: message.timestamp,
                        is_streaming: message.is_streaming,
                    };
                    worktree.chat.add_message(chat_message);
                }
            }
        }

        Action::AppendChatContent { content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.append_to_last(&content);
                }
            }
        }

        Action::SetChatTyping { is_typing } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.is_typing = is_typing;
                    if !is_typing {
                        worktree.chat.finish_streaming();
                    }
                }
            }
        }

        Action::SetChatError { error } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.error = Some(error);
                    worktree.chat.is_typing = false;
                }
            }
        }

        Action::ClearChatError => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.error = None;
                }
            }
        }

        Action::ClearChat => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.clear();
                }
            }
        }
        _ => {}
    }
}

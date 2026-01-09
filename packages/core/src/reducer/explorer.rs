use crate::actions::Action;
use crate::app_state::AppState;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::ExploreDir { .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.is_loading = true;
                }
            }
        }

        Action::SetExplorerEntries { path, entries } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.current_path = path;
                    worktree.explorer.entries = entries.into_iter().map(|e| e.into()).collect();
                    worktree.explorer.is_loading = false;
                }
            }
        }

        Action::SetFileComments { comments, .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.selected_comments = comments.into_iter().map(|c| c.into()).collect();
                }
            }
        }

        Action::NavigateBack => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(path) = worktree.explorer.history.back_stack.pop() {
                        worktree.explorer.history.forward_stack.push(worktree.explorer.current_path.clone());
                        worktree.explorer.current_path = path;
                        worktree.explorer.is_loading = true;
                    }
                }
            }
        }

        Action::NavigateForward => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(path) = worktree.explorer.history.forward_stack.pop() {
                        worktree.explorer.history.back_stack.push(worktree.explorer.current_path.clone());
                        worktree.explorer.current_path = path;
                        worktree.explorer.is_loading = true;
                    }
                }
            }
        }

        Action::NavigateUp => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let current = std::path::Path::new(&worktree.explorer.current_path);
                    if let Some(parent) = current.parent() {
                        let parent_path = parent.to_string_lossy().to_string();
                        worktree.explorer.history.back_stack.push(worktree.explorer.current_path.clone());
                        worktree.explorer.current_path = parent_path;
                        worktree.explorer.is_loading = true;
                    }
                }
            }
        }

        Action::SelectFile { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.selected_path = path;
                    worktree.explorer.selected_comments.clear();
                }
            }
        }

        Action::SetExplorerSort { field, direction } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.sort_config.field = field.into();
                    worktree.explorer.sort_config.direction = direction.into();
                }
            }
        }

        Action::SetExplorerFilter { query } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.filter_query = query;
                }
            }
        }
        _ => {}
    }
}

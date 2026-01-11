use crate::actions::Action;
use crate::app_state::AppState;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::ExploreDir { ref path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    // Only push to history if navigating to a different path
                    if !worktree.explorer.current_path.is_empty() && worktree.explorer.current_path != *path {
                        worktree.explorer.history.back_stack.push(worktree.explorer.current_path.clone());
                        // Clear forward stack when navigating to a new path
                        worktree.explorer.history.forward_stack.clear();
                    }
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
            eprintln!("[Reducer] SetFileComments: Received {} comments", comments.len());
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.selected_comments = comments.into_iter().map(|c| c.into()).collect();
                    eprintln!("[Reducer] SetFileComments: Updated selected_comments to {} items", worktree.explorer.selected_comments.len());
                } else {
                    eprintln!("[Reducer] SetFileComments: No active worktree!");
                }
            } else {
                eprintln!("[Reducer] SetFileComments: No active project!");
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

        // ========================================================================
        // Tab Management Actions (VSCode-style preview tabs)
        // ========================================================================
        Action::OpenFileTab { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let explorer = &mut worktree.explorer;

                    // Check if this file is already open
                    if let Some(existing_idx) = explorer.tabs.iter().position(|t| t.path == path) {
                        // Already open, just activate it
                        explorer.active_tab_path = Some(path);
                        // If it was a preview tab, keep it as preview (user can double-click to pin)
                        let _ = existing_idx; // silence unused warning
                    } else {
                        // Find and replace existing preview tab (unpinned tab)
                        if let Some(preview_idx) = explorer.tabs.iter().position(|t| !t.is_pinned) {
                            // Replace the preview tab
                            explorer.tabs[preview_idx] = crate::app_state::FileTab {
                                path: path.clone(),
                                is_pinned: false,
                            };
                        } else {
                            // No preview tab exists, add new preview tab
                            explorer.tabs.push(crate::app_state::FileTab {
                                path: path.clone(),
                                is_pinned: false,
                            });
                        }
                        explorer.active_tab_path = Some(path);
                    }
                }
            }
        }

        Action::PinTab { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    // Find the tab and pin it
                    if let Some(tab) = worktree.explorer.tabs.iter_mut().find(|t| t.path == path) {
                        tab.is_pinned = true;
                    }
                }
            }
        }

        Action::CloseTab { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let explorer = &mut worktree.explorer;

                    if let Some(idx) = explorer.tabs.iter().position(|t| t.path == path) {
                        explorer.tabs.remove(idx);

                        // If we closed the active tab, select another
                        if explorer.active_tab_path.as_ref() == Some(&path) {
                            if explorer.tabs.is_empty() {
                                explorer.active_tab_path = None;
                            } else {
                                // Select the tab at the same index, or the last one if index is out of bounds
                                let new_idx = idx.min(explorer.tabs.len() - 1);
                                explorer.active_tab_path = Some(explorer.tabs[new_idx].path.clone());
                            }
                        }
                    }
                }
            }
        }

        Action::SwitchTab { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    // Only switch if the tab exists
                    if worktree.explorer.tabs.iter().any(|t| t.path == path) {
                        worktree.explorer.active_tab_path = Some(path);
                    }
                }
            }
        }

        Action::ExpandDirectory { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.expanded_paths.insert(path.clone());
                    if !worktree.explorer.directory_cache.contains_key(&path) {
                        worktree.explorer.loading_paths.insert(path);
                    }
                }
            }
        }

        Action::CollapseDirectory { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.expanded_paths.remove(&path);
                }
            }
        }

        Action::SetDirectoryCache { path, entries } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.explorer.directory_cache.insert(
                        path.clone(), 
                        entries.into_iter().map(|e| e.into()).collect()
                    );
                    worktree.explorer.loading_paths.remove(&path);
                }
            }
        }

        Action::CreateFile { .. }
        | Action::RenameFile { .. }
        | Action::DeleteFile { .. }
        | Action::RevealInOS { .. }
        | Action::AddFileComment { .. }
        | Action::DeleteFileComment { .. } => {}

        _ => {}
    }
}
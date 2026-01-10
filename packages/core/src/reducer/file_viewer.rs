use crate::actions::Action;
use crate::app_state::AppState;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::ReadFile { path } => {
            state.file_viewer.path = Some(path);
            state.file_viewer.is_loading = true;
            state.file_viewer.content = None;
            state.file_viewer.error = None;
        }

        Action::SetFileContent { path, content, error } => {
            state.file_viewer.path = Some(path);
            state.file_viewer.content = content;
            state.file_viewer.error = error;
            state.file_viewer.is_loading = false;
        }

        Action::SetFileLoading { is_loading } => {
            state.file_viewer.is_loading = is_loading;
        }

        Action::ReadBinaryFile { path } => {
            state.file_viewer.path = Some(path);
            state.file_viewer.is_loading = true;
            state.file_viewer.binary_content = None;
            state.file_viewer.content = None;
            state.file_viewer.error = None;
        }

        Action::SetBinaryFileContent { path, content, error } => {
            state.file_viewer.path = Some(path);
            state.file_viewer.binary_content = content;
            state.file_viewer.content = None; // Clear text content
            state.file_viewer.error = error;
            state.file_viewer.is_loading = false;
        }

        _ => {}
    }
}

#[cfg(test)]
mod redirection_tests {
    use rush::executor::execute::CommandExecutor;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    #[test]
    fn test_output_redirection_permission_denied() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_readonly.txt";

        // Create a read-only file
        fs::write(test_file, "initial").unwrap();
        let metadata = fs::metadata(test_file).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444); // Read-only
        fs::set_permissions(test_file, permissions).unwrap();

        // Try to redirect output to read-only file (should fail)
        let result = executor.execute("echo test > /tmp/rush_readonly.txt");
        assert!(result.is_err() || result.unwrap() != 0);

        // Clean up - restore write permission first
        let metadata = fs::metadata(test_file).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(test_file, permissions).unwrap();
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_output_redirection_to_directory() {
        let mut executor = CommandExecutor::new();

        // Try to redirect output to a directory (should fail)
        let result = executor.execute("echo test > /tmp");
        assert!(result.is_err() || result.unwrap() != 0);
    }

    #[test]
    fn test_input_redirection_file_not_found() {
        let mut executor = CommandExecutor::new();

        // Try to redirect input from non-existent file (should fail)
        let result = executor.execute("cat < /tmp/nonexistent_file_xyz123.txt");
        assert!(result.is_err() || result.unwrap() != 0);
    }

    #[test]
    fn test_append_redirection_creates_file() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_append_new.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Append to non-existent file (should create it)
        let result = executor.execute("echo test >> /tmp/rush_append_new.txt");
        assert!(result.is_ok());
        assert!(Path::new(test_file).exists());

        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("test"));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_multiple_redirections_last_wins() {
        let mut executor = CommandExecutor::new();
        let file1 = "/tmp/rush_multi1.txt";
        let file2 = "/tmp/rush_multi2.txt";

        // Clean up first
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);

        // Multiple output redirections - last one wins
        let result = executor.execute("echo test > /tmp/rush_multi1.txt > /tmp/rush_multi2.txt");
        assert!(result.is_ok());

        // Only file2 should have content
        assert!(!Path::new(file1).exists() || fs::read_to_string(file1).unwrap().is_empty());
        assert!(Path::new(file2).exists());
        let content = fs::read_to_string(file2).unwrap();
        assert!(content.contains("test"));

        // Clean up
        let _ = fs::remove_file(file1);
        fs::remove_file(file2).unwrap();
    }

    #[test]
    fn test_redirection_with_pipeline() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_pipe_redir.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Pipeline with redirection at the end
        let result = executor.execute("echo hello world | grep world > /tmp/rush_pipe_redir.txt");
        assert!(result.is_ok());
        assert!(Path::new(test_file).exists());

        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("world"));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_input_and_output_redirection_combined() {
        let mut executor = CommandExecutor::new();
        let input_file = "/tmp/rush_combo_in.txt";
        let output_file = "/tmp/rush_combo_out.txt";

        // Clean up first
        let _ = fs::remove_file(input_file);
        let _ = fs::remove_file(output_file);

        // Create input file
        fs::write(input_file, "test content\n").unwrap();

        // Use both input and output redirection
        let result = executor.execute("cat < /tmp/rush_combo_in.txt > /tmp/rush_combo_out.txt");
        assert!(result.is_ok());
        assert!(Path::new(output_file).exists());

        let content = fs::read_to_string(output_file).unwrap();
        assert!(content.contains("test content"));

        // Clean up
        fs::remove_file(input_file).unwrap();
        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_redirection_to_dev_null() {
        let mut executor = CommandExecutor::new();

        // Redirect to /dev/null (should work without error)
        let result = executor.execute("echo test > /dev/null");
        assert!(result.is_ok());
    }

    #[test]
    fn test_append_redirection_permission_denied() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_append_readonly.txt";

        // Create a read-only file
        fs::write(test_file, "initial").unwrap();
        let metadata = fs::metadata(test_file).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444); // Read-only
        fs::set_permissions(test_file, permissions).unwrap();

        // Try to append to read-only file (should fail)
        let result = executor.execute("echo test >> /tmp/rush_append_readonly.txt");
        assert!(result.is_err() || result.unwrap() != 0);

        // Clean up - restore write permission first
        let metadata = fs::metadata(test_file).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(test_file, permissions).unwrap();
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_append_redirection_to_directory() {
        let mut executor = CommandExecutor::new();

        // Try to append to a directory (should fail)
        let result = executor.execute("echo test >> /tmp");
        assert!(result.is_err() || result.unwrap() != 0);
    }

    #[test]
    fn test_input_redirection_from_directory() {
        let mut executor = CommandExecutor::new();

        // Try to read from a directory (should fail)
        let result = executor.execute("cat < /tmp");
        assert!(result.is_err() || result.unwrap() != 0);
    }
}

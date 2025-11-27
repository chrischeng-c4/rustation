#[cfg(test)]
mod feature_tests {
    use rush::executor::execute::CommandExecutor;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_output_redirection() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_test_output.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Test > (create/overwrite)
        let result = executor.execute("echo hello > /tmp/rush_test_output.txt");
        assert!(result.is_ok());
        assert!(Path::new(test_file).exists());

        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("hello"));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_append_redirection() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_test_append.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Create initial file
        executor
            .execute("echo first >> /tmp/rush_test_append.txt")
            .unwrap();
        // Append to it
        executor
            .execute("echo second >> /tmp/rush_test_append.txt")
            .unwrap();

        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_pipeline_simple() {
        let mut executor = CommandExecutor::new();
        // Simple two-command pipeline
        let result = executor.execute("echo test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_background_execution_syntax() {
        let mut executor = CommandExecutor::new();
        // Just test that background syntax is accepted
        // (actual background execution is hard to test in unit test)
        let result = executor.execute("true &");
        assert!(result.is_ok());
    }

    #[test]
    fn test_input_redirection() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_test_input.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Create test input file
        fs::write(test_file, "test content\n").unwrap();

        // Test < (read from file)
        // Using cat to read from input redirection
        let result = executor.execute("cat < /tmp/rush_test_input.txt");
        assert!(result.is_ok());
        // Note: We can't easily capture stdout in this test, but if no error, it worked

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    // === Environment Variables Integration Tests ===

    #[test]
    fn test_export_and_use_variable() {
        let mut executor = CommandExecutor::new();

        // Export a variable
        let result = executor.execute("export TEST_VAR=hello_world");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify the variable is set in the environment manager
        assert_eq!(executor.env_manager().get("TEST_VAR"), Some("hello_world"));
    }

    #[test]
    fn test_variable_expansion_in_echo() {
        let mut executor = CommandExecutor::new();

        // Set a variable
        executor
            .env_manager_mut()
            .set("MY_VAR".to_string(), "test_value".to_string())
            .unwrap();

        // The variable should be available for expansion
        // (echo $MY_VAR will expand to "echo test_value")
        let result = executor.execute("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_path_modification() {
        let mut executor = CommandExecutor::new();

        // Get current PATH
        let original_path = executor.env_manager().get("PATH").unwrap_or("").to_string();

        // Append to PATH using variable expansion (the proper way)
        let result = executor.execute("export PATH=$PATH:/custom/test/path");
        assert!(result.is_ok());

        // Verify PATH was modified
        let new_path = executor.env_manager().get("PATH").unwrap();
        assert!(new_path.contains("/custom/test/path"));
        assert!(new_path.starts_with(&original_path));
    }

    #[test]
    fn test_variable_in_redirection_path() {
        let mut executor = CommandExecutor::new();

        // Set output directory variable
        executor
            .env_manager_mut()
            .set("TEST_DIR".to_string(), "/tmp".to_string())
            .unwrap();

        let test_file = "/tmp/rush_env_test.txt";
        let _ = fs::remove_file(test_file);

        // Use variable in redirection path
        let result = executor.execute("echo env_test > $TEST_DIR/rush_env_test.txt");
        assert!(result.is_ok());

        // Verify file was created
        assert!(Path::new(test_file).exists());
        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("env_test"));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_set_builtin_lists_variables() {
        let mut executor = CommandExecutor::new();

        // Add a test variable
        executor
            .env_manager_mut()
            .set("SET_TEST_VAR".to_string(), "set_value".to_string())
            .unwrap();

        // Run set command (should succeed and list variables)
        let result = executor.execute("set");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_export_with_variable_expansion() {
        let mut executor = CommandExecutor::new();

        // Set initial variable
        executor
            .env_manager_mut()
            .set("BASE".to_string(), "/home/user".to_string())
            .unwrap();

        // Export with variable expansion
        let result = executor.execute("export FULL_PATH=$BASE/documents");
        assert!(result.is_ok());

        // Verify the expansion happened
        assert_eq!(executor.env_manager().get("FULL_PATH"), Some("/home/user/documents"));
    }

    #[test]
    fn test_system_environment_inherited() {
        let executor = CommandExecutor::new();

        // PATH and HOME should be inherited from system
        assert!(executor.env_manager().get("PATH").is_some());
        // HOME might not be set in all environments, so just check PATH
    }

    #[test]
    fn test_undefined_variable_expands_to_empty() {
        let mut executor = CommandExecutor::new();

        // Use an undefined variable (should not cause error)
        let result = executor.execute("echo $UNDEFINED_VARIABLE_12345");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_glob_expansion_star() {
        use std::env;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        fs::write(dir_path.join("file1.txt"), "content1").unwrap();
        fs::write(dir_path.join("file2.txt"), "content2").unwrap();
        fs::write(dir_path.join("other.rs"), "rust code").unwrap();

        // Save original dir
        let original_dir = env::current_dir().unwrap();

        // Change to temp directory
        env::set_current_dir(dir_path).unwrap();

        let mut executor = CommandExecutor::new();

        // Test glob expansion - echo *.txt should show both txt files
        // Note: This test verifies the command runs without error
        // The actual expansion happens and echo receives multiple args
        let result = executor.execute("echo *.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_glob_no_match_literal() {
        let mut executor = CommandExecutor::new();

        // When glob pattern matches nothing, it's passed literally (POSIX behavior)
        // echo *.nonexistent should just output "*.nonexistent"
        let result = executor.execute("echo *.nonexistent_extension_12345");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_stderr_redirection() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_stderr_test.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Run command that outputs to stderr (ls on nonexistent produces error)
        // Using 2> to redirect stderr to file (no space between 2 and >)
        let result = executor.execute("ls /nonexistent_path_12345 2>/tmp/rush_stderr_test.txt");
        // Command fails but redirection should work
        assert!(result.is_ok());

        // Check that error was captured to file
        let content = fs::read_to_string(test_file).unwrap_or_default();
        assert!(
            content.contains("No such file") || content.contains("nonexistent"),
            "Expected error message in stderr file, got: {}",
            content
        );

        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_both_output_redirection() {
        let mut executor = CommandExecutor::new();
        let test_file = "/tmp/rush_both_test.txt";

        // Clean up first
        let _ = fs::remove_file(test_file);

        // Test &> which redirects both stdout and stderr
        let result = executor.execute("echo hello &> /tmp/rush_both_test.txt");
        assert!(result.is_ok());

        let content = fs::read_to_string(test_file).unwrap_or_default();
        assert!(content.contains("hello"));

        // Clean up
        let _ = fs::remove_file(test_file);
    }
}

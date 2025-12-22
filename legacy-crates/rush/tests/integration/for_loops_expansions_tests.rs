//! Integration tests for Phase 2: For loop expansions
//!
//! Tests variable expansion, command substitution, and globbing in for loops

#[cfg(test)]
mod for_loop_expansions {
    use rush::executor::execute::CommandExecutor;
    use std::sync::{Arc, Mutex};

    fn create_executor() -> CommandExecutor {
        CommandExecutor::new()
    }

    #[test]
    fn test_for_loop_with_variable_expansion() {
        let mut executor = create_executor();

        // Set a variable with multiple words
        executor
            .variable_manager_mut()
            .set("items".to_string(), "apple banana cherry".to_string())
            .unwrap();

        // Execute: for fruit in $items; do echo $fruit; done
        let result = executor.execute("for fruit in $items; do echo $fruit; done");

        // Should succeed
        assert!(result.is_ok(), "For loop with variable expansion should succeed");
    }

    #[test]
    fn test_for_loop_with_mixed_variables() {
        let mut executor = create_executor();

        executor
            .variable_manager_mut()
            .set("first".to_string(), "a".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("second".to_string(), "b c".to_string())
            .unwrap();

        // Execute: for x in $first $second d; do true; done
        let result = executor.execute("for x in $first $second d; do true; done");

        // Should succeed - should iterate over: a, b, c, d
        assert!(result.is_ok(), "For loop with mixed variables should succeed");
    }

    #[test]
    fn test_for_loop_with_empty_variable() {
        let mut executor = create_executor();

        // Set an empty variable
        executor
            .variable_manager_mut()
            .set("empty".to_string(), "".to_string())
            .unwrap();

        // Execute: for x in $empty other; do echo $x; done
        let result = executor.execute("for x in $empty other; do echo $x; done");

        // Should succeed - should iterate over just "other"
        assert!(result.is_ok(), "For loop with empty variable should succeed");
    }

    #[test]
    fn test_for_loop_variable_binding() {
        let mut executor = create_executor();

        // Execute: for i in 1 2 3; do true; done
        let result = executor.execute("for i in 1 2 3; do true; done");
        assert!(result.is_ok());

        // After loop, variable should retain last value
        let i_value = executor.variable_manager().get("i");
        assert_eq!(i_value, Some("3"), "Loop variable should retain last value");
    }

    #[test]
    fn test_for_loop_with_special_variables() {
        let mut executor = create_executor();

        // Set $$ (process ID) and $? (exit code) shouldn't break loop
        // Execute: for x in a b; do true; done
        let result = executor.execute("for x in a b; do true; done");
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_nested_variable_reference() {
        let mut executor = create_executor();

        executor
            .variable_manager_mut()
            .set("dir".to_string(), "/tmp".to_string())
            .unwrap();

        // Execute: for x in a b; do echo $x; done
        // (This tests that we can use loop variable in body)
        let result = executor.execute("for x in one two; do echo $x; done");
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_variable_shadowing() {
        let mut executor = create_executor();

        // Set initial value
        executor
            .variable_manager_mut()
            .set("x".to_string(), "original".to_string())
            .unwrap();

        // Execute: for x in new1 new2; do true; done
        executor
            .execute("for x in new1 new2; do true; done")
            .unwrap();

        // After loop, x should have last value from loop
        let x_value = executor.variable_manager().get("x");
        assert_eq!(x_value, Some("new2"), "Loop variable should shadow original value");
    }

    #[test]
    fn test_for_loop_with_braced_variable() {
        let mut executor = create_executor();

        executor
            .variable_manager_mut()
            .set("items".to_string(), "x y z".to_string())
            .unwrap();

        // Execute: for i in ${items}; do true; done
        let result = executor.execute("for i in ${items}; do true; done");
        assert!(result.is_ok(), "For loop with braced variable should succeed");
    }

    #[test]
    fn test_for_loop_exit_code_propagation() {
        let mut executor = create_executor();

        // Execute: for x in a b c; do true; done
        let result = executor.execute("for x in a b c; do true; done");
        assert!(result.is_ok(), "Loop with true should succeed");
        assert_eq!(result.unwrap(), 0, "Loop with true should exit with code 0");

        // Execute: for x in a b c; do false; done
        let result = executor.execute("for x in a b c; do false; done");
        assert!(result.is_ok(), "Loop with false should execute");
        assert_eq!(result.unwrap(), 1, "Loop with false should exit with code 1");
    }
}

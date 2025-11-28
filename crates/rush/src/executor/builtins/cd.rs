//! Implementation of the `cd` builtin command
//!
//! The `cd` builtin changes the shell's current working directory.
//!
//! Usage:
//! - `cd` - Change to home directory
//! - `cd <path>` - Change to specified directory
//! - `cd ~` - Change to home directory
//! - `cd -` - Change to previous directory

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use std::env;
use std::path::PathBuf;

/// Execute the `cd` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used for cd, but required for builtin interface)
/// * `args` - Command arguments (directory path)
///
/// # Returns
/// * `Ok(0)` - Successfully changed directory
/// * `Ok(1)` - Failed to change directory
pub fn execute(_executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Validate arguments
    if args.len() > 1 {
        eprintln!("rush: cd: too many arguments");
        return Ok(1);
    }

    // Determine target directory
    let target = match args.first() {
        None => get_home_directory()?,                 // cd with no args
        Some(path) if path == "-" => return cd_dash(), // cd -
        Some(path) if path.starts_with('~') => expand_tilde(path)?, // cd ~
        Some(path) => PathBuf::from(path),             // cd <path>
    };

    // Change directory
    change_directory(&target)
}

/// Get the home directory from $HOME environment variable
fn get_home_directory() -> Result<PathBuf> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| RushError::Execution("cd: HOME not set".to_string()))
}

/// Get the previous directory from $OLDPWD environment variable
fn get_oldpwd() -> Result<PathBuf> {
    env::var("OLDPWD")
        .map(PathBuf::from)
        .map_err(|_| RushError::Execution("cd: OLDPWD not set".to_string()))
}

/// Expand tilde (~) to home directory
///
/// Supports:
/// - `~` -> $HOME
/// - `~/path` -> $HOME/path
fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        get_home_directory()
    } else if path.starts_with("~/") {
        // ~/rest -> $HOME/rest
        let home = get_home_directory()?;
        Ok(home.join(&path[2..])) // Skip "~/"
    } else {
        // Just "~something" - treat as literal path
        Ok(PathBuf::from(path))
    }
}

/// Handle cd - (change to previous directory)
fn cd_dash() -> Result<i32> {
    let oldpwd = get_oldpwd()?;

    // Print the new directory (bash behavior)
    println!("{}", oldpwd.display());

    // Change to it
    change_directory(&oldpwd)
}

/// Change to the specified directory
///
/// Updates PWD and OLDPWD environment variables on success.
fn change_directory(target: &PathBuf) -> Result<i32> {
    // Save current directory as OLDPWD
    if let Ok(current) = env::current_dir() {
        env::set_var("OLDPWD", current.to_string_lossy().as_ref());
    }

    // Try to change directory
    match env::set_current_dir(target) {
        Ok(_) => {
            // Update PWD to new directory
            if let Ok(new_pwd) = env::current_dir() {
                env::set_var("PWD", new_pwd.to_string_lossy().as_ref());
            }
            Ok(0)
        }
        Err(e) => {
            // Print error message matching bash format
            eprintln!("rush: cd: {}: {}", target.display(), e);
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;
    use std::env;

    /// Helper to save and restore directory after test
    struct DirGuard {
        original: PathBuf,
    }

    impl DirGuard {
        fn new() -> Self {
            Self { original: env::current_dir().unwrap() }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.original).ok();
        }
    }

    #[test]
    fn test_cd_to_tmp() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        let args = vec!["/tmp".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify we're in /tmp (or /private/tmp on macOS)
        let current = env::current_dir().unwrap();
        let current_str = current.to_string_lossy();
        assert!(
            current_str.ends_with("tmp"),
            "Expected to be in tmp directory, got: {}",
            current_str
        );
    }

    #[test]
    fn test_cd_parent_directory() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // First cd to /tmp
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();
        let before = env::current_dir().unwrap();

        // Then cd ..
        let result = execute(&mut executor, &vec!["..".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Should be in parent directory
        let current = env::current_dir().unwrap();
        assert_ne!(current, before, "Should have moved to parent directory");
        assert!(before.starts_with(&current), "Before should start with current parent");
    }

    #[test]
    fn test_cd_home_no_args() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // First cd to /tmp to ensure we're not already home
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();

        // cd with no arguments should go to HOME
        let result = execute(&mut executor, &vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify we're in home directory (canonicalize both paths to compare)
        let current = env::current_dir().unwrap().canonicalize().unwrap();
        let home = PathBuf::from(env::var("HOME").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(current, home);
    }

    #[test]
    fn test_cd_tilde() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // First cd to /tmp to ensure we're not already home
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();

        // cd ~ should go to HOME
        let result = execute(&mut executor, &vec!["~".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify we're in home directory (canonicalize both paths to compare)
        let current = env::current_dir().unwrap().canonicalize().unwrap();
        let home = PathBuf::from(env::var("HOME").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(current, home);
    }

    #[test]
    fn test_cd_too_many_arguments() {
        let mut executor = CommandExecutor::new();

        let args = vec!["/tmp".to_string(), "/var".to_string()];
        let result = execute(&mut executor, &args);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_cd_nonexistent_directory() {
        let mut executor = CommandExecutor::new();

        let args = vec!["/this/path/definitely/does/not/exist/12345".to_string()];
        let result = execute(&mut executor, &args);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_cd_current_directory() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // First cd to /tmp for a known location
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();

        // Get current directory
        let before = env::current_dir().unwrap();

        // cd . should stay in current directory
        let result = execute(&mut executor, &vec![".".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Should still be in same directory
        let after = env::current_dir().unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_cd_dash_with_oldpwd() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // First cd to /tmp
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();
        let first_dir = env::current_dir().unwrap();

        // Then cd to /
        execute(&mut executor, &vec!["/".to_string()]).unwrap();

        // Now cd - should take us back to /tmp
        let result = execute(&mut executor, &vec!["-".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify we're back in /tmp (or /private/tmp)
        let current = env::current_dir().unwrap();
        assert_eq!(current, first_dir, "cd - should return to previous directory");
    }

    #[test]
    fn test_oldpwd_and_pwd_updated() {
        let _guard = DirGuard::new();
        let mut executor = CommandExecutor::new();

        // cd to /tmp
        execute(&mut executor, &vec!["/tmp".to_string()]).unwrap();
        let first_pwd = env::current_dir().unwrap();

        // Check PWD is set correctly (may be /tmp or /private/tmp on macOS)
        let pwd = env::var("PWD").unwrap();
        assert_eq!(pwd, first_pwd.to_string_lossy());

        // cd to /
        execute(&mut executor, &vec!["/".to_string()]).unwrap();

        // Check OLDPWD matches first pwd and PWD is /
        let oldpwd = env::var("OLDPWD").unwrap();
        let pwd = env::var("PWD").unwrap();
        assert_eq!(oldpwd, first_pwd.to_string_lossy());
        assert_eq!(pwd, "/");
    }
}

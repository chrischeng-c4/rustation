//! 'cd' built-in command
//!
//! Changes the current working directory with support for:
//! - Absolute and relative paths
//! - Tilde expansion (~, ~/path)
//! - Previous directory (cd -)
//! - CDPATH search

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::env;
use std::path::{Path, PathBuf};

/// Execute the 'cd' command
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Get target directory based on argument
    let target = match args.first().map(|s| s.as_str()) {
        None | Some("") => {
            // cd with no args → HOME
            match executor.env_manager().get("HOME") {
                Some(home) => home.to_string(),
                None => {
                    eprintln!("cd: HOME not set");
                    return Ok(1);
                }
            }
        }
        Some("-") => {
            // cd - → OLDPWD
            match executor.env_manager().get("OLDPWD") {
                Some(oldpwd) => {
                    let path = oldpwd.to_string();
                    // Print the directory when using cd -
                    println!("{}", path);
                    path
                }
                None => {
                    eprintln!("cd: OLDPWD not set");
                    return Ok(1);
                }
            }
        }
        Some(path) => {
            // Expand tilde if present
            let expanded = expand_tilde(path, executor.env_manager().get("HOME"));

            // Check if it's a relative path that doesn't exist locally
            let expanded_path = Path::new(&expanded);
            if !expanded_path.is_absolute() && !expanded_path.exists() {
                // Try CDPATH search
                if let Some(cdpath) = executor.env_manager().get("CDPATH") {
                    if let Some(found) = search_cdpath(&expanded, cdpath) {
                        // Print the resolved path when using CDPATH
                        println!("{}", found.display());
                        return change_directory(&found.to_string_lossy(), executor);
                    }
                }
            }

            expanded
        }
    };

    change_directory(&target, executor)
}

/// Expand tilde at start of path to HOME directory
fn expand_tilde(path: &str, home: Option<&str>) -> String {
    if let Some(home_dir) = home {
        if path == "~" {
            return home_dir.to_string();
        } else if path.starts_with("~/") {
            return path.replacen("~", home_dir, 1);
        }
    }
    path.to_string()
}

/// Search CDPATH for a relative directory name
fn search_cdpath(name: &str, cdpath: &str) -> Option<PathBuf> {
    for dir in cdpath.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = Path::new(dir).join(name);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Validate that path exists and is a directory
fn validate_path(path: &Path) -> std::result::Result<(), String> {
    if !path.exists() {
        return Err("No such file or directory".to_string());
    }
    if !path.is_dir() {
        return Err("Not a directory".to_string());
    }
    Ok(())
}

/// Change to the target directory, updating PWD and OLDPWD
fn change_directory(target: &str, executor: &mut CommandExecutor) -> Result<i32> {
    let path = Path::new(target);

    // Validate the path
    if let Err(err) = validate_path(path) {
        eprintln!("cd: {}: {}", target, err);
        return Ok(1);
    }

    // Save current directory as OLDPWD before changing
    if let Ok(current) = env::current_dir() {
        let _ = executor
            .env_manager_mut()
            .set("OLDPWD".to_string(), current.to_string_lossy().to_string());
    }

    // Change the process working directory
    if let Err(e) = env::set_current_dir(path) {
        eprintln!("cd: {}: {}", target, e);
        return Ok(1);
    }

    // Update PWD to the new directory (canonicalized)
    if let Ok(new_dir) = env::current_dir() {
        let _ = executor
            .env_manager_mut()
            .set("PWD".to_string(), new_dir.to_string_lossy().to_string());
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;
    use std::fs;
    use std::sync::Mutex;

    // Mutex to ensure tests that change working directory run serially
    static CD_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_expand_tilde_home_only() {
        let result = expand_tilde("~", Some("/home/user"));
        assert_eq!(result, "/home/user");
    }

    #[test]
    fn test_expand_tilde_with_path() {
        let result = expand_tilde("~/Documents", Some("/home/user"));
        assert_eq!(result, "/home/user/Documents");
    }

    #[test]
    fn test_expand_tilde_no_home() {
        let result = expand_tilde("~", None);
        assert_eq!(result, "~");
    }

    #[test]
    fn test_expand_tilde_absolute_path() {
        let result = expand_tilde("/tmp", Some("/home/user"));
        assert_eq!(result, "/tmp");
    }

    #[test]
    fn test_expand_tilde_relative_path() {
        let result = expand_tilde("relative/path", Some("/home/user"));
        assert_eq!(result, "relative/path");
    }

    #[test]
    fn test_validate_path_exists() {
        let result = validate_path(Path::new("/tmp"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_not_exists() {
        let result = validate_path(Path::new("/nonexistent_path_12345"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file"));
    }

    #[test]
    fn test_validate_path_not_directory() {
        // This test doesn't change working directory, no mutex needed
        // Create a temp file
        let test_file = "/tmp/rush_cd_test_file";
        fs::write(test_file, "test").unwrap();

        let result = validate_path(Path::new(test_file));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a directory"));

        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_search_cdpath_found() {
        // /tmp should exist
        let result = search_cdpath("tmp", "/:/var");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp"));
    }

    #[test]
    fn test_search_cdpath_not_found() {
        let result = search_cdpath("nonexistent_12345", "/tmp:/var");
        assert!(result.is_none());
    }

    #[test]
    fn test_search_cdpath_empty_entries() {
        let result = search_cdpath("tmp", ":/:/var:");
        assert!(result.is_some());
    }

    #[test]
    fn test_cd_to_tmp() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        let result = execute(&mut executor, &["/tmp".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // On macOS, /tmp is symlinked to /private/tmp
        let cwd = env::current_dir().unwrap();
        assert!(
            cwd.to_string_lossy().contains("tmp"),
            "Expected cwd to contain 'tmp', got: {}",
            cwd.display()
        );

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_no_args_goes_home() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();
        let home = executor.env_manager().get("HOME").unwrap().to_string();

        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let cwd = env::current_dir().unwrap();
        assert_eq!(cwd.to_string_lossy(), home);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_tilde_goes_home() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();
        let home = executor.env_manager().get("HOME").unwrap().to_string();

        let result = execute(&mut executor, &["~".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let cwd = env::current_dir().unwrap();
        assert_eq!(cwd.to_string_lossy(), home);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_nonexistent_fails() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();

        let result = execute(&mut executor, &["/nonexistent_path_12345".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_cd_updates_pwd() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        execute(&mut executor, &["/tmp".to_string()]).unwrap();

        let pwd = executor.env_manager().get("PWD");
        assert!(pwd.is_some());
        assert!(pwd.unwrap().contains("tmp"));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_updates_oldpwd() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        // First cd to establish a known state using a non-symlinked path
        execute(&mut executor, &["/usr".to_string()]).unwrap();
        let usr_path = env::current_dir().unwrap().to_string_lossy().to_string();

        // Now cd to /tmp
        execute(&mut executor, &["/tmp".to_string()]).unwrap();

        let oldpwd = executor.env_manager().get("OLDPWD");
        assert!(oldpwd.is_some());
        assert_eq!(oldpwd.unwrap(), usr_path);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_dash_returns_to_oldpwd() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        // Go to /usr first (not a symlink on macOS)
        execute(&mut executor, &["/usr".to_string()]).unwrap();
        let usr_path = env::current_dir().unwrap().to_string_lossy().to_string();

        // Go to /tmp
        execute(&mut executor, &["/tmp".to_string()]).unwrap();

        // cd - should return to /usr
        let result = execute(&mut executor, &["-".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let cwd = env::current_dir().unwrap();
        assert_eq!(cwd.to_string_lossy(), usr_path);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_empty_string_goes_home() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();
        let home = executor.env_manager().get("HOME").unwrap().to_string();

        let result = execute(&mut executor, &["".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let cwd = env::current_dir().unwrap();
        assert_eq!(cwd.to_string_lossy(), home);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_to_file_fails() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();

        // Create a temp file
        let test_file = "/tmp/rush_cd_test_file_2";
        fs::write(test_file, "test").unwrap();

        let result = execute(&mut executor, &[test_file.to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_cd_relative_path() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        // Go to /usr first (not a symlink)
        env::set_current_dir("/usr").unwrap();

        let result = execute(&mut executor, &["bin".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let cwd = env::current_dir().unwrap();
        assert!(
            cwd.to_string_lossy().contains("bin"),
            "Expected cwd to contain 'bin', got: {}",
            cwd.display()
        );

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cd_parent_directory() {
        let _lock = CD_TEST_MUTEX.lock().expect("mutex");
        let mut executor = CommandExecutor::new();
        let original_dir = env::current_dir().unwrap();

        // Go to /usr/bin first (real directory, not a symlink)
        env::set_current_dir("/usr/bin").unwrap();

        let result = execute(&mut executor, &["..".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Parent of /usr/bin should be /usr
        let cwd = env::current_dir().unwrap();
        assert!(
            cwd.to_string_lossy().ends_with("usr"),
            "Expected cwd to end with 'usr', got: {}",
            cwd.display()
        );

        env::set_current_dir(original_dir).unwrap();
    }
}

//! Configuration management module
//!
//! Provides:
//! - Configuration loading from TOML
//! - Default configuration values
//! - Zero-config operation

pub mod defaults;

pub use defaults::{Config, Theme};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_reexports() {
        // Verify we can use Config and Theme from this module
        let config = Config::default();
        let _theme = Theme::default();
        // Just verify we can construct config
        assert_eq!(config.history_size, 10000);
    }
}

use crate::database::Database;
use crate::date_provider::{DateProvider, OverrideDateProvider};
use chrono::NaiveDate;
use rusqlite::Result;
use std::sync::Arc;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Whether to use in-memory database
    pub is_test_mode: bool,
    /// Custom database file path
    pub custom_path: Option<String>,
    /// Current date for the database (always injected, from CLI or today's date)
    pub current_date: NaiveDate,
}

impl DatabaseConfig {
    /// Gets the effective database path
    ///
    /// Priority:
    /// 1. If custom_path is ":mem:" or "memory", use in-memory database
    /// 2. If custom_path is provided, use that path
    /// 3. If is_test_mode is true, use in-memory database
    /// 4. Otherwise, use default "memory_practice.db"
    pub fn get_path(&self) -> String {
        // Check if custom path is provided
        if let Some(ref path) = self.custom_path {
            // Check if custom path explicitly requests in-memory database
            if path == ":mem:" || path == "memory" {
                ":memory:".to_string()
            } else {
                // Use the custom path as provided
                path.clone()
            }
        } else if self.is_test_mode {
            // Test mode defaults to in-memory
            ":memory:".to_string()
        } else {
            // Production default
            "memory_practice.db".to_string()
        }
    }
}

/// Factory for creating Database instances
pub struct DatabaseFactory;

impl DatabaseFactory {
    /// Creates a database with the specified configuration
    pub fn create(config: DatabaseConfig) -> Result<Database> {
        let path = config.get_path();
        let date_provider: Arc<dyn DateProvider> = Arc::new(OverrideDateProvider::new(config.current_date));
        Database::with_date_provider(&path, date_provider)
    }

    /// Detects the database configuration from command line arguments
    ///
    /// Supported arguments:
    /// - `--test`: Use in-memory database
    /// - `--db-path <path>`: Use custom database file path
    /// - `--override-date <YYYY-MM-DD>`: Override the current date for the database (format: YYYY-MM-DD)
    ///
    /// If `--override-date` is not provided, uses today's date.
    pub fn detect_config() -> DatabaseConfig {
        let args: Vec<String> = std::env::args().collect();
        let is_test_mode = args.iter().any(|arg| arg == "--test");

        let custom_path = args
            .iter()
            .position(|arg| arg == "--db-path")
            .and_then(|idx| args.get(idx + 1).cloned());

        let current_date = args
            .iter()
            .position(|arg| arg == "--override-date")
            .and_then(|idx| args.get(idx + 1))
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());

        DatabaseConfig {
            is_test_mode,
            custom_path,
            current_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Basic default behavior tests =====

    #[test]
    fn test_default_path_production_mode() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "memory_practice.db");
    }

    #[test]
    fn test_default_path_test_mode() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Custom path tests =====

    #[test]
    fn test_custom_path_in_production_mode() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("custom.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "custom.db");
    }

    #[test]
    fn test_custom_path_overrides_test_mode() {
        // When both --test and --db-path custom.db are provided,
        // custom path takes priority over test mode
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some("custom.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "custom.db");
    }

    // ===== In-memory database alias tests =====

    #[test]
    fn test_custom_path_mem_alias_colon() {
        // ":mem:" should be recognized as in-memory database alias
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some(":mem:".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_mem_alias_word() {
        // "memory" should be recognized as in-memory database alias
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("memory".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_mem_alias_overrides_test_mode() {
        // Even with --test, ":mem:" alias should work
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some(":mem:".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_memory_alias_in_production() {
        // "memory" alias should work in production mode too
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("memory".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Custom file path with complex names =====

    #[test]
    fn test_custom_path_with_extension() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("test_database.sqlite3".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "test_database.sqlite3");
    }

    #[test]
    fn test_custom_path_with_directory() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("/tmp/my_app.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "/tmp/my_app.db");
    }

    #[test]
    fn test_custom_path_relative_directory() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("./data/app.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "./data/app.db");
    }

    // ===== Factory creation tests =====

    #[test]
    fn test_create_with_test_mode() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        let db = DatabaseFactory::create(config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_with_memory_database() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        let db = DatabaseFactory::create(config).expect("Failed to create in-memory database");
        // Verify the database works by executing a simple query
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_create_with_memory_alias_mem() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some(":mem:".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        let db = DatabaseFactory::create(config)
            .expect("Failed to create in-memory database with :mem: alias");
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_create_with_memory_alias_word() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("memory".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        let db = DatabaseFactory::create(config)
            .expect("Failed to create in-memory database with memory alias");
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_custom_path_with_test_mode() {
        // Custom path should override test mode
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some("staging.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "staging.db");
    }

    // ===== Priority verification tests =====

    #[test]
    fn test_priority_mem_alias_over_test_mode() {
        // Priority: :mem: alias > test mode
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some(":mem:".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_priority_custom_path_over_test_mode() {
        // Priority: custom path > test mode
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some("override.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "override.db");
    }

    #[test]
    fn test_priority_custom_path_over_default() {
        // Priority: custom path > default
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("custom.db".to_string()),
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), "custom.db");
    }

    #[test]
    fn test_priority_test_mode_over_default() {
        // Priority: test mode > default
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 12).unwrap(),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Current date tests =====

    #[test]
    fn test_current_date_parsing_valid() {
        use chrono::NaiveDate;
        let args = vec![
            "app".to_string(),
            "--override-date".to_string(),
            "2025-11-18".to_string(),
        ];
        let current_date = args
            .iter()
            .position(|arg| arg == "--override-date")
            .and_then(|idx| args.get(idx + 1))
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());

        assert_eq!(current_date, NaiveDate::from_ymd_opt(2025, 11, 18).unwrap());
    }

    #[test]
    fn test_current_date_invalid_format_uses_today() {
        use chrono::NaiveDate;
        let args = vec![
            "app".to_string(),
            "--override-date".to_string(),
            "2025/11/18".to_string(),
        ];
        let current_date = args
            .iter()
            .position(|arg| arg == "--override-date")
            .and_then(|idx| args.get(idx + 1))
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());

        // Should fallback to today's date
        let today = chrono::Local::now().naive_local().date();
        assert_eq!(current_date, today);
    }

    #[test]
    fn test_database_config_with_current_date() {
        use chrono::NaiveDate;
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
            current_date: NaiveDate::from_ymd_opt(2025, 11, 18).unwrap(),
        };
        assert_eq!(
            config.current_date,
            NaiveDate::from_ymd_opt(2025, 11, 18).unwrap()
        );
    }
}

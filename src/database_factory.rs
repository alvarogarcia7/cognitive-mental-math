use crate::database::Database;
use crate::date_provider::{DateProvider, OverrideDateProvider};
use chrono::{NaiveDate, Utc};
use rusqlite::Result;
use std::sync::Arc;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Whether to use in-memory database
    pub is_test_mode: bool,
    /// Database file path
    pub db_path: Option<String>,
    /// Current date for the database (always injected, from CLI or today's date)
    pub current_date: NaiveDate,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfigBuilder::default().build()
    }
}

/// Builder for DatabaseConfig with fluent API
pub struct DatabaseConfigBuilder {
    is_test_mode: bool,
    db_path: Option<String>,
    current_date: NaiveDate,
}

impl DatabaseConfigBuilder {
    fn default() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder {
            is_test_mode: false,
            db_path: None,
            current_date: Utc::now().naive_local().date(),
        }
    }
}

impl DatabaseConfigBuilder {
    /// Set the builder to test mode
    pub fn test_mode(mut self) -> Self {
        self.is_test_mode = true;
        self
    }

    /// Set the database path
    pub fn path(mut self, path: Option<&str>) -> Self {
        self.db_path = path.map(|p| p.to_string());
        self
    }

    /// Set the current date
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.current_date = date;
        self
    }

    pub fn date_ymd(self, year: i32, month: u32, date: u32) -> Self {
        self.date(NaiveDate::from_ymd_opt(year, month, date).unwrap())
    }

    /// Build the DatabaseConfig
    pub fn build(self) -> DatabaseConfig {
        DatabaseConfig {
            is_test_mode: self.is_test_mode,
            db_path: self.db_path,
            current_date: self.current_date,
        }
    }
}

impl DatabaseConfig {
    /// Create a test mode config with a default test date
    pub fn builder() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder::default()
    }

    /// Gets the effective database path
    ///
    /// Priority:
    /// 1. If db_path is ":mem:" or "memory", use in-memory database
    /// 2. If db_path is provided, use that path
    /// 3. If is_test_mode is true, use in-memory database
    /// 4. Otherwise, use default "memory_practice.db"
    pub fn get_path(&self) -> String {
        // Check if database path is provided
        if let Some(ref path) = self.db_path {
            // Check if database path explicitly requests in-memory database
            if path == ":mem:" || path == "memory" {
                ":memory:".to_string()
            } else {
                // Use the database path as provided
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
        let date_provider: Arc<dyn DateProvider> =
            Arc::new(OverrideDateProvider::new(config.current_date));
        Database::with_date_provider(&path, date_provider)
    }

    /// Detects the database configuration from command line arguments using clap
    ///
    /// Supported arguments:
    /// - `--test`: Use in-memory database
    /// - `--db-path <path>`: Use custom database file path
    /// - `--override-date <YYYY-MM-DD>`: Override the current date for the database (format: YYYY-MM-DD)
    ///
    /// If `--override-date` is not provided, uses today's date.
    /// Date validation errors will cause the program to exit with an error message.
    pub fn detect_config() -> DatabaseConfig {
        use crate::cli::Args;

        let args = Args::parse_args();
        let current_date = args
            .validate_override_date()
            .unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            })
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());

        DatabaseConfig {
            is_test_mode: args.test,
            db_path: args
                .db_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
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
        let config = DatabaseConfig::builder().build();
        assert_eq!(config.get_path(), "memory_practice.db");
    }

    #[test]
    fn test_default_path_test_mode() {
        let config = DatabaseConfig::builder().test_mode().build();
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Custom path tests =====

    #[test]
    fn test_custom_path_in_production_mode() {
        let config = DatabaseConfig::builder().path(Some("custom.db")).build();
        assert_eq!(config.get_path(), "custom.db");
    }

    #[test]
    fn test_custom_path_overrides_test_mode() {
        // When both --test and --db-path custom.db are provided,
        // custom path takes priority over test mode
        let config = DatabaseConfig::builder().path(Some("custom.db")).build();
        assert_eq!(config.get_path(), "custom.db");
    }

    // ===== In-memory database alias tests =====

    #[test]
    fn test_custom_path_mem_alias_colon() {
        // ":mem:" should be recognized as in-memory database alias
        let config = DatabaseConfig::builder().path(Some(":mem:")).build();
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_mem_alias_word() {
        // "memory" should be recognized as in-memory database alias
        let config = DatabaseConfig::builder().path(Some("memory")).build();
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_mem_alias_overrides_builder() {
        // Even with --test, ":mem:" alias should work
        let config = DatabaseConfig::builder().path(Some(":mem:")).build();
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path_memory_alias_in_production() {
        // "memory" alias should work in production mode too
        let config = DatabaseConfig::builder().path(Some("memory")).build();
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Custom file path with complex names =====

    #[test]
    fn test_custom_path_with_extension() {
        let config = DatabaseConfig::builder()
            .path(Some("test_database.sqlite3"))
            .build();
        assert_eq!(config.get_path(), "test_database.sqlite3");
    }

    #[test]
    fn test_custom_path_with_directory() {
        let config = DatabaseConfig::builder()
            .path(Some("/tmp/my_app.db"))
            .build();
        assert_eq!(config.get_path(), "/tmp/my_app.db");
    }

    #[test]
    fn test_custom_path_relative_directory() {
        let config = DatabaseConfig::builder()
            .path(Some("./data/app.db"))
            .build();
        assert_eq!(config.get_path(), "./data/app.db");
    }

    // ===== Factory creation tests =====

    #[test]
    fn test_create_with_builder() {
        let config = DatabaseConfig::builder().build();
        let db = DatabaseFactory::create(config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_with_memory_database() {
        let config = DatabaseConfig::builder().build();
        let db = DatabaseFactory::create(config).expect("Failed to create in-memory database");
        // Verify the database works by executing a simple query
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_create_with_memory_alias_mem() {
        let config = DatabaseConfig::builder().path(Some(":mem:")).build();
        let db = DatabaseFactory::create(config)
            .expect("Failed to create in-memory database with :mem: alias");
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_create_with_memory_alias_word() {
        let config = DatabaseConfig::builder().path(Some("memory")).build();
        let db = DatabaseFactory::create(config)
            .expect("Failed to create in-memory database with memory alias");
        assert!(db.count_operations().is_ok());
    }

    #[test]
    fn test_custom_path_with_builder() {
        // Custom path should override test mode
        let config = DatabaseConfig::builder().path(Some("staging.db")).build();
        assert_eq!(config.get_path(), "staging.db");
    }

    // ===== Priority verification tests =====

    #[test]
    fn test_priority_mem_alias_over_builder() {
        // Priority: :mem: alias > test mode
        let config = DatabaseConfig::builder().path(Some(":mem:")).build();
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_priority_custom_path_over_builder() {
        // Priority: custom path > test mode
        let config = DatabaseConfig::builder().path(Some("override.db")).build();
        assert_eq!(config.get_path(), "override.db");
    }

    #[test]
    fn test_priority_custom_path_over_default() {
        // Priority: custom path > default
        let config = DatabaseConfig::builder().path(Some("custom.db")).build();
        assert_eq!(config.get_path(), "custom.db");
    }

    #[test]
    fn test_priority_test_mode_over_default() {
        // Priority: test mode > default
        let config = DatabaseConfig::builder().test_mode().build();
        assert_eq!(config.get_path(), ":memory:");
    }

    // ===== Current date tests =====

    fn parse_date_from_args(args: &[String]) -> NaiveDate {
        args.iter()
            .position(|arg| arg == "--override-date")
            .and_then(|idx| args.get(idx + 1))
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Local::now().naive_local().date())
    }

    #[test]
    fn test_current_date_parsing_valid() {
        use chrono::NaiveDate;
        let args = vec![
            "app".to_string(),
            "--override-date".to_string(),
            "2025-11-18".to_string(),
        ];
        let current_date = parse_date_from_args(&args);
        assert_eq!(current_date, NaiveDate::from_ymd_opt(2025, 11, 18).unwrap());
    }

    #[test]
    fn test_current_date_invalid_format_uses_today() {
        let args = vec![
            "app".to_string(),
            "--override-date".to_string(),
            "2025/11/18".to_string(),
        ];
        let current_date = parse_date_from_args(&args);

        // Should fallback to today's date
        let today = chrono::Local::now().naive_local().date();
        assert_eq!(current_date, today);
    }

    #[test]
    fn test_database_config_with_current_date() {
        use chrono::NaiveDate;
        let config = DatabaseConfig::builder()
            .date(NaiveDate::from_ymd_opt(2025, 11, 18).unwrap())
            .build();
        assert_eq!(
            config.current_date,
            NaiveDate::from_ymd_opt(2025, 11, 18).unwrap()
        );
    }
}

use crate::database::Database;
use rusqlite::Result;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Whether to use in-memory database
    pub is_test_mode: bool,
    /// Custom database file path (ignored if in test mode)
    pub custom_path: Option<String>,
}

impl DatabaseConfig {
    /// Gets the effective database path
    pub fn get_path(&self) -> &str {
        if self.is_test_mode {
            ":memory:"
        } else {
            self.custom_path.as_deref().unwrap_or("memory_practice.db")
        }
    }
}

/// Factory for creating Database instances
pub struct DatabaseFactory;

impl DatabaseFactory {
    /// Creates a database with the specified configuration
    pub fn create(config: DatabaseConfig) -> Result<Database> {
        let path = config.get_path();
        Database::new(path)
    }

    /// Detects the database configuration from command line arguments
    ///
    /// Supported arguments:
    /// - `--test`: Use in-memory database
    /// - `--db-path <path>`: Use custom database file path
    pub fn detect_config() -> DatabaseConfig {
        let args: Vec<String> = std::env::args().collect();
        let is_test_mode = args.iter().any(|arg| arg == "--test");

        let custom_path = args
            .iter()
            .position(|arg| arg == "--db-path")
            .and_then(|idx| args.get(idx + 1).cloned());

        DatabaseConfig {
            is_test_mode,
            custom_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: None,
        };
        assert_eq!(config.get_path(), "memory_practice.db");
    }

    #[test]
    fn test_test_mode_path() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_custom_path() {
        let config = DatabaseConfig {
            is_test_mode: false,
            custom_path: Some("custom.db".to_string()),
        };
        assert_eq!(config.get_path(), "custom.db");
    }

    #[test]
    fn test_test_mode_ignores_custom_path() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: Some("custom.db".to_string()),
        };
        assert_eq!(config.get_path(), ":memory:");
    }

    #[test]
    fn test_create_with_test_mode() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
        };
        let db = DatabaseFactory::create(config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_with_memory_database() {
        let config = DatabaseConfig {
            is_test_mode: true,
            custom_path: None,
        };
        let db = DatabaseFactory::create(config).expect("Failed to create in-memory database");
        // Verify the database works by executing a simple query
        assert!(db.count_operations().is_ok());
    }
}

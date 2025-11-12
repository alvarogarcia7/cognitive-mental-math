use clap::Parser;
use chrono::NaiveDate;
use std::path::PathBuf;

/// Mental math practice application using spaced repetition learning
#[derive(Parser, Debug, Clone)]
#[command(name = "Memory Practice")]
#[command(about = "Exercise mental math with spaced repetition", long_about = None)]
#[command(version)]
pub struct Args {
    /// Use in-memory database for testing
    #[arg(long, help = "Use in-memory database for testing")]
    pub test: bool,

    /// Custom database file path
    #[arg(long, value_name = "PATH", help = "Use custom database file path")]
    pub db_path: Option<PathBuf>,

    /// Override current date for testing (YYYY-MM-DD format)
    #[arg(
        long,
        value_name = "DATE",
        help = "Override current date (YYYY-MM-DD format)"
    )]
    pub override_date: Option<String>,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Args::parse()
    }

    /// Validate the override_date argument if provided
    pub fn validate_override_date(&self) -> Result<Option<NaiveDate>, String> {
        match &self.override_date {
            Some(date_str) => {
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map(Some)
                    .map_err(|_| {
                        format!(
                            "Invalid date format for --override-date: '{}'. Expected YYYY-MM-DD",
                            date_str
                        )
                    })
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_args() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: None,
        };
        assert!(!args.test);
        assert!(args.db_path.is_none());
        assert!(args.override_date.is_none());
    }

    #[test]
    fn test_parse_test_flag() {
        let args = Args {
            test: true,
            db_path: None,
            override_date: None,
        };
        assert!(args.test);
    }

    #[test]
    fn test_parse_db_path() {
        let args = Args {
            test: false,
            db_path: Some(PathBuf::from("/tmp/test.db")),
            override_date: None,
        };
        assert_eq!(args.db_path.as_deref(), Some(PathBuf::from("/tmp/test.db").as_path()));
    }

    #[test]
    fn test_parse_override_date() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: Some("2024-01-15".to_string()),
        };
        assert_eq!(args.override_date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_validate_override_date_valid() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: Some("2024-01-15".to_string()),
        };
        let result = args.validate_override_date();
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_validate_override_date_invalid_format() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: Some("2024/01/15".to_string()),
        };
        let result = args.validate_override_date();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid date format"));
    }

    #[test]
    fn test_validate_override_date_invalid_date() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: Some("2024-13-01".to_string()),
        };
        let result = args.validate_override_date();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_override_date_none() {
        let args = Args {
            test: false,
            db_path: None,
            override_date: None,
        };
        let result = args.validate_override_date();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_all_args_set() {
        let args = Args {
            test: true,
            db_path: Some(PathBuf::from("/tmp/test.db")),
            override_date: Some("2024-06-15".to_string()),
        };
        assert!(args.test);
        assert_eq!(args.db_path.as_deref(), Some(PathBuf::from("/tmp/test.db").as_path()));
        assert_eq!(args.override_date, Some("2024-06-15".to_string()));
    }

    #[test]
    fn test_validate_all_args_with_valid_date() {
        let args = Args {
            test: true,
            db_path: Some(PathBuf::from("/tmp/test.db")),
            override_date: Some("2024-12-31".to_string()),
        };
        let result = args.validate_override_date();
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()));
    }
}

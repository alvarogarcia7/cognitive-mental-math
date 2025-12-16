pub mod analytics;
pub mod answers;
pub mod connection;
pub mod decks;
pub mod operations;
pub mod review_items;

use crate::date_provider::{DateProvider, SystemDateProvider};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};
use std::sync::Arc;

pub use analytics::Analytics;
pub use answers::{AnswerRecord, AnswersRepository};
pub use decks::DecksRepository;
pub use operations::{OperationRecord, OperationsRepository};
pub use review_items::ReviewItemsRepository;

/// Main Database struct providing access to all repositories
pub struct Database {
    pub conn: Connection,
    date_provider: Arc<dyn DateProvider>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        Self::init(db_path, Arc::new(SystemDateProvider))
    }

    pub fn with_date_provider(db_path: &str, date_provider: Arc<dyn DateProvider>) -> Result<Self> {
        Self::init(db_path, date_provider)
    }

    fn init(db_path: &str, date_provider: Arc<dyn DateProvider>) -> Result<Self> {
        let conn = connection::init_connection(db_path)?;
        Ok(Database {
            conn,
            date_provider,
        })
    }

    /// Helper method to get the current time (delegates to date provider)
    pub fn get_current_time(&self) -> DateTime<Utc> {
        self.date_provider.get_current_time()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::analytics::StreakRepository;
    use crate::database_factory::{DatabaseConfig, DatabaseFactory};

    fn create_test_db() -> Database {
        // Use an in-memory database for each test
        Database::new(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_db();
        // Verify tables were created by checking counts
        let repo = OperationsRepository::new(&db.conn);
        assert_eq!(repo.count().unwrap(), 0);
        let repo1 = AnswersRepository::new(&db.conn);
        assert_eq!(repo1.count().unwrap(), 0);
    }

    #[test]
    fn test_insert_operation() {
        let db = create_test_db();
        let repo = OperationsRepository::new(&db.conn);
        let op_id = repo.insert("ADD", 5, 3, 8, None).unwrap();
        assert_eq!(op_id, 1);

        let repo1 = OperationsRepository::new(&db.conn);
        let op_record = repo1.get(op_id).unwrap().unwrap();
        assert_eq!(op_record.operation_type, "ADD");
        assert_eq!(op_record.operand1, 5);
        assert_eq!(op_record.operand2, 3);
        assert_eq!(op_record.result, 8);
    }

    #[test]
    fn test_insert_multiple_operations() {
        let db = create_test_db();
        let repo = OperationsRepository::new(&db.conn);
        let id1 = repo.insert("ADD", 10, 20, 30, None).unwrap();
        let repo = OperationsRepository::new(&db.conn);
        let id2 = repo.insert("MULTIPLY", 4, 5, 20, None).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        let repo1 = OperationsRepository::new(&db.conn);
        assert_eq!(repo1.count().unwrap(), 2);
    }

    #[test]
    fn test_insert_answer() {
        let db = create_test_db();
        let repo = OperationsRepository::new(&db.conn);
        let op_id = repo.insert("MULTIPLY", 7, 8, 56, None).unwrap();

        let repo1 = AnswersRepository::new(&db.conn);
        repo1.insert(op_id, 56, true, 2.5, None).unwrap();
        let repo2 = AnswersRepository::new(&db.conn);
        assert_eq!(repo2.count().unwrap(), 1);

        let repo2 = AnswersRepository::new(&db.conn);
        let answer = repo2.get(1).unwrap().unwrap();
        assert_eq!(answer.operation_id, op_id);
        assert_eq!(answer.user_answer, 56);
        assert!(answer.is_correct);
        assert_eq!(answer.time_spent_seconds, 2.5);
    }

    #[test]
    fn test_create_deck() {
        let db = create_test_db();
        let current_time = db.get_current_time();
        let repo = DecksRepository::new(&db.conn, Box::new(move || current_time));
        let deck_id = repo.create().unwrap();
        assert_eq!(deck_id, 1);
        let repo1 = DecksRepository::new(&db.conn, Box::new(|| db.get_current_time()));
        assert_eq!(repo1.count().unwrap(), 1);
    }

    #[test]
    fn test_operations_with_deck_id() {
        let db = create_test_db();
        let current_time = db.get_current_time();
        let repo = DecksRepository::new(&db.conn, Box::new(move || current_time));
        let deck_id = repo.create().unwrap();

        let deck_id1 = Some(deck_id);
        let repo = OperationsRepository::new(&db.conn);
        let op_id = repo.insert("ADD", 5, 3, 8, deck_id1).unwrap();
        let deck_id2 = Some(deck_id);
        let repo1 = AnswersRepository::new(&db.conn);
        repo1.insert(op_id, 8, true, 2.0, deck_id2).unwrap();

        let repo1 = OperationsRepository::new(&db.conn);
        assert_eq!(repo1.count().unwrap(), 1);
        let repo2 = AnswersRepository::new(&db.conn);
        assert_eq!(repo2.count().unwrap(), 1);
    }

    #[test]
    fn test_get_days_with_answers_for_given_day() {
        let db_config = DatabaseConfig::builder()
            .test_mode()
            .path(Some(":memory:"))
            .date_ymd(2025, 11, 12)
            .build();
        let db = DatabaseFactory::create(db_config).unwrap();
        let current_time = db.get_current_time();
        let decks_repo = DecksRepository::new(&db.conn, Box::new(move || current_time));
        let date_provider_fn = Box::new(move || current_time);
        let answers_repo = AnswersRepository::new_with_date_provider(&db.conn, &*date_provider_fn);
        let operations_repo = OperationsRepository::new(&db.conn);

        let deck_id1 = Some(decks_repo.create().unwrap());
        let op_id = operations_repo.insert("ADD", 2, 3, 5, deck_id1).unwrap();
        let deck_id2 = Some(decks_repo.create().unwrap());
        answers_repo.insert(op_id, 5, true, 1.0, deck_id2).unwrap();

        let analytics = Analytics::new(&db.conn);
        let days_with_answers = StreakRepository::new(analytics.conn)
            .get_days_with_answers(current_time)
            .unwrap();
        assert_eq!(days_with_answers, vec!["2025-11-12"]);
    }
}

pub mod analytics;
pub mod answers;
pub mod connection;
pub mod decks;
pub mod operations;
pub mod review_items;

use crate::date_provider::{DateProvider, SystemDateProvider};
use crate::deck::{Deck, DeckSummary};
use crate::spaced_repetition::{AnswerTimedEvaluator, ReviewItem};
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
    fn get_current_time(&self) -> DateTime<Utc> {
        self.date_provider.get_current_time()
    }

    // ===== Operations Repository Access =====

    pub fn insert_operation(
        &self,
        operation_type: &str,
        operand1: i32,
        operand2: i32,
        result: i32,
        deck_id: Option<i64>,
    ) -> Result<i64> {
        let repo = OperationsRepository::new(&self.conn);
        repo.insert(operation_type, operand1, operand2, result, deck_id)
    }

    pub fn get_operation(&self, operation_id: i64) -> Result<Option<OperationRecord>> {
        let repo = OperationsRepository::new(&self.conn);
        repo.get(operation_id)
    }

    pub fn count_operations(&self) -> Result<i64> {
        let repo = OperationsRepository::new(&self.conn);
        repo.count()
    }

    // ===== Answers Repository Access =====

    pub fn insert_answer(
        &self,
        operation_id: i64,
        user_answer: i32,
        is_correct: bool,
        time_spent_seconds: f64,
        deck_id: Option<i64>,
    ) -> Result<()> {
        let repo = AnswersRepository::new(&self.conn);
        repo.insert(
            operation_id,
            user_answer,
            is_correct,
            time_spent_seconds,
            deck_id,
        )
    }

    pub fn get_answer(&self, answer_id: i64) -> Result<Option<AnswerRecord>> {
        let repo = AnswersRepository::new(&self.conn);
        repo.get(answer_id)
    }

    pub fn count_answers(&self) -> Result<i64> {
        let repo = AnswersRepository::new(&self.conn);
        repo.count()
    }

    // ===== Decks Repository Access =====

    pub fn create_deck(&self) -> Result<i64> {
        let current_time = self.get_current_time();
        let repo = DecksRepository::new(&self.conn, Box::new(move || current_time));
        repo.create()
    }

    pub fn get_deck(&self, deck_id: i64) -> Result<Option<Deck>> {
        let repo = DecksRepository::new(&self.conn, Box::new(|| self.get_current_time()));
        repo.get(deck_id)
    }

    pub fn update_deck_summary(&self, deck_id: i64, summary: &DeckSummary) -> Result<()> {
        let repo = DecksRepository::new(&self.conn, Box::new(|| self.get_current_time()));
        repo.update_summary(deck_id, summary)
    }

    pub fn complete_deck(&self, deck_id: i64) -> Result<()> {
        let current_time = self.get_current_time();
        let repo = DecksRepository::new(&self.conn, Box::new(move || current_time));
        repo.complete(deck_id)
    }

    pub fn abandon_deck(&self, deck_id: i64) -> Result<()> {
        let repo = DecksRepository::new(&self.conn, Box::new(|| self.get_current_time()));
        repo.abandon(deck_id)
    }

    pub fn get_recent_decks(&self, limit: i32) -> Result<Vec<Deck>> {
        let repo = DecksRepository::new(&self.conn, Box::new(|| self.get_current_time()));
        repo.get_recent(limit)
    }

    pub fn count_decks(&self) -> Result<i64> {
        let repo = DecksRepository::new(&self.conn, Box::new(|| self.get_current_time()));
        repo.count()
    }

    // ===== Review Items Repository Access =====

    pub fn insert_review_item(
        &self,
        operation_id: i64,
        next_review_date: DateTime<Utc>,
    ) -> Result<i64> {
        let repo = ReviewItemsRepository::new(&self.conn);
        repo.insert(operation_id, next_review_date)
    }

    pub fn update_review_item(&self, item: &ReviewItem) -> Result<()> {
        let repo = ReviewItemsRepository::new(&self.conn);
        repo.update(item)
    }

    pub fn get_review_item(&self, operation_id: i64) -> Result<Option<ReviewItem>> {
        let repo = ReviewItemsRepository::new(&self.conn);
        repo.get(operation_id)
    }

    pub fn get_due_reviews(&self, before_date: DateTime<Utc>) -> Result<Vec<ReviewItem>> {
        let repo = ReviewItemsRepository::new(&self.conn);
        repo.get_due(before_date)
    }

    pub fn count_due_reviews(&self, before_date: DateTime<Utc>) -> Result<i64> {
        let repo = ReviewItemsRepository::new(&self.conn);
        repo.count_due(before_date)
    }

    // ===== Analytics Access =====

    pub fn compute_time_statistics(
        &self,
        operation_type: &str,
    ) -> Result<Option<AnswerTimedEvaluator>> {
        let analytics = Analytics::new(&self.conn);
        analytics
            .time_statistics()
            .for_operation_type(operation_type)
    }

    pub fn compute_time_statistics_all_operations(
        &self,
    ) -> Result<std::collections::HashMap<String, AnswerTimedEvaluator>> {
        let analytics = Analytics::new(&self.conn);
        analytics.time_statistics().all_operations()
    }

    pub fn compute_time_statistics_all_operations_last_30_days(
        &self,
    ) -> Result<std::collections::HashMap<String, AnswerTimedEvaluator>> {
        let analytics = Analytics::new(&self.conn);
        analytics.time_statistics().all_operations_last_30_days()
    }

    pub fn compute_time_statistics_all_operations_last_10_decks(
        &self,
    ) -> Result<std::collections::HashMap<String, AnswerTimedEvaluator>> {
        let analytics = Analytics::new(&self.conn);
        analytics.time_statistics().all_operations_last_10_decks()
    }

    pub fn compute_accuracy_all_operations(
        &self,
    ) -> Result<std::collections::HashMap<String, (i64, i64, f64)>> {
        let analytics = Analytics::new(&self.conn);
        analytics.accuracy().all_operations()
    }

    pub fn compute_accuracy_all_operations_last_30_days(
        &self,
    ) -> Result<std::collections::HashMap<String, (i64, i64, f64)>> {
        let analytics = Analytics::new(&self.conn);
        analytics.accuracy().all_operations_last_30_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database_factory::{DatabaseConfig, DatabaseFactory};

    fn create_test_db() -> Database {
        // Use an in-memory database for each test
        Database::new(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_db();
        // Verify tables were created by checking counts
        assert_eq!(db.count_operations().unwrap(), 0);
        assert_eq!(db.count_answers().unwrap(), 0);
    }

    #[test]
    fn test_insert_operation() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 5, 3, 8, None).unwrap();
        assert_eq!(op_id, 1);

        let op_record = db.get_operation(op_id).unwrap().unwrap();
        assert_eq!(op_record.operation_type, "ADD");
        assert_eq!(op_record.operand1, 5);
        assert_eq!(op_record.operand2, 3);
        assert_eq!(op_record.result, 8);
    }

    #[test]
    fn test_insert_multiple_operations() {
        let db = create_test_db();
        let id1 = db.insert_operation("ADD", 10, 20, 30, None).unwrap();
        let id2 = db.insert_operation("MULTIPLY", 4, 5, 20, None).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(db.count_operations().unwrap(), 2);
    }

    #[test]
    fn test_insert_answer() {
        let db = create_test_db();
        let op_id = db.insert_operation("MULTIPLY", 7, 8, 56, None).unwrap();

        db.insert_answer(op_id, 56, true, 2.5, None).unwrap();
        assert_eq!(db.count_answers().unwrap(), 1);

        let answer = db.get_answer(1).unwrap().unwrap();
        assert_eq!(answer.operation_id, op_id);
        assert_eq!(answer.user_answer, 56);
        assert!(answer.is_correct);
        assert_eq!(answer.time_spent_seconds, 2.5);
    }

    #[test]
    fn test_create_deck() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        assert_eq!(deck_id, 1);
        assert_eq!(db.count_decks().unwrap(), 1);
    }

    #[test]
    fn test_operations_with_deck_id() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        let op_id = db.insert_operation("ADD", 5, 3, 8, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 8, true, 2.0, Some(deck_id))
            .unwrap();

        assert_eq!(db.count_operations().unwrap(), 1);
        assert_eq!(db.count_answers().unwrap(), 1);
    }

    #[test]
    fn test_get_days_with_answers_for_given_day() {
        let db_config = DatabaseConfig::builder()
            .test_mode()
            .date_ymd(2025, 11, 12)
            .build();
        let db = DatabaseFactory::create(db_config).unwrap();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 5, true, 1.0, Some(deck_id))
            .unwrap();

        let now = Utc::now();
        let analytics = Analytics::new(&db.conn);
        let days_with_answers = analytics.streak().get_days_with_answers(now).unwrap();
        assert_eq!(days_with_answers, vec!["2025-11-12"]);
    }
}

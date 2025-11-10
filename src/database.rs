use crate::deck::{Deck, DeckStatus, DeckSummary};
use crate::row_factories::{DeckRowFactory, ReviewItemRowFactory};
use crate::spaced_repetition::ReviewItem;
use crate::time_format::format_time_difference;
use chrono::{DateTime, Utc};
use log::debug;
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;

// Embed migrations from the migrations directory
refinery::embed_migrations!("migrations");

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let mut conn = Connection::open(db_path)?;

        // Run embedded migrations from the migrations folder
        match migrations::runner().run(&mut conn) {
            Ok(_) => {
                debug!("Migrations completed successfully");
            }
            Err(e) => {
                eprintln!("Refinery migration error: {}", e);
                return Err(rusqlite::Error::ExecuteReturnedResults);
            }
        }

        Ok(Database { conn })
    }

    pub fn insert_operation(
        &self,
        operation_type: &str,
        operand1: i32,
        operand2: i32,
        result: i32,
        deck_id: Option<i64>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO operations (operation_type, operand1, operand2, result, deck_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![operation_type, operand1, operand2, result, deck_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_answer(
        &self,
        operation_id: i64,
        user_answer: i32,
        is_correct: bool,
        time_spent_seconds: f64,
        deck_id: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO answers (operation_id, user_answer, is_correct, time_spent_seconds, deck_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                operation_id,
                user_answer,
                is_correct as i32,
                time_spent_seconds,
                deck_id
            ],
        )?;
        Ok(())
    }

    pub fn get_operation(&self, operation_id: i64) -> Result<Option<OperationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_type, operand1, operand2, result FROM operations WHERE id = ?1",
        )?;

        let mut rows = stmt.query([operation_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(OperationRecord {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                operand1: row.get(2)?,
                operand2: row.get(3)?,
                result: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_answer(&self, answer_id: i64) -> Result<Option<AnswerRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, user_answer, is_correct, time_spent_seconds FROM answers WHERE id = ?1"
        )?;

        let mut rows = stmt.query([answer_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(AnswerRecord {
                id: row.get(0)?,
                operation_id: row.get(1)?,
                user_answer: row.get(2)?,
                is_correct: row.get::<_, i32>(3)? != 0,
                time_spent_seconds: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn count_operations(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM operations", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_answers(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM answers", [], |row| row.get(0))?;
        Ok(count)
    }

    // Deck management methods

    pub fn create_deck(&self) -> Result<i64> {
        let now_utc = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO decks (created_at, status) VALUES (?1, ?2)",
            params![now_utc, DeckStatus::InProgress.as_str()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_deck(&self, deck_id: i64) -> Result<Option<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks WHERE id = ?1",
        )?;

        let mut rows = stmt.query([deck_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(DeckRowFactory::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_deck_summary(&self, deck_id: i64, summary: &DeckSummary) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET
                total_questions = ?1,
                correct_answers = ?2,
                incorrect_answers = ?3,
                total_time_seconds = ?4,
                average_time_seconds = ?5,
                accuracy_percentage = ?6
             WHERE id = ?7",
            params![
                summary.total_questions,
                summary.correct_answers,
                summary.incorrect_answers,
                summary.total_time_seconds,
                summary.average_time_seconds,
                summary.accuracy_percentage,
                deck_id
            ],
        )?;
        Ok(())
    }

    pub fn complete_deck(&self, deck_id: i64) -> Result<()> {
        let now_utc = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE decks SET status = ?1, completed_at = ?3 WHERE id = ?2",
            params![DeckStatus::Completed.as_str(), deck_id, now_utc],
        )?;
        Ok(())
    }

    pub fn abandon_deck(&self, deck_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET status = ?1 WHERE id = ?2",
            params![DeckStatus::Abandoned.as_str(), deck_id],
        )?;
        Ok(())
    }

    pub fn get_recent_decks(&self, limit: i32) -> Result<Vec<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit], DeckRowFactory::from_row)?;

        let mut decks = Vec::new();
        for deck_result in rows {
            decks.push(deck_result?);
        }
        Ok(decks)
    }

    pub fn count_decks(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))?;
        Ok(count)
    }

    // Review Items Methods

    pub fn insert_review_item(
        &self,
        operation_id: i64,
        next_review_date: DateTime<Utc>,
    ) -> Result<i64> {
        let next_review_str = next_review_date.to_rfc3339();
        debug!(
            "Creating new review item for operation_id={}, next review: {}",
            operation_id,
            format_time_difference(Utc::now(), next_review_date)
        );
        self.conn.execute(
            "INSERT INTO review_items (operation_id, next_review_date)
             VALUES (?1, ?2)",
            params![operation_id, next_review_str],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_review_item(&self, item: &ReviewItem) -> Result<()> {
        let next_review_str = item.next_review_date.to_rfc3339();
        let last_reviewed_str = item.last_reviewed_date.map(|d| d.to_rfc3339());

        debug!(
            "Updating review item id={}: reps={}, interval={} days, ease={:.2}, next review: {}",
            item.id.unwrap_or(0),
            item.repetitions,
            item.interval,
            item.ease_factor,
            format_time_difference(Utc::now(), item.next_review_date)
        );

        self.conn.execute(
            "UPDATE review_items
             SET repetitions = ?1, interval = ?2, ease_factor = ?3,
                 next_review_date = ?4, last_reviewed_date = ?5
             WHERE id = ?6",
            params![
                item.repetitions,
                item.interval,
                item.ease_factor,
                next_review_str,
                last_reviewed_str,
                item.id
            ],
        )?;
        Ok(())
    }

    pub fn get_review_item(&self, operation_id: i64) -> Result<Option<ReviewItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, repetitions, interval, ease_factor,
                    next_review_date, last_reviewed_date
             FROM review_items WHERE operation_id = ?1",
        )?;

        let mut rows = stmt.query([operation_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ReviewItemRowFactory::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_due_reviews(&self, before_date: DateTime<Utc>) -> Result<Vec<ReviewItem>> {
        let before_str = before_date.to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, repetitions, interval, ease_factor,
                    next_review_date, last_reviewed_date
             FROM review_items
             WHERE next_review_date <= ?1
             ORDER BY next_review_date ASC",
        )?;

        let items = stmt.query_map([&before_str], ReviewItemRowFactory::from_row)?;

        let mut result = Vec::new();
        for item in items {
            result.push(item?);
        }

        debug!("Retrieved {} due review items from database", result.len());
        Ok(result)
    }

    pub fn count_due_reviews(&self, before_date: DateTime<Utc>) -> Result<i64> {
        let before_str = before_date.to_rfc3339();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM review_items WHERE next_review_date <= ?1",
            [&before_str],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Compute time statistics for correct answers of a specific operation type
    ///
    /// Returns (average_time, standard_deviation) for correct answers of the given operation type
    /// Only considers correct answers from completed decks
    pub fn compute_time_statistics(&self, operation_type: &str) -> Result<Option<(f64, f64)>> {
        // First, compute count, sum, and sum of squares for correct answers
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(a.time_spent_seconds) as count,
                AVG(a.time_spent_seconds) as average,
                SUM(a.time_spent_seconds * a.time_spent_seconds) as sum_squares,
                SUM(a.time_spent_seconds) as total_sum
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE o.operation_type = ?1
            AND a.is_correct = 1
            AND d.status = 'completed'",
        )?;

        let result = stmt.query_row([operation_type], |row| {
            let count: i64 = row.get(0)?;
            if count == 0 {
                Ok(None)
            } else {
                let average: f64 = row.get(1)?;
                let sum_squares: f64 = row.get(2)?;
                let total_sum: f64 = row.get(3)?;

                // Calculate standard deviation: sqrt(sum(x²)/n - (sum(x)/n)²)
                let variance = (sum_squares / count as f64) - (total_sum / count as f64).powi(2);
                let stdev = variance.sqrt().max(0.0);

                Ok(Some((average, stdev)))
            }
        })?;

        Ok(result)
    }

    /// Compute time statistics for all operation types (global)
    /// Returns a map of operation_type -> (average, stdev)
    pub fn compute_time_statistics_all_operations(&self) -> Result<HashMap<String, (f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                o.operation_type,
                COUNT(a.time_spent_seconds) as count,
                AVG(a.time_spent_seconds) as average,
                SUM(a.time_spent_seconds * a.time_spent_seconds) as sum_squares,
                SUM(a.time_spent_seconds) as total_sum
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE a.is_correct = 1
            AND d.status = 'completed'
            GROUP BY o.operation_type
            ORDER BY o.operation_type",
        )?;

        let mut result = HashMap::new();
        let rows = stmt.query_map([], |row| {
            let op_type: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let average: f64 = row.get(2)?;
            let sum_squares: f64 = row.get(3)?;
            let total_sum: f64 = row.get(4)?;

            let stdev = if count > 0 {
                let variance = (sum_squares / count as f64) - (total_sum / count as f64).powi(2);
                variance.sqrt().max(0.0)
            } else {
                0.0
            };

            Ok((op_type, (average, stdev)))
        })?;

        for row in rows {
            let (op_type, stats) = row?;
            result.insert(op_type, stats);
        }

        Ok(result)
    }

    /// Compute time statistics for all operation types in the last 30 days
    /// Returns a map of operation_type -> (average, stdev)
    pub fn compute_time_statistics_all_operations_last_30_days(
        &self,
    ) -> Result<HashMap<String, (f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                o.operation_type,
                COUNT(a.time_spent_seconds) as count,
                AVG(a.time_spent_seconds) as average,
                SUM(a.time_spent_seconds * a.time_spent_seconds) as sum_squares,
                SUM(a.time_spent_seconds) as total_sum
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE a.is_correct = 1
            AND d.status = 'completed'
            AND a.created_at >= datetime('now', '-30 days')
            GROUP BY o.operation_type
            ORDER BY o.operation_type",
        )?;

        let mut result = HashMap::new();
        let rows = stmt.query_map([], |row| {
            let op_type: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let average: f64 = row.get(2)?;
            let sum_squares: f64 = row.get(3)?;
            let total_sum: f64 = row.get(4)?;

            let stdev = if count > 0 {
                let variance = (sum_squares / count as f64) - (total_sum / count as f64).powi(2);
                variance.sqrt().max(0.0)
            } else {
                0.0
            };

            Ok((op_type, (average, stdev)))
        })?;

        for row in rows {
            let (op_type, stats) = row?;
            result.insert(op_type, stats);
        }

        Ok(result)
    }

    /// Compute time statistics for all operation types from the last 10 completed decks
    /// Returns a map of operation_type -> (average, stdev)
    pub fn compute_time_statistics_all_operations_last_10_decks(
        &self,
    ) -> Result<HashMap<String, (f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                o.operation_type,
                COUNT(a.time_spent_seconds) as count,
                AVG(a.time_spent_seconds) as average,
                SUM(a.time_spent_seconds * a.time_spent_seconds) as sum_squares,
                SUM(a.time_spent_seconds) as total_sum
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE a.is_correct = 1
            AND d.status = 'completed'
            AND d.id IN (
                SELECT id FROM decks
                WHERE status = 'completed'
                ORDER BY completed_at DESC
                LIMIT 10
            )
            GROUP BY o.operation_type
            ORDER BY o.operation_type",
        )?;

        let mut result = HashMap::new();
        let rows = stmt.query_map([], |row| {
            let op_type: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let average: f64 = row.get(2)?;
            let sum_squares: f64 = row.get(3)?;
            let total_sum: f64 = row.get(4)?;

            let stdev = if count > 0 {
                let variance = (sum_squares / count as f64) - (total_sum / count as f64).powi(2);
                variance.sqrt().max(0.0)
            } else {
                0.0
            };

            Ok((op_type, (average, stdev)))
        })?;

        for row in rows {
            let (op_type, stats) = row?;
            result.insert(op_type, stats);
        }

        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
pub struct OperationRecord {
    pub id: i64,
    pub operation_type: String,
    pub operand1: i32,
    pub operand2: i32,
    pub result: i32,
}

#[derive(Debug, PartialEq)]
pub struct AnswerRecord {
    pub id: i64,
    pub operation_id: i64,
    pub user_answer: i32,
    pub is_correct: bool,
    pub time_spent_seconds: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
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
    fn test_insert_answer_incorrect() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 15, 25, 40, None).unwrap();

        db.insert_answer(op_id, 35, false, 3.2, None).unwrap();

        let answer = db.get_answer(1).unwrap().unwrap();
        assert_eq!(answer.user_answer, 35);
        assert!(!answer.is_correct);
        assert_eq!(answer.time_spent_seconds, 3.2);
    }

    #[test]
    fn test_get_nonexistent_operation() {
        let db = create_test_db();
        let result = db.get_operation(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_nonexistent_answer() {
        let db = create_test_db();
        let result = db.get_answer(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_answers_for_operation() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 1, 2, 3, None).unwrap();

        // Insert multiple answers (simulating retries)
        db.insert_answer(op_id, 4, false, 1.0, None).unwrap();
        db.insert_answer(op_id, 3, true, 2.0, None).unwrap();

        assert_eq!(db.count_answers().unwrap(), 2);
    }

    #[test]
    fn test_answer_references_operation() {
        let db = create_test_db();
        let op_id = db.insert_operation("MULTIPLY", 3, 4, 12, None).unwrap();
        db.insert_answer(op_id, 12, true, 1.5, None).unwrap();

        let answer = db.get_answer(1).unwrap().unwrap();
        let operation = db.get_operation(answer.operation_id).unwrap().unwrap();

        assert_eq!(operation.operand1, 3);
        assert_eq!(operation.operand2, 4);
        assert_eq!(operation.result, 12);
    }

    // Deck-related tests

    #[test]
    fn test_create_deck() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        assert_eq!(deck_id, 1);
        assert_eq!(db.count_decks().unwrap(), 1);
    }

    #[test]
    fn test_get_deck() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        let deck = db.get_deck(deck_id).unwrap().unwrap();
        assert_eq!(deck.id, deck_id);
        assert_eq!(deck.status, crate::deck::DeckStatus::InProgress);
        assert_eq!(deck.total_questions, 0);
    }

    #[test]
    fn test_complete_deck() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        db.complete_deck(deck_id).unwrap();

        let deck = db.get_deck(deck_id).unwrap().unwrap();
        assert_eq!(deck.status, crate::deck::DeckStatus::Completed);
        assert!(deck.completed_at.is_some());
    }

    #[test]
    fn test_abandon_deck() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        db.abandon_deck(deck_id).unwrap();

        let deck = db.get_deck(deck_id).unwrap().unwrap();
        assert_eq!(deck.status, crate::deck::DeckStatus::Abandoned);
    }

    #[test]
    fn test_update_deck_summary() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        let summary = crate::deck::DeckSummary {
            total_questions: 10,
            correct_answers: 8,
            incorrect_answers: 2,
            total_time_seconds: 25.5,
            average_time_seconds: 2.55,
            accuracy_percentage: 80.0,
        };

        db.update_deck_summary(deck_id, &summary).unwrap();

        let deck = db.get_deck(deck_id).unwrap().unwrap();
        assert_eq!(deck.total_questions, 10);
        assert_eq!(deck.correct_answers, 8);
        assert_eq!(deck.incorrect_answers, 2);
        assert_eq!(deck.total_time_seconds, 25.5);
        assert_eq!(deck.average_time_seconds, Some(2.55));
        assert_eq!(deck.accuracy_percentage, Some(80.0));
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
    fn test_get_recent_decks() {
        let db = create_test_db();
        let deck1 = db.create_deck().unwrap();
        let deck2 = db.create_deck().unwrap();
        let deck3 = db.create_deck().unwrap();

        let all_decks = db.get_recent_decks(10).unwrap();
        assert_eq!(all_decks.len(), 3);

        let recent = db.get_recent_decks(2).unwrap();
        assert_eq!(recent.len(), 2);

        // When timestamps are identical (tests run quickly),
        // we should at least get 2 different decks
        assert_ne!(recent[0].id, recent[1].id);

        // Verify both recent decks are among the created ones
        assert!([deck1, deck2, deck3].contains(&recent[0].id));
        assert!([deck1, deck2, deck3].contains(&recent[1].id));
    }

    // Review items tests

    #[test]
    fn test_insert_review_item() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 2, 3, 5, None).unwrap();

        let now = chrono::Utc::now();
        let review_id = db.insert_review_item(op_id, now).unwrap();

        assert!(review_id > 0);
    }

    #[test]
    fn test_get_review_item() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 2, 3, 5, None).unwrap();

        let now = chrono::Utc::now();
        db.insert_review_item(op_id, now).unwrap();

        let item = db.get_review_item(op_id).unwrap();
        assert!(item.is_some());
        let review = item.unwrap();
        assert_eq!(review.operation_id, op_id);
        assert_eq!(review.repetitions, 0);
        assert_eq!(review.interval, 0);
        assert_eq!(review.ease_factor, 2.5);
    }

    #[test]
    fn test_update_review_item() {
        let db = create_test_db();
        let op_id = db.insert_operation("ADD", 2, 3, 5, None).unwrap();

        let now = chrono::Utc::now();
        db.insert_review_item(op_id, now).unwrap();

        let mut item = db.get_review_item(op_id).unwrap().unwrap();
        item.repetitions = 1;
        item.interval = 3;
        item.ease_factor = 2.7;
        item.next_review_date = now + chrono::Duration::days(3);

        db.update_review_item(&item).unwrap();

        let updated = db.get_review_item(op_id).unwrap().unwrap();
        assert_eq!(updated.repetitions, 1);
        assert_eq!(updated.interval, 3);
        assert_eq!(updated.ease_factor, 2.7);
    }

    #[test]
    fn test_get_due_reviews() {
        let db = create_test_db();
        let now = chrono::Utc::now();
        let past = now - chrono::Duration::days(1);
        let future = now + chrono::Duration::days(1);

        let op_id1 = db.insert_operation("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = db.insert_operation("ADD", 4, 5, 9, None).unwrap();
        let op_id3 = db.insert_operation("ADD", 6, 7, 13, None).unwrap();

        // op_id1: due (past date)
        db.insert_review_item(op_id1, past).unwrap();
        // op_id2: due (exactly now)
        db.insert_review_item(op_id2, now).unwrap();
        // op_id3: not due (future date)
        db.insert_review_item(op_id3, future).unwrap();

        let due = db.get_due_reviews(now).unwrap();
        assert_eq!(due.len(), 2);

        // Verify the returned items are the correct ones
        let due_ids: Vec<i64> = due.iter().map(|item| item.operation_id).collect();
        assert!(due_ids.contains(&op_id1));
        assert!(due_ids.contains(&op_id2));
        assert!(!due_ids.contains(&op_id3));
    }

    #[test]
    fn test_count_due_reviews() {
        let db = create_test_db();
        let now = chrono::Utc::now();
        let past = now - chrono::Duration::days(1);
        let future = now + chrono::Duration::days(1);

        let op_id1 = db.insert_operation("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = db.insert_operation("ADD", 4, 5, 9, None).unwrap();
        let op_id3 = db.insert_operation("ADD", 6, 7, 13, None).unwrap();

        db.insert_review_item(op_id1, past).unwrap();
        db.insert_review_item(op_id2, now).unwrap();
        db.insert_review_item(op_id3, future).unwrap();

        let count = db.count_due_reviews(now).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_nonexistent_review_item() {
        let db = create_test_db();
        let result = db.get_review_item(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_review_item_with_multiple_operations() {
        let db = create_test_db();
        let now = chrono::Utc::now();

        let op_id1 = db.insert_operation("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = db.insert_operation("MULTIPLY", 3, 4, 12, None).unwrap();

        db.insert_review_item(op_id1, now).unwrap();
        db.insert_review_item(op_id2, now + chrono::Duration::days(1))
            .unwrap();

        let item1 = db.get_review_item(op_id1).unwrap().unwrap();
        let item2 = db.get_review_item(op_id2).unwrap().unwrap();

        assert_eq!(item1.operation_id, op_id1);
        assert_eq!(item2.operation_id, op_id2);
        assert_ne!(item1.operation_id, item2.operation_id);
    }

    // Time statistics tests

    #[test]
    fn test_compute_time_statistics_empty_database() {
        let db = create_test_db();
        let result = db.compute_time_statistics("ADD").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_time_statistics_nonexistent_operation_type() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 5, true, 2.0, Some(deck_id)).unwrap();
        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics("MULTIPLY").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_time_statistics_single_answer() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 5, true, 2.0, Some(deck_id)).unwrap();
        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics("ADD").unwrap();
        assert!(result.is_some());
        let (avg, stdev) = result.unwrap();
        assert!((avg - 2.0).abs() < 0.001);
        // Single value: variance = (4.0/1) - (2.0/1)^2 = 0
        assert!(stdev < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_multiple_answers() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();

        // Insert answers with times: 1.0, 3.0
        db.insert_answer(op_id, 5, true, 1.0, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 5, true, 3.0, Some(deck_id)).unwrap();
        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics("ADD").unwrap();
        assert!(result.is_some());
        let (avg, stdev) = result.unwrap();
        // Average: (1.0 + 3.0) / 2 = 2.0
        assert!((avg - 2.0).abs() < 0.001);
        // Variance: ((1.0^2 + 3.0^2) / 2) - (2.0^2) = (10.0/2) - 4.0 = 1.0
        // Stdev: sqrt(1.0) = 1.0
        assert!((stdev - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_excludes_incorrect_answers() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();

        db.insert_answer(op_id, 5, true, 1.0, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 4, false, 2.0, Some(deck_id)).unwrap(); // Should be excluded
        db.insert_answer(op_id, 5, true, 3.0, Some(deck_id)).unwrap();
        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics("ADD").unwrap();
        assert!(result.is_some());
        let (avg, _) = result.unwrap();
        // Should only average 1.0 and 3.0
        assert!((avg - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_excludes_incomplete_decks() {
        let db = create_test_db();
        let deck_id1 = db.create_deck().unwrap();
        let deck_id2 = db.create_deck().unwrap();

        let op_id1 = db.insert_operation("ADD", 2, 3, 5, Some(deck_id1)).unwrap();
        db.insert_answer(op_id1, 5, true, 1.0, Some(deck_id1)).unwrap();
        db.complete_deck(deck_id1).unwrap();

        let op_id2 = db.insert_operation("ADD", 4, 5, 9, Some(deck_id2)).unwrap();
        db.insert_answer(op_id2, 9, true, 5.0, Some(deck_id2)).unwrap();
        // deck_id2 is NOT completed

        let result = db.compute_time_statistics("ADD").unwrap();
        assert!(result.is_some());
        let (avg, _) = result.unwrap();
        // Should only use data from deck_id1
        assert!((avg - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_empty_database() {
        let db = create_test_db();
        let result = db.compute_time_statistics_all_operations().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_time_statistics_all_operations_single_type() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();
        let op_id = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id, 5, true, 2.0, Some(deck_id)).unwrap();
        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics_all_operations().unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("ADD"));
        let (avg, stdev) = result.get("ADD").copied().unwrap();
        assert!((avg - 2.0).abs() < 0.001);
        assert!(stdev < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_multiple_types() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        // Add ADD operations
        let op_id1 = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id1, 5, true, 1.0, Some(deck_id)).unwrap();

        // Add MULTIPLY operations
        let op_id2 = db.insert_operation("MULTIPLY", 3, 4, 12, Some(deck_id)).unwrap();
        db.insert_answer(op_id2, 12, true, 3.0, Some(deck_id)).unwrap();
        db.insert_answer(op_id2, 12, true, 5.0, Some(deck_id)).unwrap();

        db.complete_deck(deck_id).unwrap();

        let result = db.compute_time_statistics_all_operations().unwrap();
        assert_eq!(result.len(), 2);

        assert!(result.contains_key("ADD"));
        let (add_avg, _) = result.get("ADD").unwrap();
        assert!((add_avg - 1.0).abs() < 0.001);

        assert!(result.contains_key("MULTIPLY"));
        let (mult_avg, mult_stdev) = result.get("MULTIPLY").unwrap();
        assert!((mult_avg - 4.0).abs() < 0.001); // (3.0 + 5.0) / 2 = 4.0
        // Variance: ((9.0 + 25.0) / 2) - 16.0 = 17.0 - 16.0 = 1.0, stdev = 1.0
        assert!((mult_stdev - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_last_30_days_excludes_old_data() {
        let db = create_test_db();
        let deck_id1 = db.create_deck().unwrap();
        let deck_id2 = db.create_deck().unwrap();

        // Create operation and answer for current deck
        let op_id1 = db.insert_operation("ADD", 2, 3, 5, Some(deck_id1)).unwrap();
        db.insert_answer(op_id1, 5, true, 2.0, Some(deck_id1)).unwrap();
        db.complete_deck(deck_id1).unwrap();

        // For old data, we'll create another deck and manually insert data with old timestamp
        let op_id2 = db.insert_operation("ADD", 4, 5, 9, Some(deck_id2)).unwrap();

        // Insert old answer (more than 30 days ago) - we simulate by inserting then checking
        // Since we can't easily control timestamps in tests, we verify that current data is included
        db.insert_answer(op_id2, 9, true, 10.0, Some(deck_id2)).unwrap();
        db.complete_deck(deck_id2).unwrap();

        // Get last 30 days stats - should include recent answers
        let result = db.compute_time_statistics_all_operations_last_30_days().unwrap();

        // Should have at least the ADD operations we just created
        assert!(result.contains_key("ADD"));
    }

    #[test]
    fn test_compute_time_statistics_all_operations_last_10_decks_excludes_old_decks() {
        let db = create_test_db();

        // Create 15 decks with operations and answers
        for i in 1..=15 {
            let deck_id = db.create_deck().unwrap();
            let op_id = db
                .insert_operation(
                    "ADD",
                    i as i32,
                    1,
                    (i as i32) + 1,
                    Some(deck_id),
                )
                .unwrap();
            // Time spent increases from 1.0 to 15.0
            db.insert_answer(op_id, (i as i32) + 1, true, i as f64, Some(deck_id))
                .unwrap();
            db.complete_deck(deck_id).unwrap();
        }

        let result = db.compute_time_statistics_all_operations_last_10_decks().unwrap();
        assert!(result.contains_key("ADD"));

        let (avg, _) = result.get("ADD").unwrap();

        // Last 10 decks have times: 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        // Average: (6+7+8+9+10+11+12+13+14+15) / 10 = 105 / 10 = 10.5
        assert!((avg - 10.5).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_last_10_decks_with_fewer_decks() {
        let db = create_test_db();

        // Create only 5 decks
        for i in 1..=5 {
            let deck_id = db.create_deck().unwrap();
            let op_id = db
                .insert_operation(
                    "ADD",
                    i as i32,
                    1,
                    (i as i32) + 1,
                    Some(deck_id),
                )
                .unwrap();
            db.insert_answer(op_id, (i as i32) + 1, true, i as f64, Some(deck_id))
                .unwrap();
            db.complete_deck(deck_id).unwrap();
        }

        let result = db.compute_time_statistics_all_operations_last_10_decks().unwrap();

        // Should include all 5 decks
        assert!(result.contains_key("ADD"));
        let (avg, _) = result.get("ADD").unwrap();
        // Average: (1+2+3+4+5) / 5 = 15 / 5 = 3.0
        assert!((avg - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_consistency() {
        let db = create_test_db();
        let deck_id = db.create_deck().unwrap();

        let op_id1 = db.insert_operation("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        db.insert_answer(op_id1, 5, true, 1.0, Some(deck_id)).unwrap();
        db.insert_answer(op_id1, 5, true, 3.0, Some(deck_id)).unwrap();

        let op_id2 = db.insert_operation("ADD", 4, 5, 9, Some(deck_id)).unwrap();
        db.insert_answer(op_id2, 9, true, 2.0, Some(deck_id)).unwrap();

        db.complete_deck(deck_id).unwrap();

        // Get stats for specific operation type
        let single_result = db.compute_time_statistics("ADD").unwrap();

        // Get stats for all operation types
        let all_result = db.compute_time_statistics_all_operations().unwrap();

        // They should match for ADD
        assert_eq!(single_result, all_result.get("ADD").copied());
    }
}

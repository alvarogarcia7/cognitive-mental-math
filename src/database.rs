use crate::deck::{Deck, DeckStatus, DeckSummary};
use crate::row_factories::{DeckRowFactory, ReviewItemRowFactory};
use crate::spaced_repetition::ReviewItem;
use crate::time_format::format_time_difference;
use chrono::{DateTime, Utc};
use log::debug;
use rusqlite::{Connection, Result, params};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                operand1 INTEGER NOT NULL,
                operand2 INTEGER NOT NULL,
                result INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS answers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id INTEGER NOT NULL,
                user_answer INTEGER NOT NULL,
                is_correct INTEGER NOT NULL,
                time_spent_seconds REAL NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (operation_id) REFERENCES operations(id)
            )",
            [],
        )?;

        // Create decks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS decks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                completed_at DATETIME,
                status TEXT NOT NULL DEFAULT 'in_progress',
                total_questions INTEGER NOT NULL DEFAULT 0,
                correct_answers INTEGER NOT NULL DEFAULT 0,
                incorrect_answers INTEGER NOT NULL DEFAULT 0,
                total_time_seconds REAL NOT NULL DEFAULT 0.0,
                average_time_seconds REAL,
                accuracy_percentage REAL
            )",
            [],
        )?;

        // Add deck_id columns using ALTER TABLE (safe for existing databases)
        // SQLite doesn't have ALTER TABLE IF NOT EXISTS for columns, so we need to check
        let has_deck_id_in_operations: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('operations') WHERE name='deck_id'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !has_deck_id_in_operations {
            conn.execute(
                "ALTER TABLE operations ADD COLUMN deck_id INTEGER REFERENCES decks(id)",
                [],
            )?;
        }

        let has_deck_id_in_answers: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('answers') WHERE name='deck_id'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !has_deck_id_in_answers {
            conn.execute(
                "ALTER TABLE answers ADD COLUMN deck_id INTEGER REFERENCES decks(id)",
                [],
            )?;
        }

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_deck_operations ON operations(deck_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_deck_answers ON answers(deck_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_deck_status ON decks(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_deck_created ON decks(created_at DESC)",
            [],
        )?;

        // Create review_items table for spaced repetition
        conn.execute(
            "CREATE TABLE IF NOT EXISTS review_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id INTEGER UNIQUE NOT NULL,
                repetitions INTEGER NOT NULL DEFAULT 0,
                interval INTEGER NOT NULL DEFAULT 0,
                ease_factor REAL NOT NULL DEFAULT 2.5,
                next_review_date TEXT NOT NULL,
                last_reviewed_date TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (operation_id) REFERENCES operations(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_next_review ON review_items(next_review_date)",
            [],
        )?;

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
        self.conn.execute(
            "INSERT INTO decks (status) VALUES (?1)",
            [DeckStatus::InProgress.as_str()],
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
        self.conn.execute(
            "UPDATE decks SET status = ?1, completed_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![DeckStatus::Completed.as_str(), deck_id],
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
}

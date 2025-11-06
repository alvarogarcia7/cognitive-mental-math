use crate::deck::{Deck, DeckStatus, DeckSummary};
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
            Ok(Some(Deck {
                id: row.get(0)?,
                created_at: row.get(1)?,
                completed_at: row.get(2)?,
                status: DeckStatus::from(&row.get::<_, String>(3)?)
                    .unwrap_or(DeckStatus::InProgress),
                total_questions: row.get(4)?,
                correct_answers: row.get(5)?,
                incorrect_answers: row.get(6)?,
                total_time_seconds: row.get(7)?,
                average_time_seconds: row.get(8)?,
                accuracy_percentage: row.get(9)?,
            }))
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

        let rows = stmt.query_map([limit], |row| {
            Ok(Deck {
                id: row.get(0)?,
                created_at: row.get(1)?,
                completed_at: row.get(2)?,
                status: DeckStatus::from(&row.get::<_, String>(3)?)
                    .unwrap_or(DeckStatus::InProgress),
                total_questions: row.get(4)?,
                correct_answers: row.get(5)?,
                incorrect_answers: row.get(6)?,
                total_time_seconds: row.get(7)?,
                average_time_seconds: row.get(8)?,
                accuracy_percentage: row.get(9)?,
            })
        })?;

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
}

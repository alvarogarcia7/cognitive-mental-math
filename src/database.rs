use rusqlite::{Connection, Result};

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

        Ok(Database { conn })
    }

    pub fn insert_operation(
        &self,
        operation_type: &str,
        operand1: i32,
        operand2: i32,
        result: i32,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO operations (operation_type, operand1, operand2, result) VALUES (?1, ?2, ?3, ?4)",
            [operation_type, &operand1.to_string(), &operand2.to_string(), &result.to_string()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_answer(
        &self,
        operation_id: i64,
        user_answer: i32,
        is_correct: bool,
        time_spent_seconds: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO answers (operation_id, user_answer, is_correct, time_spent_seconds) VALUES (?1, ?2, ?3, ?4)",
            [
                &operation_id.to_string(),
                &user_answer.to_string(),
                &(is_correct as i32).to_string(),
                &time_spent_seconds.to_string(),
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
        let op_id = db.insert_operation("ADD", 5, 3, 8).unwrap();
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
        let id1 = db.insert_operation("ADD", 10, 20, 30).unwrap();
        let id2 = db.insert_operation("MULTIPLY", 4, 5, 20).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(db.count_operations().unwrap(), 2);
    }

    #[test]
    fn test_insert_answer() {
        let db = create_test_db();
        let op_id = db.insert_operation("MULTIPLY", 7, 8, 56).unwrap();

        db.insert_answer(op_id, 56, true, 2.5).unwrap();
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
        let op_id = db.insert_operation("ADD", 15, 25, 40).unwrap();

        db.insert_answer(op_id, 35, false, 3.2).unwrap();

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
        let op_id = db.insert_operation("ADD", 1, 2, 3).unwrap();

        // Insert multiple answers (simulating retries)
        db.insert_answer(op_id, 4, false, 1.0).unwrap();
        db.insert_answer(op_id, 3, true, 2.0).unwrap();

        assert_eq!(db.count_answers().unwrap(), 2);
    }

    #[test]
    fn test_answer_references_operation() {
        let db = create_test_db();
        let op_id = db.insert_operation("MULTIPLY", 3, 4, 12).unwrap();
        db.insert_answer(op_id, 12, true, 1.5).unwrap();

        let answer = db.get_answer(1).unwrap().unwrap();
        let operation = db.get_operation(answer.operation_id).unwrap().unwrap();

        assert_eq!(operation.operand1, 3);
        assert_eq!(operation.operand2, 4);
        assert_eq!(operation.result, 12);
    }
}

use rusqlite::{Connection, Result, params};

#[derive(Debug, PartialEq)]
pub struct AnswerRecord {
    pub id: i64,
    pub operation_id: i64,
    pub user_answer: i32,
    pub is_correct: bool,
    pub time_spent_seconds: f64,
}

pub struct AnswersRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AnswersRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        AnswersRepository { conn }
    }

    pub fn insert(
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

    pub fn get(&self, answer_id: i64) -> Result<Option<AnswerRecord>> {
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

    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM answers", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::init_connection;
    use crate::database::operations::OperationsRepository;

    fn create_test_db() -> Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_insert_answer() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);

        let op_id = ops_repo.insert("MULTIPLY", 7, 8, 56, None).unwrap();
        answers_repo.insert(op_id, 56, true, 2.5, None).unwrap();
        assert_eq!(answers_repo.count().unwrap(), 1);

        let answer = answers_repo.get(1).unwrap().unwrap();
        assert_eq!(answer.operation_id, op_id);
        assert_eq!(answer.user_answer, 56);
        assert!(answer.is_correct);
        assert_eq!(answer.time_spent_seconds, 2.5);
    }

    #[test]
    fn test_insert_answer_incorrect() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);

        let op_id = ops_repo.insert("ADD", 15, 25, 40, None).unwrap();
        answers_repo.insert(op_id, 35, false, 3.2, None).unwrap();

        let answer = answers_repo.get(1).unwrap().unwrap();
        assert_eq!(answer.user_answer, 35);
        assert!(!answer.is_correct);
        assert_eq!(answer.time_spent_seconds, 3.2);
    }

    #[test]
    fn test_get_nonexistent_answer() {
        let conn = create_test_db();
        let answers_repo = AnswersRepository::new(&conn);
        let result = answers_repo.get(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_answers_for_operation() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);

        let op_id = ops_repo.insert("ADD", 1, 2, 3, None).unwrap();

        // Insert multiple answers (simulating retries)
        answers_repo.insert(op_id, 4, false, 1.0, None).unwrap();
        answers_repo.insert(op_id, 3, true, 2.0, None).unwrap();

        assert_eq!(answers_repo.count().unwrap(), 2);
    }

    #[test]
    fn test_answer_references_operation() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);

        let op_id = ops_repo.insert("MULTIPLY", 3, 4, 12, None).unwrap();
        answers_repo.insert(op_id, 12, true, 1.5, None).unwrap();

        let answer = answers_repo.get(1).unwrap().unwrap();
        let operation = ops_repo.get(answer.operation_id).unwrap().unwrap();

        assert_eq!(operation.operand1, 3);
        assert_eq!(operation.operand2, 4);
        assert_eq!(operation.result, 12);
    }
}

use rusqlite::{Connection, Result, params};

#[derive(Debug, PartialEq)]
pub struct OperationRecord {
    pub id: i64,
    pub operation_type: String,
    pub operand1: i32,
    pub operand2: i32,
    pub result: i32,
}

pub struct OperationsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> OperationsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        OperationsRepository { conn }
    }

    pub fn insert(
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

    pub fn get(&self, operation_id: i64) -> Result<Option<OperationRecord>> {
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

    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM operations", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::init_connection;

    fn create_test_db() -> Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_insert_operation() {
        let conn = create_test_db();
        let repo = OperationsRepository::new(&conn);
        let op_id = repo.insert("ADD", 5, 3, 8, None).unwrap();
        assert_eq!(op_id, 1);

        let op_record = repo.get(op_id).unwrap().unwrap();
        assert_eq!(op_record.operation_type, "ADD");
        assert_eq!(op_record.operand1, 5);
        assert_eq!(op_record.operand2, 3);
        assert_eq!(op_record.result, 8);
    }

    #[test]
    fn test_insert_multiple_operations() {
        let conn = create_test_db();
        let repo = OperationsRepository::new(&conn);
        let id1 = repo.insert("ADD", 10, 20, 30, None).unwrap();
        let id2 = repo.insert("MULTIPLY", 4, 5, 20, None).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_get_nonexistent_operation() {
        let conn = create_test_db();
        let repo = OperationsRepository::new(&conn);
        let result = repo.get(999).unwrap();
        assert!(result.is_none());
    }
}

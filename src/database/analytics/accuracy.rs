use rusqlite::Connection;
use rusqlite::Result;
use std::collections::HashMap;

// SQL WHERE clause constants for template method filters
const LAST_30_DAYS_WHERE: &str = "a.created_at >= datetime('now', '-30 days')";
const LAST_10_DECKS_WHERE: &str = r#"d.id IN (
    SELECT id FROM decks
    WHERE status = 'completed'
    ORDER BY completed_at DESC
    LIMIT 10
)"#;

pub struct AccuracyRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AccuracyRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        AccuracyRepository { conn }
    }

    /// Template method for computing accuracy statistics per operation type with custom WHERE clauses
    fn compute_all_operations_template(
        &self,
        additional_where: &str,
    ) -> Result<HashMap<String, (i64, i64, f64)>> {
        let mut query = r#"SELECT
                o.operation_type,
                COUNT(CASE WHEN a.is_correct = 1 THEN 1 END) as correct_count,
                COUNT(a.id) as total_count,
                CAST(COUNT(CASE WHEN a.is_correct = 1 THEN 1 END) AS FLOAT) /
                COUNT(a.id) * 100.0 as accuracy_percentage
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE d.status = 'completed'"#
            .to_string();

        if !additional_where.is_empty() {
            query.push_str("\n            AND ");
            query.push_str(additional_where);
        }

        query.push_str(
            r#"
            GROUP BY o.operation_type
            ORDER BY o.operation_type"#,
        );

        let mut stmt = self.conn.prepare(&query)?;
        let mut result = HashMap::new();
        let rows = stmt.query_map([], |row| {
            let op_type: String = row.get(0)?;
            let correct_count: i64 = row.get(1)?;
            let total_count: i64 = row.get(2)?;
            let accuracy: f64 = row.get(3)?;
            Ok((op_type, (correct_count, total_count, accuracy)))
        })?;

        for row in rows {
            let (op_type, stats) = row?;
            result.insert(op_type, stats);
        }

        Ok(result)
    }

    /// Template method for computing total accuracy with custom WHERE clauses
    fn compute_total_accuracy_template(&self, additional_where: &str) -> Result<(i64, i64, f64)> {
        let mut query = r#"SELECT
                COUNT(CASE WHEN a.is_correct = 1 THEN 1 END) as correct_count,
                COUNT(a.id) as total_count,
                CAST(COUNT(CASE WHEN a.is_correct = 1 THEN 1 END) AS FLOAT) /
                COUNT(a.id) * 100.0 as accuracy_percentage
            FROM answers a
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE d.status = 'completed'"#
            .to_string();

        if !additional_where.is_empty() {
            query.push_str("\n            AND ");
            query.push_str(additional_where);
        }

        let mut stmt = self.conn.prepare(&query)?;
        let result = stmt.query_row([], |row| {
            let correct_count: i64 = row.get(0)?;
            let total_count: i64 = row.get(1)?;
            let accuracy: f64 = row.get(2)?;
            Ok((correct_count, total_count, accuracy))
        })?;

        Ok(result)
    }

    /// Compute accuracy statistics for all operation types (global)
    /// Returns a map of operation_type -> (correct_count, total_count, accuracy_percentage)
    pub fn all_operations(&self) -> Result<HashMap<String, (i64, i64, f64)>> {
        self.compute_all_operations_template("")
    }

    /// Compute total accuracy for all operations (global)
    /// Returns (correct_count, total_count, accuracy_percentage)
    pub fn total_accuracy(&self) -> Result<(i64, i64, f64)> {
        self.compute_total_accuracy_template("")
    }

    /// Compute accuracy statistics for all operation types in the last 30 days
    /// Returns a map of operation_type -> (correct_count, total_count, accuracy_percentage)
    pub fn all_operations_last_30_days(&self) -> Result<HashMap<String, (i64, i64, f64)>> {
        self.compute_all_operations_template(LAST_30_DAYS_WHERE)
    }

    /// Compute total accuracy for all operations in the last 30 days
    /// Returns (correct_count, total_count, accuracy_percentage)
    pub fn total_accuracy_last_30_days(&self) -> Result<(i64, i64, f64)> {
        self.compute_total_accuracy_template(LAST_30_DAYS_WHERE)
    }

    /// Compute accuracy statistics for all operation types from the last 10 completed decks
    /// Returns a map of operation_type -> (correct_count, total_count, accuracy_percentage)
    pub fn all_operations_last_10_decks(&self) -> Result<HashMap<String, (i64, i64, f64)>> {
        self.compute_all_operations_template(LAST_10_DECKS_WHERE)
    }

    /// Compute total accuracy for all operations in the last 10 completed decks
    /// Returns (correct_count, total_count, accuracy_percentage)
    pub fn total_accuracy_last_10_decks(&self) -> Result<(i64, i64, f64)> {
        self.compute_total_accuracy_template(LAST_10_DECKS_WHERE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::init_connection;
    use crate::database::answers::AnswersRepository;
    use crate::database::decks::DecksRepository;
    use crate::database::operations::OperationsRepository;

    fn create_test_db() -> rusqlite::Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_compute_accuracy_all_operations_empty_database() {
        let conn = create_test_db();
        let repo = AccuracyRepository::new(&conn);
        let result = repo.all_operations().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_accuracy_all_operations_single_type() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);
        let decks_repo = DecksRepository::new(&conn, Box::new(|| chrono::Utc::now()));
        let accuracy_repo = AccuracyRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 2.0, Some(deck_id))
            .unwrap();
        decks_repo.complete(deck_id).unwrap();

        let result = accuracy_repo.all_operations().unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("ADD"));
        let (correct, total, accuracy) = result.get("ADD").unwrap();
        assert_eq!(*correct, 1);
        assert_eq!(*total, 1);
        assert!((accuracy - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_accuracy_all_operations_multiple_types() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);
        let decks_repo = DecksRepository::new(&conn, Box::new(|| chrono::Utc::now()));
        let accuracy_repo = AccuracyRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();

        // Add ADD operations (all correct)
        let op_id1 = ops_repo
            .insert("ADD", 2, 3, 5, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id1, 5, true, 1.0, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id1, 5, true, 1.5, Some(deck_id))
            .unwrap();

        // Add MULTIPLY operations (1 correct, 1 incorrect)
        let op_id2 = ops_repo
            .insert("MULTIPLY", 3, 4, 12, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id2, 12, true, 3.0, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id2, 10, false, 2.0, Some(deck_id))
            .unwrap();

        decks_repo.complete(deck_id).unwrap();

        let result = accuracy_repo.all_operations().unwrap();
        assert_eq!(result.len(), 2);

        // ADD: 2/2 = 100%
        let (add_correct, add_total, add_accuracy) = result.get("ADD").unwrap();
        assert_eq!(*add_correct, 2);
        assert_eq!(*add_total, 2);
        assert!((add_accuracy - 100.0).abs() < 0.001);

        // MULTIPLY: 1/2 = 50%
        let (mult_correct, mult_total, mult_accuracy) = result.get("MULTIPLY").unwrap();
        assert_eq!(*mult_correct, 1);
        assert_eq!(*mult_total, 2);
        assert!((mult_accuracy - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_total_accuracy_single_deck() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);
        let decks_repo = DecksRepository::new(&conn, Box::new(|| chrono::Utc::now()));
        let accuracy_repo = AccuracyRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 2.0, Some(deck_id))
            .unwrap();
        decks_repo.complete(deck_id).unwrap();

        let (correct, total, accuracy) = accuracy_repo.total_accuracy().unwrap();
        assert_eq!(correct, 1);
        assert_eq!(total, 1);
        assert!((accuracy - 100.0).abs() < 0.001);
    }
}

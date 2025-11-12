use crate::spaced_repetition::AnswerTimedEvaluator;
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

pub struct TimeStatisticsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TimeStatisticsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        TimeStatisticsRepository { conn }
    }

    /// Compute time statistics for correct answers of a specific operation type
    ///
    /// Returns (average_time, standard_deviation) for correct answers of the given operation type
    /// Only considers correct answers from completed decks
    pub fn for_operation_type(&self, operation_type: &str) -> Result<Option<AnswerTimedEvaluator>> {
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

                Ok(Some(AnswerTimedEvaluator::new(average, stdev)))
            }
        })?;

        Ok(result)
    }

    /// Helper function to extract and calculate statistics from a row
    fn extract_row_statistics(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, AnswerTimedEvaluator)> {
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

        Ok((op_type, AnswerTimedEvaluator::new(average, stdev)))
    }

    /// Helper function to process statistics query and build HashMap
    fn process_statistics_query(
        stmt: &mut rusqlite::Statement,
    ) -> Result<HashMap<String, AnswerTimedEvaluator>> {
        let mut result = HashMap::new();
        let rows = stmt.query_map([], Self::extract_row_statistics)?;
        for row in rows {
            let (op_type, evaluator) = row?;
            result.insert(op_type, evaluator);
        }
        Ok(result)
    }

    /// Template method for computing time statistics with custom WHERE clauses
    /// Accepts an optional additional WHERE condition to filter results
    fn compute_all_operations_template(
        &self,
        additional_where: &str,
    ) -> Result<HashMap<String, AnswerTimedEvaluator>> {
        let mut query = "SELECT
                o.operation_type,
                COUNT(a.time_spent_seconds) as count,
                AVG(a.time_spent_seconds) as average,
                SUM(a.time_spent_seconds * a.time_spent_seconds) as sum_squares,
                SUM(a.time_spent_seconds) as total_sum
            FROM answers a
            INNER JOIN operations o ON a.operation_id = o.id
            INNER JOIN decks d ON a.deck_id = d.id
            WHERE a.is_correct = 1
            AND d.status = 'completed'"
            .to_string();

        if !additional_where.is_empty() {
            query.push_str("\n            AND ");
            query.push_str(additional_where);
        }

        query.push_str(
            "
            GROUP BY o.operation_type
            ORDER BY o.operation_type
",
        );

        let mut stmt = self.conn.prepare(&query)?;
        Self::process_statistics_query(&mut stmt)
    }

    /// Compute time statistics for all operation types (global)
    /// Returns a map of operation_type -> AnswerTimedEvaluator
    pub fn all_operations(&self) -> Result<HashMap<String, AnswerTimedEvaluator>> {
        self.compute_all_operations_template("")
    }

    /// Compute time statistics for all operation types in the last 30 days
    /// Returns a map of operation_type -> AnswerTimedEvaluator
    pub fn all_operations_last_30_days(&self) -> Result<HashMap<String, AnswerTimedEvaluator>> {
        self.compute_all_operations_template(&format!("AND {}", LAST_30_DAYS_WHERE))
    }

    /// Compute time statistics for all operation types from the last 10 completed decks
    /// Returns a map of operation_type -> AnswerTimedEvaluator
    pub fn all_operations_last_10_decks(&self) -> Result<HashMap<String, AnswerTimedEvaluator>> {
        self.compute_all_operations_template(&format!("AND {}", LAST_10_DECKS_WHERE))
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
    fn test_compute_time_statistics_empty_database() {
        let conn = create_test_db();
        let repo = TimeStatisticsRepository::new(&conn);
        let result = repo.for_operation_type("ADD").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_time_statistics_single_answer() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);
        let decks_repo = DecksRepository::new(&conn, Box::new(|| chrono::Utc::now()));
        let time_stats_repo = TimeStatisticsRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 2.0, Some(deck_id))
            .unwrap();
        decks_repo.complete(deck_id).unwrap();

        let result = time_stats_repo.for_operation_type("ADD").unwrap();
        assert!(result.is_some());
        let eval = result.unwrap();
        assert!((eval.average - 2.0).abs() < 0.001);
        // Single value: variance = (4.0/1) - (2.0/1)^2 = 0
        assert!(eval.standard_deviation < 0.001);
    }

    #[test]
    fn test_compute_time_statistics_all_operations_empty_database() {
        let conn = create_test_db();
        let repo = TimeStatisticsRepository::new(&conn);
        let result = repo.all_operations().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_time_statistics_all_operations_multiple_types() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let answers_repo = AnswersRepository::new(&conn);
        let decks_repo = DecksRepository::new(&conn, Box::new(|| chrono::Utc::now()));
        let time_stats_repo = TimeStatisticsRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();

        // Add ADD operations
        let op_id1 = ops_repo
            .insert("ADD", 2, 3, 5, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id1, 5, true, 1.0, Some(deck_id))
            .unwrap();

        // Add MULTIPLY operations
        let op_id2 = ops_repo
            .insert("MULTIPLY", 3, 4, 12, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id2, 12, true, 3.0, Some(deck_id))
            .unwrap();
        answers_repo
            .insert(op_id2, 12, true, 5.0, Some(deck_id))
            .unwrap();

        decks_repo.complete(deck_id).unwrap();

        let result = time_stats_repo.all_operations().unwrap();
        assert_eq!(result.len(), 2);

        assert!(result.contains_key("ADD"));
        let add_eval = result.get("ADD").unwrap();
        assert!((add_eval.average - 1.0).abs() < 0.001);

        assert!(result.contains_key("MULTIPLY"));
        let mult_eval = result.get("MULTIPLY").unwrap();
        assert!((mult_eval.average - 4.0).abs() < 0.001); // (3.0 + 5.0) / 2 = 4.0
    }
}

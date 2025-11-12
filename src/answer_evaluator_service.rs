use crate::database::analytics::TimeStatisticsRepository;
use crate::spaced_repetition::AnswerTimedEvaluator;
use rusqlite::Connection;

/// Service for evaluating answer performance based on historical timing data
pub struct AnswerEvaluatorService<'a> {
    conn: &'a Connection,
}

impl<'a> AnswerEvaluatorService<'a> {
    /// Create a new AnswerEvaluatorService
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get or create an AnswerTimedEvaluator for the given operation type
    ///
    /// Retrieves historical timing statistics from the database for the operation type.
    /// Falls back to default values (average: 3.0s, stdev: 2.0s) if no historical data exists.
    pub fn get_evaluator(&self, operation_type: &str) -> AnswerTimedEvaluator {
        TimeStatisticsRepository::new(self.conn)
            .for_operation_type(operation_type)
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                // Fallback if no historical data exists
                AnswerTimedEvaluator::new(3.0, 2.0)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{AnswersRepository, Database, OperationsRepository};
    use std::sync::Arc;

    #[test]
    fn test_get_evaluator_with_historical_data() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(&db.conn);

        // Insert some operations and answers to create historical data
        let repo_operations = OperationsRepository::new(&db.conn);
        let operation_id = repo_operations.insert("addition", 5, 3, 8, None).unwrap();

        // Insert several answers with known times
        let repo_answers = AnswersRepository::new(&db.conn);
        let _ = repo_answers.insert(operation_id, 8, true, 1.0, None);
        let _ = repo_answers.insert(operation_id, 8, true, 1.5, None);
        let _ = repo_answers.insert(operation_id, 8, true, 2.0, None);

        let evaluator = service.get_evaluator("addition");

        // Verify that the evaluator was created and has valid values
        assert!(evaluator.average > 0.0);
        assert!(evaluator.standard_deviation >= 0.0);
    }

    #[test]
    fn test_get_evaluator_without_historical_data() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(&db.conn);

        let evaluator = service.get_evaluator("multiplication");

        // Should return default fallback values
        assert_eq!(evaluator.average, 3.0);
        assert_eq!(evaluator.standard_deviation, 2.0);
    }

    #[test]
    fn test_get_evaluator_different_operation_types() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(&db.conn);

        // Test that different operation types can have different statistics
        let eval_add = service.get_evaluator("addition");
        let eval_sub = service.get_evaluator("subtraction");

        // Both should return fallback since no data exists
        assert_eq!(eval_add.average, 3.0);
        assert_eq!(eval_sub.average, 3.0);
    }

    #[test]
    fn test_get_evaluator_fallback_values() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(&db.conn);

        let evaluator = service.get_evaluator("division");

        // Verify fallback values
        assert_eq!(evaluator.average, 3.0);
        assert_eq!(evaluator.standard_deviation, 2.0);
    }
}

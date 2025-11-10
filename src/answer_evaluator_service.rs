use crate::database::Database;
use crate::spaced_repetition::AnswerTimedEvaluator;
use std::sync::Arc;

/// Service for evaluating answer performance based on historical timing data
pub struct AnswerEvaluatorService {
    db: Arc<Database>,
}

impl AnswerEvaluatorService {
    /// Create a new AnswerEvaluatorService
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get or create an AnswerTimedEvaluator for the given operation type
    ///
    /// Retrieves historical timing statistics from the database for the operation type.
    /// Falls back to default values (average: 3.0s, stdev: 2.0s) if no historical data exists.
    pub fn get_evaluator(&self, operation_type: &str) -> AnswerTimedEvaluator {
        self.db
            .compute_time_statistics(operation_type)
            .ok()
            .flatten()
            .map(|(avg, stdev)| AnswerTimedEvaluator::new(avg, stdev))
            .unwrap_or_else(|| {
                // Fallback if no historical data exists
                AnswerTimedEvaluator::new(3.0, 2.0)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[test]
    fn test_get_evaluator_with_historical_data() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(db.clone());

        // Insert some operations and answers to create historical data
        let operation_id = db.insert_operation("addition", 5, 3, 8, None).unwrap();

        // Insert several answers with known times
        let _ = db.insert_answer(operation_id, 8, true, 1.0, None);
        let _ = db.insert_answer(operation_id, 8, true, 1.5, None);
        let _ = db.insert_answer(operation_id, 8, true, 2.0, None);

        let evaluator = service.get_evaluator("addition");

        // Verify that the evaluator was created and has valid values
        assert!(evaluator.average > 0.0);
        assert!(evaluator.standard_deviation >= 0.0);
    }

    #[test]
    fn test_get_evaluator_without_historical_data() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(db);

        let evaluator = service.get_evaluator("multiplication");

        // Should return default fallback values
        assert_eq!(evaluator.average, 3.0);
        assert_eq!(evaluator.standard_deviation, 2.0);
    }

    #[test]
    fn test_get_evaluator_different_operation_types() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let service = AnswerEvaluatorService::new(db.clone());

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
        let service = AnswerEvaluatorService::new(db);

        let evaluator = service.get_evaluator("division");

        // Verify fallback values
        assert_eq!(evaluator.average, 3.0);
        assert_eq!(evaluator.standard_deviation, 2.0);
    }
}

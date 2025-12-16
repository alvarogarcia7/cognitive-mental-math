use chrono::{DateTime, Duration, Utc};
use sra::sm_2::{Quality, SM2};

/// Evaluates answer performance based on timing statistics for a specific operation type.
/// Uses historical data to assign quality grades (0, 3, 4, 5) based on how long
/// the user took to answer relative to typical performance for that operation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnswerTimedEvaluator {
    /// Average time spent on correct answers (in seconds)
    pub average: f64,
    /// Standard deviation of time spent on correct answers (in seconds)
    pub standard_deviation: f64,
}

impl AnswerTimedEvaluator {
    pub fn new(average: f64, standard_deviation: f64) -> Self {
        Self {
            average,
            standard_deviation,
        }
    }

    pub fn evaluate_performance(&self, is_correct: bool, time_spent: f64) -> Quality {
        if !is_correct {
            // Incorrect: complete blackout
            Quality::Grade0
        } else if time_spent >= self.average + (3.0 * self.standard_deviation) {
            // Correct but slow: serious difficulty recalling
            Quality::Grade3
        } else if time_spent >= self.average + self.standard_deviation {
            // Correct with some hesitation: recalled with hesitation
            Quality::Grade4
        } else {
            // Fast and correct: perfect response
            Quality::Grade5
        }
    }
}

/// Represents a single item scheduled for spaced repetition review
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewItem {
    pub id: Option<i64>,
    pub operation_id: i64,
    pub repetitions: i32,
    pub interval: i32,
    pub ease_factor: f32,
    pub next_review_date: DateTime<Utc>,
    pub last_reviewed_date: Option<DateTime<Utc>>,
}

/// Wraps the SM-2 algorithm for convenient review scheduling
pub struct ReviewScheduler;

impl ReviewScheduler {
    /// Creates a new review scheduler with default SM-2 parameters
    pub fn new() -> Self {
        Self
    }

    /// Processes a review and returns updated scheduling parameters
    ///
    /// Returns: (repetitions, interval, ease_factor, next_review_date)
    pub fn process_review(
        &self,
        item: &ReviewItem,
        quality: Quality,
    ) -> (i32, i32, f32, DateTime<Utc>) {
        // Create a new SM2 with current item parameters
        let mut sm2 = SM2::new();
        sm2 = sm2
            .set_repetitions(item.repetitions as usize)
            .set_interval(item.interval as usize)
            .set_ease_factor(item.ease_factor);

        // Apply the review
        let updated_sm2 = sm2.review(quality);

        let next_review_date = Utc::now() + Duration::days(updated_sm2.interval() as i64);

        (
            updated_sm2.repetitions() as i32,
            updated_sm2.interval() as i32,
            updated_sm2.ease_factor(),
            next_review_date,
        )
    }
}

impl Default for ReviewScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_mock_stats() -> AnswerTimedEvaluator {
        AnswerTimedEvaluator::new(3.0, 2.0)
    }

    #[test]
    fn test_performance_to_quality_incorrect() {
        let stats = AnswerTimedEvaluator::new(3.0, 2.0);
        let quality = stats.evaluate_performance(false, 1.0);
        assert!(matches!(quality, Quality::Grade0));
    }

    #[test]
    fn test_performance_to_quality_correct_fast() {
        let quality = example_mock_stats().evaluate_performance(true, 2.0);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_at_average() {
        let stats = example_mock_stats();
        let quality = stats.evaluate_performance(true, 3.0);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_slightly_above_average() {
        let quality = example_mock_stats().evaluate_performance(true, 4.0);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_one_stdev_above_average() {
        let quality = example_mock_stats().evaluate_performance(true, 5.0);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_between_stdev1_and_stdev2() {
        let quality = example_mock_stats().evaluate_performance(true, 6.0);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_two_stdev_above_average() {
        let quality = example_mock_stats().evaluate_performance(true, 7.0);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_between_stdev2_and_stdev3() {
        let quality = example_mock_stats().evaluate_performance(true, 8.0);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_three_stdev_above_average() {
        let quality = example_mock_stats().evaluate_performance(true, 9.0);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_very_slow() {
        let quality = example_mock_stats().evaluate_performance(true, 15.0);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_review_scheduler_creation() {
        let _scheduler = ReviewScheduler::new();
        let _default_scheduler = ReviewScheduler::default();
        // Just verify they can be created
        assert!(true);
    }

    #[test]
    fn test_review_scheduler_process_first_review_perfect() {
        let scheduler = ReviewScheduler::new();
        let item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: Utc::now(),
            last_reviewed_date: None,
        };

        let (reps, interval, ease, _next_date) = scheduler.process_review(&item, Quality::Grade5);

        // First review with perfect quality should schedule for 1 day
        assert_eq!(reps, 1);
        assert_eq!(interval, 1);
        assert!(ease >= 2.5); // Should not decrease
    }

    #[test]
    fn test_review_scheduler_process_difficult() {
        let scheduler = ReviewScheduler::new();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: fixed_date,
            last_reviewed_date: None,
        };

        let (reps, _interval, ease, _next_date) = scheduler.process_review(&item, Quality::Grade3);

        // Difficult response should decrease ease factor but still increment repetitions
        assert_eq!(reps, 1);
        assert!(ease < 2.5);
    }

    // ========== New comprehensive tests ==========

    #[test]
    fn test_performance_to_quality_grade0_always_incorrect() {
        // Grade0 should always be returned for incorrect answers regardless of time
        assert!(matches!(
            example_mock_stats().evaluate_performance(false, 0.5),
            Quality::Grade0
        ));
        assert!(matches!(
            example_mock_stats().evaluate_performance(false, 100.0),
            Quality::Grade0
        ));
        assert!(matches!(
            example_mock_stats().evaluate_performance(false, 0.0),
            Quality::Grade0
        ));
    }

    #[test]
    fn test_review_scheduler_multiple_sequential_reviews() {
        let scheduler = ReviewScheduler::new();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        // First review - correct (Grade5)
        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: fixed_date,
            last_reviewed_date: None,
        };

        let (reps1, interval1, ease1, _) = scheduler.process_review(&item, Quality::Grade5);
        assert_eq!(reps1, 1);
        assert_eq!(interval1, 1);
        assert!(ease1 >= 2.5);

        // Second review - still correct (Grade5)
        item.repetitions = reps1;
        item.interval = interval1;
        item.ease_factor = ease1;

        let (reps2, interval2, ease2, _) = scheduler.process_review(&item, Quality::Grade5);
        assert_eq!(reps2, 2);
        assert!(interval2 > interval1); // Interval should increase
        assert!(ease2 >= ease1); // Ease should not decrease
    }

    #[test]
    fn test_review_scheduler_ease_factor_degrades_with_poor_quality() {
        let scheduler = ReviewScheduler::new();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: fixed_date,
            last_reviewed_date: None,
        };

        // First review - correct (Grade5)
        let (reps1, interval1, ease1, _) = scheduler.process_review(&item, Quality::Grade5);

        // Second review - serious difficulty (Grade3)
        item.repetitions = reps1;
        item.interval = interval1;
        item.ease_factor = ease1;

        let (_reps2, _interval2, ease2, _) = scheduler.process_review(&item, Quality::Grade3);
        assert!(ease2 < ease1); // Ease should decrease with poor quality
        assert!(ease2 >= 1.3); // Ease factor has a minimum value (1.3)
    }

    #[test]
    fn test_review_scheduler_next_date_increases_with_interval() {
        let scheduler = ReviewScheduler::new();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: fixed_date,
            last_reviewed_date: None,
        };

        // First review
        let (reps1, interval1, ease1, next_date1) =
            scheduler.process_review(&item, Quality::Grade5);

        // Second review
        item.repetitions = reps1;
        item.interval = interval1;
        item.ease_factor = ease1;
        let (_reps2, _interval2, _ease2, next_date2) =
            scheduler.process_review(&item, Quality::Grade5);

        // Next review dates should be progressively further in the future
        assert!(next_date2 > next_date1);
        assert!(next_date1 > fixed_date);
    }

    #[test]
    fn test_performance_to_quality_boundary_conditions() {
        let stats = AnswerTimedEvaluator::new(10.0, 5.0);

        // Test exact thresholds
        // Grade5: < 15.0 (average + 1σ)
        assert!(matches!(
            stats.evaluate_performance(true, 14.99),
            Quality::Grade5
        ));
        // Grade4: >= 15.0 (average + 1σ), < 25.0 (average + 3σ)
        assert!(matches!(
            stats.evaluate_performance(true, 15.01),
            Quality::Grade4
        ));
        // Grade4: also at 20.0 (average + 2σ)
        assert!(matches!(
            stats.evaluate_performance(true, 20.0),
            Quality::Grade4
        ));
        // Grade3: >= 25.0 (average + 3σ)
        assert!(matches!(
            stats.evaluate_performance(true, 25.0),
            Quality::Grade3
        ));
        assert!(matches!(
            stats.evaluate_performance(true, 100.0),
            Quality::Grade3
        ));
    }

    #[test]
    fn test_review_item_equality_and_cloning() {
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let item1 = ReviewItem {
            id: Some(1),
            operation_id: 42,
            repetitions: 3,
            interval: 7,
            ease_factor: 2.6,
            next_review_date: fixed_date,
            last_reviewed_date: Some(fixed_date),
        };

        let item2 = item1.clone();
        assert_eq!(item1, item2);
        assert_eq!(item1.operation_id, item2.operation_id);
        assert_eq!(item1.repetitions, item2.repetitions);
        assert_eq!(item1.interval, item2.interval);
    }

    #[test]
    fn test_review_scheduler_default_is_new() {
        let default_scheduler = ReviewScheduler::default();
        let new_scheduler = ReviewScheduler::new();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: fixed_date,
            last_reviewed_date: None,
        };

        let (reps1, interval1, ease1, _) = default_scheduler.process_review(&item, Quality::Grade5);
        let (reps2, interval2, ease2, _) = new_scheduler.process_review(&item, Quality::Grade5);

        assert_eq!(reps1, reps2);
        assert_eq!(interval1, interval2);
        assert_eq!(ease1, ease2);
    }
}

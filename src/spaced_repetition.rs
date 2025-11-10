use chrono::{DateTime, Duration, Utc};
use sra::sm_2::{Quality, SM2};

/// Statistics about answer times for a specific operation type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeStatistics {
    /// Average time spent on correct answers (in seconds)
    pub average: f64,
    /// Standard deviation of time spent on correct answers (in seconds)
    pub standard_deviation: f64,
}

impl TimeStatistics {
    /// Create a new TimeStatistics
    pub fn new(average: f64, standard_deviation: f64) -> Self {
        Self {
            average,
            standard_deviation,
        }
    }

    /// Lower bound of typical performance (average time)
    /// Used as reference point for quality grading thresholds
    pub fn threshold_grade5(&self) -> f64 {
        self.average
    }

    /// Upper bound for Grade4 (After hesitation): average + 1σ
    /// Times >= this threshold but < threshold_grade3 are Grade4
    pub fn threshold_grade4(&self) -> f64 {
        self.average + self.standard_deviation
    }

    /// Upper bound for Grade3 (Serious difficulty): average + 2σ
    /// Times >= this threshold are Grade3 (slowest responses)
    pub fn threshold_grade3(&self) -> f64 {
        self.average + (2.0 * self.standard_deviation)
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

// /// Convert SM-2 quality grade to human-readable string
// pub fn quality_to_string(quality: Quality) -> String {
//     match quality {
//         Quality::Grade0 => "Grade0 (Incorrect)".to_string(),
//         Quality::Grade3 => "Grade3 (Serious difficulty)".to_string(),
//         Quality::Grade4 => "Grade4 (After hesitation)".to_string(),
//         Quality::Grade5 => "Grade5 (Perfect)".to_string(),
//         _ => "N/A".to_string(),
//     }
// }

/// Maps user performance (correctness and speed) to an SM-2 Quality rating
///
/// Uses statistical thresholds based on historical performance data for the operation type.
/// Grades are assigned based on how fast the answer was relative to typical performance:
///
/// Quality scale:
/// - Grade0: Incorrect (complete blackout)
/// - Grade3: Correct but slow (≥ average + 2 stdev)
/// - Grade4: Correct with some hesitation (≥ average + 1 stdev, but < average + 2 stdev)
/// - Grade5: Perfect and immediate recall (< average + 1 stdev)
pub fn performance_to_quality(
    is_correct: bool,
    time_spent: f64,
    stats: &TimeStatistics,
) -> Quality {
    if !is_correct {
        // Incorrect: complete blackout
        Quality::Grade0
    } else if time_spent >= stats.threshold_grade3() {
        Quality::Grade3
    } else if time_spent >= stats.threshold_grade4() {
        // Correct but slow: recalled with serious difficulty
        Quality::Grade4
    } else {
        // Fast and correct: perfect response
        Quality::Grade5
    }
}

/// Creates a new ReviewItem with initial SM-2 parameters
pub fn create_initial_review_item(operation_id: i64, is_correct: bool) -> ReviewItem {
    let next_review_date = if is_correct {
        // First review after 1 day if correct
        Utc::now() + Duration::days(1)
    } else {
        // Retry soon if incorrect (10 minutes)
        Utc::now() + Duration::minutes(10)
    };

    ReviewItem {
        id: None,
        operation_id,
        repetitions: 0,
        interval: 0,
        ease_factor: 2.5,
        next_review_date,
        last_reviewed_date: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_mock_stats() -> TimeStatistics {
        TimeStatistics::new(3.0, 2.0)
    }

    #[test]
    fn test_performance_to_quality_incorrect() {
        let stats = TimeStatistics::new(3.0, 2.0);
        let quality = performance_to_quality(false, 1.0, &stats);
        assert!(matches!(quality, Quality::Grade0));
    }

    #[test]
    fn test_performance_to_quality_correct_fast() {
        let stats = example_mock_stats();
        let quality = performance_to_quality(true, 2.0, &stats);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_at_average() {
        let stats = example_mock_stats();
        let quality = performance_to_quality(true, 3.0, &stats);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_slightly_above_average() {
        let stats = example_mock_stats();
        // 4.0 is below grade4 threshold (5.0), should be Grade5
        let quality = performance_to_quality(true, 4.0, &stats);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_one_stdev_above_average() {
        let stats = example_mock_stats();
        // Exactly at 5.0, which is grade4 threshold, should be Grade4
        let quality = performance_to_quality(true, 5.0, &stats);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_between_stdev1_and_stdev2() {
        let stats = example_mock_stats();
        // 6.0 is between 5.0 (grade4) and 7.0 (grade3), should be Grade4
        let quality = performance_to_quality(true, 6.0, &stats);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_two_stdev_above_average() {
        let stats = example_mock_stats();
        // Exactly at 7.0, which is grade3 threshold, should be Grade3
        let quality = performance_to_quality(true, 7.0, &stats);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_between_stdev2_and_stdev3() {
        let stats = example_mock_stats();
        // 8.0 is above 7.0 (grade3 threshold), should be Grade3
        let quality = performance_to_quality(true, 8.0, &stats);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_three_stdev_above_average() {
        let stats = example_mock_stats();
        // Exactly at 9.0 (average + 3σ), which is above grade3 threshold, should be Grade3
        let quality = performance_to_quality(true, 9.0, &stats);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_very_slow() {
        let stats = example_mock_stats();
        let quality = performance_to_quality(true, 15.0, &stats);
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
        let item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: Utc::now(),
            last_reviewed_date: None,
        };

        let (reps, _interval, ease, _next_date) = scheduler.process_review(&item, Quality::Grade3);

        // Difficult response should decrease ease factor but still increment repetitions
        assert_eq!(reps, 1);
        assert!(ease < 2.5);
    }

    #[test]
    fn test_create_initial_review_item_correct() {
        let item = create_initial_review_item(42, true);

        assert_eq!(item.operation_id, 42);
        assert_eq!(item.repetitions, 0);
        assert_eq!(item.interval, 0);
        assert_eq!(item.ease_factor, 2.5);
        assert!(item.id.is_none());

        // Next review should be about 1 day from now
        let duration = item.next_review_date - Utc::now();
        assert!(duration.num_hours() >= 23 && duration.num_hours() <= 25);
    }

    #[test]
    fn test_create_initial_review_item_incorrect() {
        let item = create_initial_review_item(42, false);

        assert_eq!(
            ReviewItem {
                id: None,
                operation_id: 42,
                repetitions: 0,
                interval: 0,
                ease_factor: 2.5,
                next_review_date: item.next_review_date,
                last_reviewed_date: None,
            },
            item
        );

        // Next review should be about 10 minutes from now
        let duration = item.next_review_date - Utc::now();
        assert!(duration.num_minutes() >= 9 && duration.num_minutes() <= 11);
    }

    // ========== New comprehensive tests ==========

    #[test]
    fn test_time_statistics_edge_cases() {
        // Test with zero standard deviation
        let stats_zero_stdev = TimeStatistics::new(5.0, 0.0);
        assert_eq!(stats_zero_stdev.threshold_grade5(), 5.0);
        assert_eq!(stats_zero_stdev.threshold_grade4(), 5.0);
        assert_eq!(stats_zero_stdev.threshold_grade3(), 5.0);

        // Test with very high standard deviation
        let stats_high_stdev = TimeStatistics::new(2.0, 10.0);
        assert_eq!(stats_high_stdev.threshold_grade5(), 2.0);
        assert_eq!(stats_high_stdev.threshold_grade4(), 12.0);
        assert_eq!(stats_high_stdev.threshold_grade3(), 22.0);

        // Test with very small values
        let stats_small = TimeStatistics::new(0.5, 0.1);
        assert_eq!(stats_small.threshold_grade5(), 0.5);
        assert_eq!(stats_small.threshold_grade4(), 0.6);
        assert_eq!(stats_small.threshold_grade3(), 0.7);
    }

    #[test]
    fn test_performance_to_quality_grade0_always_incorrect() {
        let stats = example_mock_stats();
        // Grade0 should always be returned for incorrect answers regardless of time
        assert!(matches!(
            performance_to_quality(false, 0.5, &stats),
            Quality::Grade0
        ));
        assert!(matches!(
            performance_to_quality(false, 100.0, &stats),
            Quality::Grade0
        ));
        assert!(matches!(
            performance_to_quality(false, 0.0, &stats),
            Quality::Grade0
        ));
    }

    #[test]
    fn test_review_scheduler_multiple_sequential_reviews() {
        let scheduler = ReviewScheduler::new();

        // First review - correct (Grade5)
        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: Utc::now(),
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

        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: Utc::now(),
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
        let now = Utc::now();

        let mut item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: now,
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
        assert!(next_date1 > now);
    }

    #[test]
    fn test_create_initial_review_item_different_operation_ids() {
        for op_id in 1..=10 {
            let item = create_initial_review_item(op_id, true);
            assert_eq!(item.operation_id, op_id);
        }
    }

    #[test]
    fn test_performance_to_quality_boundary_conditions() {
        let stats = TimeStatistics::new(10.0, 5.0);

        // Test exact thresholds
        // Grade5: < 15.0 (average + 1σ)
        assert!(matches!(
            performance_to_quality(true, 14.99, &stats),
            Quality::Grade5
        ));
        // Grade4: >= 15.0 (average + 1σ), < 20.0 (average + 2σ)
        assert!(matches!(
            performance_to_quality(true, 15.01, &stats),
            Quality::Grade4
        ));
        // Grade3: >= 20.0 (average + 2σ)
        assert!(matches!(
            performance_to_quality(true, 20.01, &stats),
            Quality::Grade3
        ));
        // Grade3: also for very slow times
        assert!(matches!(
            performance_to_quality(true, 100.0, &stats),
            Quality::Grade3
        ));
    }

    #[test]
    fn test_review_item_equality_and_cloning() {
        let item1 = ReviewItem {
            id: Some(1),
            operation_id: 42,
            repetitions: 3,
            interval: 7,
            ease_factor: 2.6,
            next_review_date: Utc::now(),
            last_reviewed_date: Some(Utc::now()),
        };

        let item2 = item1.clone();
        assert_eq!(item1, item2);
        assert_eq!(item1.operation_id, item2.operation_id);
        assert_eq!(item1.repetitions, item2.repetitions);
        assert_eq!(item1.interval, item2.interval);
    }

    #[test]
    fn test_create_initial_review_item_correct_vs_incorrect_timing() {
        let correct_item = create_initial_review_item(1, true);
        let incorrect_item = create_initial_review_item(2, false);

        let correct_duration = correct_item.next_review_date - Utc::now();
        let incorrect_duration = incorrect_item.next_review_date - Utc::now();

        // Correct should be scheduled much later than incorrect
        assert!(correct_duration.num_hours() > incorrect_duration.num_hours() * 10);

        // Correct should be ~1 day
        assert!(correct_duration.num_hours() >= 23 && correct_duration.num_hours() <= 25);

        // Incorrect should be ~10 minutes
        assert!(incorrect_duration.num_minutes() >= 9 && incorrect_duration.num_minutes() <= 11);
    }

    #[test]
    fn test_review_scheduler_default_is_new() {
        let default_scheduler = ReviewScheduler::default();
        let new_scheduler = ReviewScheduler::new();

        // Both should produce identical results
        let item = ReviewItem {
            id: Some(1),
            operation_id: 1,
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            next_review_date: Utc::now(),
            last_reviewed_date: None,
        };

        let (reps1, interval1, ease1, _) = default_scheduler.process_review(&item, Quality::Grade5);
        let (reps2, interval2, ease2, _) = new_scheduler.process_review(&item, Quality::Grade5);

        assert_eq!(reps1, reps2);
        assert_eq!(interval1, interval2);
        assert_eq!(ease1, ease2);
    }
}

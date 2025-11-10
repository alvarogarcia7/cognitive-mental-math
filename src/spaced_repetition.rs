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

    /// Get the threshold for Grade 5 (average or below)
    pub fn threshold_grade5(&self) -> f64 {
        self.average
    }

    /// Get the threshold for Grade 4 (average + 1 stdev)
    pub fn threshold_grade4(&self) -> f64 {
        self.average + self.standard_deviation
    }

    /// Get the threshold for Grade 3 (average + 2 stdev)
    pub fn threshold_grade3(&self) -> f64 {
        self.average + (2.0 * self.standard_deviation)
    }

    /// Get the threshold for Grade 2 (average + 3 stdev)
    pub fn threshold_grade2(&self) -> f64 {
        self.average + (3.0 * self.standard_deviation)
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

/// Maps user performance (correctness and speed) to an SM-2 Quality rating
///
/// Uses statistical thresholds based on historical performance data for the operation type.
/// Grades are assigned based on how fast the answer was relative to typical performance:
///
/// Quality scale (0-5):
/// - Grade0: Incorrect (complete blackout)
/// - Grade2: Correct but very slow (≥ average + 3 stdev)
/// - Grade3: Correct but slow (≥ average + 2 stdev, but < average + 3 stdev)
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
    } else if time_spent >= stats.threshold_grade2() {
        // Correct but very slow: recalled with difficulty
        Quality::Grade2
    } else if time_spent >= stats.threshold_grade3() {
        // Correct but slow: recalled with serious difficulty
        Quality::Grade3
    } else if time_spent >= stats.threshold_grade4() {
        // Correct with some thought: after hesitation
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

    /// Mock statistics for tests: average 3.0s with stdev 2.0s
    /// - Grade5 threshold: 3.0s
    /// - Grade4 threshold: 5.0s (3.0 + 1*2.0)
    /// - Grade3 threshold: 7.0s (3.0 + 2*2.0)
    /// - Grade2 threshold: 9.0s (3.0 + 3*3.0)
    fn example_mock_stats() -> TimeStatistics {
        TimeStatistics::new(3.0, 2.0)
    }

    #[test]
    fn test_time_statistics_thresholds() {
        let stats = example_mock_stats();
        assert_eq!(stats.threshold_grade5(), 3.0);
        assert_eq!(stats.threshold_grade4(), 5.0);
        assert_eq!(stats.threshold_grade3(), 7.0);
        assert_eq!(stats.threshold_grade2(), 9.0);
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
        // 8.0 is between 7.0 (grade3) and 9.0 (grade2), should be Grade3
        let quality = performance_to_quality(true, 8.0, &stats);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_three_stdev_above_average() {
        let stats = example_mock_stats();
        // Exactly at 9.0, which is grade2 threshold, should be Grade2
        let quality = performance_to_quality(true, 9.0, &stats);
        assert!(matches!(quality, Quality::Grade2));
    }

    #[test]
    fn test_performance_to_quality_correct_very_slow() {
        let stats = example_mock_stats();
        let quality = performance_to_quality(true, 15.0, &stats);
        assert!(matches!(quality, Quality::Grade2));
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
}

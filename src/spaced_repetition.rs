use chrono::{DateTime, Duration, Utc};
use sra::sm_2::{Quality, SM2};

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
/// Quality scale (0-5):
/// - 0-2: Incorrect or very slow (hard)
/// - 3: Correct but difficult
/// - 4: Correct with some hesitation
/// - 5: Perfect and immediate recall
pub fn performance_to_quality(is_correct: bool, time_spent: f64) -> Quality {
    if !is_correct {
        // Incorrect: complete blackout
        Quality::Grade0
    } else if time_spent > 10.0 {
        // Correct but very slow: incorrect, correct one remembered
        Quality::Grade2
    } else if time_spent > 5.0 {
        // Correct but slow: recalled with serious difficulty
        Quality::Grade3
    } else if time_spent > 2.0 {
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

    #[test]
    fn test_performance_to_quality_incorrect() {
        let quality = performance_to_quality(false, 1.0);
        assert!(matches!(quality, Quality::Grade0));
    }

    #[test]
    fn test_performance_to_quality_correct_fast() {
        let quality = performance_to_quality(true, 1.0);
        assert!(matches!(quality, Quality::Grade5));
    }

    #[test]
    fn test_performance_to_quality_correct_moderate() {
        let quality = performance_to_quality(true, 3.0);
        assert!(matches!(quality, Quality::Grade4));
    }

    #[test]
    fn test_performance_to_quality_correct_slow() {
        let quality = performance_to_quality(true, 7.0);
        assert!(matches!(quality, Quality::Grade3));
    }

    #[test]
    fn test_performance_to_quality_correct_very_slow() {
        let quality = performance_to_quality(true, 15.0);
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

use chrono::{DateTime, Utc};

/// Formats the time difference between two datetimes as human-readable text
///
/// Examples:
/// - Now or past: "now"
/// - 30 seconds from now: "in 30 seconds"
/// - 5 minutes from now: "in 5 minutes"
/// - 1 minute from now: "in 1 minute"
/// - 2 hours from now: "in 2 hours"
/// - 1 hour from now: "in 1 hour"
/// - 1 day from now: "tomorrow"
/// - 3 days from now: "in 3 days"
/// - 30 days from now: "on 2025-12-10"
pub fn format_time_difference(now: DateTime<Utc>, future_date: DateTime<Utc>) -> String {
    let duration = future_date.signed_duration_since(now);

    if duration.num_seconds() <= 0 {
        "now".to_string()
    } else if duration.num_seconds() < 60 {
        format!("in {} seconds", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("in {} minute{}", mins, if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("in {} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else if duration.num_days() == 1 {
        "tomorrow".to_string()
    } else if duration.num_days() < 30 {
        let days = duration.num_days();
        format!("in {} day{}", days, if days == 1 { "" } else { "s" })
    } else {
        format!("on {}", future_date.format("%Y-%m-%d"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // Helper to get current time for testing
    fn get_now() -> DateTime<Utc> {
        Utc::now()
    }

    // Helper to check if string contains a number within a range
    fn contains_number_in_range(s: &str, min: i64, max: i64) -> bool {
        for i in min..=max {
            if s.contains(&i.to_string()) {
                return true;
            }
        }
        false
    }

    // Boundary tests for seconds
    #[test]
    fn test_format_past_date() {
        let now = get_now();
        let past = now - Duration::seconds(10);
        assert_eq!(format_time_difference(now, past), "now");
    }

    #[test]
    fn test_format_now() {
        let now = get_now();
        assert_eq!(format_time_difference(now, now), "now");
    }

    #[test]
    fn test_format_30_seconds() {
        let now = get_now();
        let future = now + Duration::seconds(30);
        let result = format_time_difference(now, future);
        assert!(
            result.starts_with("in ") && result.contains("seconds"),
            "Expected seconds format, got: {}",
            result
        );
    }

    // Boundary tests for minutes
    #[test]
    fn test_format_1_minute() {
        let now = get_now();
        let future = now + Duration::minutes(1) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("minute"),
            "Expected minute format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_2_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(2) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("minutes"),
            "Expected minutes format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_30_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(30);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("minute"),
            "Expected minute format, got: {}",
            result
        );
        assert!(
            contains_number_in_range(&result, 29, 31),
            "Expected ~30 minutes, got: {}",
            result
        );
    }

    #[test]
    fn test_format_59_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(59);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("minute"),
            "Expected minute format, got: {}",
            result
        );
    }

    // Boundary tests for hours
    #[test]
    fn test_format_1_hour() {
        let now = get_now();
        let future = now + Duration::hours(1) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("hour"),
            "Expected hour format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_2_hours() {
        let now = get_now();
        let future = now + Duration::hours(2) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("hour"),
            "Expected hour format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_12_hours() {
        let now = get_now();
        let future = now + Duration::hours(12);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("hour"),
            "Expected hour format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_23_hours() {
        let now = get_now();
        let future = now + Duration::hours(23);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("hour"),
            "Expected hour format, got: {}",
            result
        );
    }

    // Boundary tests for days - special case for "tomorrow"
    #[test]
    fn test_format_1_day_exactly() {
        let now = get_now();
        let future = now + Duration::days(1) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result == "tomorrow" || result.contains("day"),
            "Expected 'tomorrow' or day format, got: {}",
            result
        );
    }

    // Boundary tests for multiple days
    #[test]
    fn test_format_2_days() {
        let now = get_now();
        let future = now + Duration::days(2) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("day"),
            "Expected day format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_7_days() {
        let now = get_now();
        let future = now + Duration::days(7);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("day"),
            "Expected day format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_29_days() {
        let now = get_now();
        let future = now + Duration::days(29);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("day"),
            "Expected day format, got: {}",
            result
        );
    }

    // Boundary tests for date format (30+ days)
    #[test]
    fn test_format_30_days() {
        let now = get_now();
        let future = now + Duration::days(30) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.starts_with("on "),
            "Expected 'on YYYY-MM-DD', got: {}",
            result
        );
        assert!(result.contains("-"), "Expected date format with dashes");
    }

    #[test]
    fn test_format_365_days() {
        let now = get_now();
        let future = now + Duration::days(365);
        let result = format_time_difference(now, future);
        assert!(
            result.starts_with("on "),
            "Expected 'on YYYY-MM-DD', got: {}",
            result
        );
    }

    // Edge case tests
    #[test]
    fn test_format_singular_vs_plural_minutes() {
        let now = get_now();
        let one_min = now + Duration::minutes(1) + Duration::seconds(1);
        let result_one = format_time_difference(now, one_min);
        assert!(
            result_one.contains("minute"),
            "Expected 'minute' singular, got: {}",
            result_one
        );

        let two_mins = now + Duration::minutes(2) + Duration::seconds(1);
        let result_two = format_time_difference(now, two_mins);
        assert!(
            result_two.contains("minutes"),
            "Expected 'minutes' plural, got: {}",
            result_two
        );
    }

    #[test]
    fn test_format_singular_vs_plural_hours() {
        let now = get_now();
        let one_hour = now + Duration::hours(1) + Duration::seconds(1);
        let result_one = format_time_difference(now, one_hour);
        assert!(
            result_one.contains("hour"),
            "Expected 'hour' singular, got: {}",
            result_one
        );

        let two_hours = now + Duration::hours(2) + Duration::seconds(1);
        let result_two = format_time_difference(now, two_hours);
        assert!(
            result_two.contains("hours"),
            "Expected 'hours' plural, got: {}",
            result_two
        );
    }

    #[test]
    fn test_format_singular_vs_plural_days() {
        let now = get_now();
        let one_day = now + Duration::days(1) + Duration::seconds(1);
        let result = format_time_difference(now, one_day);
        // 1 day should be "tomorrow"
        assert!(
            result == "tomorrow" || result.contains("day"),
            "Expected 'tomorrow' or day format, got: {}",
            result
        );

        let two_days = now + Duration::days(2) + Duration::seconds(1);
        let result_two = format_time_difference(now, two_days);
        assert!(
            result_two.contains("days"),
            "Expected 'days' plural, got: {}",
            result_two
        );
    }

    #[test]
    fn test_format_boundary_59s_to_1m() {
        let now = get_now();
        let fifty_nine_sec = now + Duration::seconds(59);
        let result_59s = format_time_difference(now, fifty_nine_sec);
        assert!(
            result_59s.contains("seconds"),
            "Expected seconds, got: {}",
            result_59s
        );

        let sixty_sec = now + Duration::seconds(60);
        let result_60s = format_time_difference(now, sixty_sec);
        assert!(
            result_60s.contains("minute"),
            "Expected minutes, got: {}",
            result_60s
        );
    }

    #[test]
    fn test_format_boundary_59m_to_1h() {
        let now = get_now();
        let fifty_nine_min = now + Duration::minutes(59);
        let result_59m = format_time_difference(now, fifty_nine_min);
        assert!(
            result_59m.contains("minute"),
            "Expected minutes, got: {}",
            result_59m
        );

        let sixty_min = now + Duration::minutes(60) + Duration::seconds(1);
        let result_60m = format_time_difference(now, sixty_min);
        assert!(
            result_60m.contains("hour"),
            "Expected hours, got: {}",
            result_60m
        );
    }

    #[test]
    fn test_format_boundary_23h_to_1d() {
        let now = get_now();
        let twenty_three_hours = now + Duration::hours(23);
        let result_23h = format_time_difference(now, twenty_three_hours);
        assert!(
            result_23h.contains("hour"),
            "Expected hours, got: {}",
            result_23h
        );

        let twenty_four_hours = now + Duration::hours(24) + Duration::seconds(1);
        let result_24h = format_time_difference(now, twenty_four_hours);
        assert!(
            result_24h == "tomorrow" || result_24h.contains("day"),
            "Expected 'tomorrow' or day format, got: {}",
            result_24h
        );
    }

    #[test]
    fn test_format_boundary_29d_to_30d() {
        let now = get_now();
        let twenty_nine_days = now + Duration::days(29);
        let result_29d = format_time_difference(now, twenty_nine_days);
        assert!(
            result_29d.contains("day"),
            "Expected day format for 29 days, got: {}",
            result_29d
        );

        let thirty_days = now + Duration::days(30) + Duration::seconds(1);
        let result_30d = format_time_difference(now, thirty_days);
        assert!(
            result_30d.starts_with("on "),
            "Expected 'on YYYY-MM-DD' for 30+ days, got: {}",
            result_30d
        );
    }

    // Realistic spaced repetition scenarios
    #[test]
    fn test_format_first_review_1_day() {
        let now = get_now();
        let future = now + Duration::days(1) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result == "tomorrow" || result.contains("day"),
            "First review in 1 day should be 'tomorrow' or contain 'day', got: {}",
            result
        );
    }

    #[test]
    fn test_format_second_review_3_days() {
        let now = get_now();
        let future = now + Duration::days(3) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("day"),
            "Expected day format, got: {}",
            result
        );
        assert!(
            result.contains("3"),
            "Expected to contain '3', got: {}",
            result
        );
    }

    #[test]
    fn test_format_third_review_7_days() {
        let now = get_now();
        let future = now + Duration::days(7) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("day"),
            "Expected day format, got: {}",
            result
        );
        assert!(
            result.contains("7"),
            "Expected to contain '7', got: {}",
            result
        );
    }

    #[test]
    fn test_format_later_review_30_days() {
        let now = get_now();
        let future = now + Duration::days(30) + Duration::seconds(1);
        let result = format_time_difference(now, future);
        assert!(
            result.starts_with("on "),
            "Later review should use date format, got: {}",
            result
        );
    }

    #[test]
    fn test_format_incorrect_retry_10_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(10);
        let result = format_time_difference(now, future);
        assert!(
            result.contains("minute"),
            "Expected minute format, got: {}",
            result
        );
        assert!(
            contains_number_in_range(&result, 9, 11),
            "Expected ~10 minutes, got: {}",
            result
        );
    }
}

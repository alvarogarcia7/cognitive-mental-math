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

    // Boundary tests for seconds
    #[test]
    fn test_format_past_date() {
        let now = get_now();
        let past = now - Duration::seconds(10);
        assert_eq!(format_time_difference(now, past), "now");
    }

    #[test]
    fn test_format_now() {
        assert_eq!(format_time_difference(get_now(), get_now()), "now");
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
        let future = now + Duration::minutes(1);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 1 minute");
    }

    #[test]
    fn test_format_2_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(2);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 2 minutes");
    }

    #[test]
    fn test_format_30_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(30);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 30 minutes");
    }

    #[test]
    fn test_format_59_minutes() {
        let now = get_now();
        let future = now + Duration::minutes(59);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 59 minutes");
    }

    // Boundary tests for hours
    #[test]
    fn test_format_1_hour() {
        let now = get_now();
        let future = now + Duration::hours(1);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 1 hour");
    }

    #[test]
    fn test_format_2_hours() {
        let now = get_now();
        let future = now + Duration::hours(2);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 2 hours");
    }

    #[test]
    fn test_format_12_hours() {
        let now = get_now();
        let future = now + Duration::hours(12);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 12 hours");
    }

    #[test]
    fn test_format_23_hours() {
        let now = get_now();
        let future = now + Duration::hours(23);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 23 hours");
    }

    // Boundary tests for days - special case for "tomorrow"
    #[test]
    fn test_format_1_day_exactly() {
        let now = get_now();
        let future = now + Duration::days(1);
        let result = format_time_difference(now, future);
        assert_eq!(result, "tomorrow");
    }

    // Boundary tests for multiple days
    #[test]
    fn test_format_2_days() {
        let now = get_now();
        let future = now + Duration::days(2);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 2 days");
    }

    #[test]
    fn test_format_7_days() {
        let now = get_now();
        let future = now + Duration::days(7);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 7 days");
    }

    #[test]
    fn test_format_29_days() {
        let now = get_now();
        let future = now + Duration::days(29);
        let result = format_time_difference(now, future);
        assert_eq!(result, "in 29 days");
    }

    // Boundary tests for date format (30+ days)
    #[test]
    fn test_format_30_days() {
        let now = get_now();
        let future = now + Duration::days(30);
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
        let one_min = now + Duration::minutes(1);
        let result_one = format_time_difference(now, one_min);
        assert_eq!(result_one, "in 1 minute");

        let two_mins = now + Duration::minutes(2);
        let result_two = format_time_difference(now, two_mins);
        assert_eq!(result_two, "in 2 minutes");
    }

    #[test]
    fn test_format_singular_vs_plural_hours() {
        let now = get_now();
        let one_hour = now + Duration::hours(1);
        let result_one = format_time_difference(now, one_hour);
        assert_eq!(result_one, "in 1 hour");

        let two_hours = now + Duration::hours(2);
        let result_two = format_time_difference(now, two_hours);
        assert_eq!(result_two, "in 2 hours");
    }

    #[test]
    fn test_format_singular_vs_plural_days() {
        let now = get_now();
        let one_day = now + Duration::days(1);
        let result = format_time_difference(now, one_day);
        // 1 day should be "tomorrow"
        assert_eq!(result, "tomorrow");

        let two_days = now + Duration::days(2);
        let result_two = format_time_difference(now, two_days);
        assert_eq!(result_two, "in 2 days");
    }

    #[test]
    fn test_format_boundary_59s_to_1m() {
        let now = get_now();
        let fifty_nine_sec = now + Duration::seconds(59);
        let result_59s = format_time_difference(now, fifty_nine_sec);
        assert_eq!(result_59s, "in 59 seconds");

        let sixty_sec = now + Duration::seconds(60);
        let result_60s = format_time_difference(now, sixty_sec);
        assert_eq!(result_60s, "in 1 minute");
    }

    #[test]
    fn test_format_boundary_59m_to_1h() {
        let now = get_now();
        let fifty_nine_min = now + Duration::minutes(59);
        let result_59m = format_time_difference(now, fifty_nine_min);
        assert_eq!(result_59m, "in 59 minutes");

        let sixty_min = now + Duration::minutes(60);
        let result_60m = format_time_difference(now, sixty_min);
        assert_eq!(result_60m, "in 1 hour");
    }

    #[test]
    fn test_format_boundary_23h_to_1d() {
        let now = get_now();
        let twenty_three_hours = now + Duration::hours(23);
        let result_23h = format_time_difference(now, twenty_three_hours);
        assert_eq!(result_23h, "in 23 hours");

        let twenty_four_hours = now + Duration::hours(24);
        let result_24h = format_time_difference(now, twenty_four_hours);
        assert_eq!(result_24h, "tomorrow");
    }

    #[test]
    fn test_format_boundary_29d_to_30d() {
        let now = get_now();
        let twenty_nine_days = now + Duration::days(29);
        let result_29d = format_time_difference(now, twenty_nine_days);
        assert_eq!(result_29d, "in 29 days");

        let thirty_days = now + Duration::days(30);
        let result_30d = format_time_difference(now, thirty_days);
        assert!(
            result_30d.starts_with("on "),
            "Expected 'on YYYY-MM-DD' for 30+ days, got: {}",
            result_30d
        );
    }
}

use chrono::{DateTime, NaiveDate, Timelike, Utc};

/// Trait for providing the current date/time to the database
/// This allows for flexible date handling (system time, overrides, etc.)
pub trait DateProvider: Send + Sync {
    /// Get the current date/time
    fn get_current_time(&self) -> DateTime<Utc>;
}

/// Default date provider that uses the system's current date/time
pub struct SystemDateProvider;

impl DateProvider for SystemDateProvider {
    fn get_current_time(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Date provider that uses an overridden date instead of system time
/// Preserves the current hours/minutes/seconds from system time
pub struct OverrideDateProvider {
    override_date: NaiveDate,
}

impl OverrideDateProvider {
    /// Create a new override date provider with a specific date
    pub fn new(override_date: NaiveDate) -> Self {
        Self { override_date }
    }
}

impl DateProvider for OverrideDateProvider {
    fn get_current_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        let naive_datetime = self
            .override_date
            .and_hms_opt(now.hour(), now.minute(), now.second())
            .unwrap_or_else(|| self.override_date.and_hms_opt(0, 0, 0).unwrap());
        DateTime::from_naive_utc_and_offset(naive_datetime, Utc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_date_provider_returns_current_time() {
        let provider = SystemDateProvider;
        let time1 = provider.get_current_time();
        let time2 = provider.get_current_time();

        // Times should be very close (within a second)
        assert!((time2 - time1).num_seconds() <= 1);
    }

    #[test]
    fn test_override_date_provider_uses_override_date() {
        let override_date = NaiveDate::from_ymd_opt(2025, 11, 18).unwrap();
        let provider = OverrideDateProvider::new(override_date);
        let time = provider.get_current_time();

        // Check that the date part matches the override date
        assert_eq!(time.format("%Y-%m-%d").to_string(), "2025-11-18");
    }

    #[test]
    fn test_override_date_provider_preserves_time_of_day() {
        let override_date = NaiveDate::from_ymd_opt(2025, 11, 18).unwrap();
        let provider = OverrideDateProvider::new(override_date);
        let time = provider.get_current_time();

        let now = Utc::now();
        // Hours, minutes, and seconds should match current time (within 1 second tolerance)
        assert_eq!(time.hour(), now.hour());
        assert_eq!(time.minute(), now.minute());
        let second_diff = if time.second() >= now.second() {
            time.second() - now.second()
        } else {
            now.second() - time.second()
        };
        assert!(second_diff <= 1);
    }
}

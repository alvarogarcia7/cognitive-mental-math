pub mod accuracy;
pub mod streak;
pub mod time_statistics;

use rusqlite::Connection;

pub use accuracy::AccuracyRepository;
pub use streak::StreakRepository;
pub use time_statistics::TimeStatisticsRepository;

/// Analytics facade providing high-level analytics operations
pub struct Analytics<'a> {
    pub conn: &'a Connection,
}

impl<'a> Analytics<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Analytics { conn }
    }
}

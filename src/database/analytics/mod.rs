pub mod accuracy;
pub mod streak;
pub mod time_statistics;

use rusqlite::Connection;

pub use accuracy::AccuracyRepository;
pub use streak::StreakRepository;
pub use time_statistics::TimeStatisticsRepository;

/// Analytics facade providing high-level analytics operations
pub struct Analytics<'a> {
    conn: &'a Connection,
}

impl<'a> Analytics<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Analytics { conn }
    }

    pub fn time_statistics(&self) -> TimeStatisticsRepository<'_> {
        TimeStatisticsRepository::new(self.conn)
    }

    pub fn accuracy(&self) -> AccuracyRepository<'_> {
        AccuracyRepository::new(self.conn)
    }

    pub fn streak(&self) -> StreakRepository<'_> {
        StreakRepository::new(self.conn)
    }
}

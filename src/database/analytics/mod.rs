pub mod accuracy;
pub mod time_statistics;
pub mod streak;

use rusqlite::Connection;
use std::collections::HashMap;

use crate::spaced_repetition::AnswerTimedEvaluator;

pub use accuracy::AccuracyRepository;
pub use time_statistics::TimeStatisticsRepository;
pub use streak::StreakRepository;

/// Analytics facade providing high-level analytics operations
pub struct Analytics<'a> {
    conn: &'a Connection,
}

impl<'a> Analytics<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Analytics { conn }
    }

    pub fn time_statistics(&self) -> TimeStatisticsRepository {
        TimeStatisticsRepository::new(self.conn)
    }

    pub fn accuracy(&self) -> AccuracyRepository {
        AccuracyRepository::new(self.conn)
    }

    pub fn streak(&self) -> StreakRepository {
        StreakRepository::new(self.conn)
    }
}

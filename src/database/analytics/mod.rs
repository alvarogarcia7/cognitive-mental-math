pub mod accuracy;
pub mod answer_timed_evaluator;
pub mod streak;

use rusqlite::Connection;

pub use accuracy::AccuracyRepository;
pub use answer_timed_evaluator::AnswerTimedEvaluatorRepository;
pub use streak::StreakRepository;

/// Analytics facade providing high-level analytics operations
pub struct Analytics<'a> {
    pub conn: &'a Connection,
}

impl<'a> Analytics<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Analytics { conn }
    }
}

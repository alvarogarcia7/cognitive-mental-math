use crate::deck::Deck;
use crate::spaced_repetition::ReviewItem;
use chrono::{DateTime, Utc};
use rusqlite::Row;

/// Factory for creating Deck objects from database rows
pub struct DeckRowFactory;

impl DeckRowFactory {
    /// Creates a Deck from a database row
    /// Expected columns: id, created_at, completed_at, status, total_questions,
    ///                   correct_answers, incorrect_answers, total_time_seconds,
    ///                   average_time_seconds, accuracy_percentage
    pub fn from_row(row: &Row) -> rusqlite::Result<Deck> {
        use crate::deck::DeckStatus;

        Ok(Deck {
            id: row.get(0)?,
            created_at: row.get(1)?,
            completed_at: row.get(2)?,
            status: DeckStatus::from(&row.get::<_, String>(3)?).unwrap_or(DeckStatus::InProgress),
            total_questions: row.get(4)?,
            correct_answers: row.get(5)?,
            incorrect_answers: row.get(6)?,
            total_time_seconds: row.get(7)?,
            average_time_seconds: row.get(8)?,
            accuracy_percentage: row.get(9)?,
        })
    }
}

/// Factory for creating ReviewItem objects from database rows
pub struct ReviewItemRowFactory;

impl ReviewItemRowFactory {
    /// Creates a ReviewItem from a database row
    /// Expected columns: id, operation_id, repetitions, interval, ease_factor,
    ///                   next_review_date, last_reviewed_date
    pub fn from_row(row: &Row) -> rusqlite::Result<ReviewItem> {
        Ok(ReviewItem {
            id: Some(row.get(0)?),
            operation_id: row.get(1)?,
            repetitions: row.get(2)?,
            interval: row.get(3)?,
            ease_factor: row.get(4)?,
            next_review_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .unwrap()
                .with_timezone(&Utc),
            last_reviewed_date: row.get::<_, Option<String>>(6)?.map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .unwrap()
                    .with_timezone(&Utc)
            }),
        })
    }
}

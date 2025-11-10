use crate::deck::Deck;
use crate::spaced_repetition::ReviewItem;
use chrono::{DateTime, Utc};
use rusqlite::Row;

/// Parses a datetime string that may be in RFC3339 format (with timezone) or naive format
/// Always returns a DateTime<Utc> to ensure timezone information
fn parse_datetime_with_timezone(datetime_str: &str) -> DateTime<Utc> {
    // Try parsing as RFC3339 first (includes timezone info)
    if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
        return dt.with_timezone(&Utc);
    }

    // Fallback: parse as naive datetime and assume UTC
    chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| {
            // Last resort: try other common formats
            chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S")
                .expect("Unable to parse datetime string")
        })
        .and_utc()
}

/// Factory for creating Deck objects from database rows
pub struct DeckRowFactory;

impl DeckRowFactory {
    /// Creates a Deck from a database row
    /// Expected columns: id, created_at, completed_at, status, total_questions,
    ///                   correct_answers, incorrect_answers, total_time_seconds,
    ///                   average_time_seconds, accuracy_percentage
    pub fn from_row(row: &Row) -> rusqlite::Result<Deck> {
        use crate::deck::DeckStatus;

        // Parse created_at - try RFC3339 first (with timezone), then fallback to naive datetime
        let created_at_str: String = row.get(1)?;
        let created_at = parse_datetime_with_timezone(&created_at_str);

        // Parse completed_at if present
        let completed_at = row
            .get::<_, Option<String>>(2)?
            .map(|s| parse_datetime_with_timezone(&s));

        Ok(Deck {
            id: row.get(0)?,
            created_at,
            completed_at,
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
            next_review_date: parse_datetime_with_timezone(&row.get::<_, String>(5)?),
            last_reviewed_date: row
                .get::<_, Option<String>>(6)?
                .map(|s| parse_datetime_with_timezone(&s)),
        })
    }
}

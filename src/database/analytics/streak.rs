use chrono::{DateTime, Utc};
use rusqlite::Connection;
use rusqlite::Result;
use std::collections::HashSet;

pub struct StreakRepository<'a> {
    conn: &'a Connection,
}

impl<'a> StreakRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        StreakRepository { conn }
    }

    /// Calculate the number of consecutive days the user has completed answers
    /// Returns the streak count (0 if no answers exist)
    pub fn calculate_consecutive_days(&self) -> Result<i32> {
        // Get all unique dates when answers were completed, ordered by date descending
        let mut stmt = self.conn.prepare(
            r#"SELECT DISTINCT DATE(a.created_at) as answer_date
            FROM answers a
            ORDER BY answer_date DESC"#,
        )?;

        let dates: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        if dates.is_empty() {
            return Ok(0);
        }

        // Calculate streak starting from the most recent date
        let mut streak = 0;
        let mut current_date_idx = 0;

        // Get today's date and yesterday's date to handle cases where last activity wasn't today
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let yesterday = (Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        // Start from most recent date
        let mut expected_date = if dates[0] == today {
            today.clone()
        } else if dates[0] == yesterday {
            yesterday.clone()
        } else {
            // If most recent activity is more than 1 day old, streak is broken
            return Ok(0);
        };

        // Count consecutive days
        while current_date_idx < dates.len() {
            if dates[current_date_idx] == expected_date {
                streak += 1;
                current_date_idx += 1;

                // Move to previous day
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&expected_date, "%Y-%m-%d") {
                    if let Some(prev_date) = date.pred_opt() {
                        expected_date = prev_date.format("%Y-%m-%d").to_string();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                // Date mismatch means streak is broken
                break;
            }
        }

        Ok(streak)
    }

    /// Get days with answers in the last 10 days
    /// Returns a vector of dates that had any answers (correct or not)
    /// Example: if in the last 10 days answers exist on [2025-01-10, 2025-01-08, 2025-01-05]
    /// returns [2025-01-10, 2025-01-08, 2025-01-05] (sorted descending)
    pub fn get_days_with_answers(&self, now: DateTime<Utc>) -> Result<Vec<String>> {
        let today = now.date_naive();
        let ten_days_ago = today - chrono::Duration::days(9); // 10 days including today

        // Get all unique dates with answers in the last 10 days
        let mut stmt = self.conn.prepare(
            r#"SELECT DISTINCT DATE(a.created_at) as answer_date
            FROM answers a
            WHERE DATE(a.created_at) >= ?1 AND DATE(a.created_at) <= ?2
            ORDER BY answer_date DESC"#,
        )?;

        let dates_with_answers: Vec<String> = stmt
            .query_map([ten_days_ago.to_string(), today.to_string()], |row| {
                row.get(0)
            })?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(dates_with_answers)
    }

    /// Get missing days in the last 10 days
    /// Returns a vector of dates that did not have any answers (correct or not)
    /// Ignores the max_days parameter for backward compatibility but always checks last 10 days
    /// Example: if in the last 10 days answers exist on [2025-01-10, 2025-01-08, 2025-01-05]
    /// returns [2025-01-09, 2025-01-07, 2025-01-06, 2025-01-04, 2025-01-03, 2025-01-02, 2025-01-01]
    pub fn get_missing_days(&self, _max_days: i32, now: DateTime<Utc>) -> Result<Vec<String>> {
        // Always check the last 10 days
        let today = now.date_naive();
        let ten_days_ago = today - chrono::Duration::days(9); // 10 days including today

        // Generate all dates in the last 10 days
        let mut all_dates = Vec::new();
        let mut current = today;
        while current >= ten_days_ago {
            all_dates.push(current.format("%Y-%m-%d").to_string());
            current -= chrono::Duration::days(1);
        }

        // Get all unique dates with answers in the last 10 days
        let mut stmt = self.conn.prepare(
            r#"SELECT DISTINCT DATE(a.created_at) as answer_date
            FROM answers a
            WHERE DATE(a.created_at) >= ?1 AND DATE(a.created_at) <= ?2
            ORDER BY answer_date DESC"#,
        )?;

        let dates_with_answers: Vec<String> = stmt
            .query_map([ten_days_ago.to_string(), today.to_string()], |row| {
                row.get(0)
            })?
            .collect::<Result<Vec<String>, _>>()?;

        // Find dates without answers
        let dates_with_answers_set: HashSet<_> = dates_with_answers.into_iter().collect();
        let missing_days: Vec<String> = all_dates
            .into_iter()
            .filter(|date| !dates_with_answers_set.contains(date))
            .collect();

        Ok(missing_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::answers::AnswersRepository;
    use crate::database::connection::init_connection;
    use crate::database::decks::DecksRepository;
    use crate::database::operations::OperationsRepository;

    fn create_test_db() -> rusqlite::Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_consecutive_days_streak_no_answers() {
        let conn = create_test_db();
        let streak_repo = StreakRepository::new(&conn);
        let streak = streak_repo.calculate_consecutive_days().unwrap();
        assert_eq!(streak, 0);
    }

    #[test]
    fn test_consecutive_days_streak_single_day() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        // Use current date for this test since calculate_consecutive_days uses Utc::now() internally
        let fixed_date = Utc::now();
        let date_provider = Box::new(move || fixed_date);
        let answers_repo = AnswersRepository::new_with_date_provider(&conn, &*date_provider);
        let decks_repo = DecksRepository::new(&conn, Box::new(move || fixed_date));
        let streak_repo = StreakRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 1.0, Some(deck_id))
            .unwrap();
        decks_repo.complete(deck_id).unwrap();

        let streak = streak_repo.calculate_consecutive_days().unwrap();
        assert_eq!(streak, 1);
    }

    #[test]
    fn test_get_days_with_answers_empty() {
        let conn = create_test_db();
        let streak_repo = StreakRepository::new(&conn);
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        // No answers in the database
        let days_with_answers = streak_repo.get_days_with_answers(fixed_date).unwrap();
        assert_eq!(days_with_answers.len(), 0);
    }

    #[test]
    fn test_get_days_with_answers_with_recent_answer() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let date_provider = Box::new(move || fixed_date);
        let answers_repo = AnswersRepository::new_with_date_provider(&conn, &*date_provider);
        let decks_repo = DecksRepository::new(&conn, Box::new(move || fixed_date));
        let streak_repo = StreakRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 1.0, Some(deck_id))
            .unwrap();

        // With one recent answer today, there should be 1 day with answers
        let days_with_answers = streak_repo.get_days_with_answers(fixed_date).unwrap();
        assert_eq!(days_with_answers.len(), 1);
    }

    #[test]
    fn test_get_missing_days_in_streak_empty() {
        let conn = create_test_db();
        let streak_repo = StreakRepository::new(&conn);
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        // No answers in the database
        let missing_days = streak_repo.get_missing_days(10, fixed_date).unwrap();
        // All days in the last 10 days should be missing since there are no answers
        // (exactly 10 days: today + 9 days back)
        assert_eq!(missing_days.len(), 10);
    }

    #[test]
    fn test_get_missing_days_in_streak_with_recent_answer() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let date_provider = Box::new(move || fixed_date);
        let answers_repo = AnswersRepository::new_with_date_provider(&conn, &*date_provider);
        let decks_repo = DecksRepository::new(&conn, Box::new(move || fixed_date));
        let streak_repo = StreakRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 1.0, Some(deck_id))
            .unwrap();

        // With one recent answer today, there should be 9 missing days in the last 10
        let missing_days = streak_repo.get_missing_days(10, fixed_date).unwrap();
        // Should have 9 missing days (1 day has answer, 9 other days don't)
        assert_eq!(missing_days.len(), 9);
    }

    #[test]
    fn test_get_missing_days_ignores_max_days_parameter() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let date_provider = Box::new(move || fixed_date);
        let answers_repo = AnswersRepository::new_with_date_provider(&conn, &*date_provider);
        let decks_repo = DecksRepository::new(&conn, Box::new(move || fixed_date));
        let streak_repo = StreakRepository::new(&conn);

        let deck_id = decks_repo.create().unwrap();
        let op_id = ops_repo.insert("ADD", 2, 3, 5, Some(deck_id)).unwrap();
        answers_repo
            .insert(op_id, 5, true, 1.0, Some(deck_id))
            .unwrap();

        // Test with different max_days values - should always return 10 days worth of data
        let missing_5 = streak_repo.get_missing_days(5, fixed_date).unwrap();
        let missing_10 = streak_repo.get_missing_days(10, fixed_date).unwrap();
        let missing_20 = streak_repo.get_missing_days(20, fixed_date).unwrap();

        // All should return 10 days of data (today + 9 days back) minus days with answers
        // Since we have 1 answer today, each should have 9 missing days
        assert_eq!(missing_5.len(), 9);
        assert_eq!(missing_10.len(), 9);
        assert_eq!(missing_20.len(), 9);
    }
}

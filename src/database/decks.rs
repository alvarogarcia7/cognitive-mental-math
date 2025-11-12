use crate::deck::{Deck, DeckStatus, DeckSummary};
use crate::row_factories::DeckRowFactory;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, params};

pub struct DecksRepository<'a> {
    conn: &'a Connection,
    get_current_time: Box<dyn Fn() -> DateTime<Utc> + 'a>,
}

impl<'a> DecksRepository<'a> {
    pub fn new(
        conn: &'a Connection,
        get_current_time: Box<dyn Fn() -> DateTime<Utc> + 'a>,
    ) -> Self {
        DecksRepository {
            conn,
            get_current_time,
        }
    }

    pub fn create(&self) -> Result<i64> {
        let now_utc = (self.get_current_time)().to_rfc3339();
        self.conn.execute(
            "INSERT INTO decks (created_at, status) VALUES (?1, ?2)",
            params![now_utc, DeckStatus::InProgress.as_str()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get(&self, deck_id: i64) -> Result<Option<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks WHERE id = ?1",
        )?;

        let mut rows = stmt.query([deck_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(DeckRowFactory::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_summary(&self, deck_id: i64, summary: &DeckSummary) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET
                total_questions = ?1,
                correct_answers = ?2,
                incorrect_answers = ?3,
                total_time_seconds = ?4,
                average_time_seconds = ?5,
                accuracy_percentage = ?6
             WHERE id = ?7",
            params![
                summary.total_questions,
                summary.correct_answers,
                summary.incorrect_answers,
                summary.total_time_seconds,
                summary.average_time_seconds,
                summary.accuracy_percentage,
                deck_id
            ],
        )?;
        Ok(())
    }

    pub fn complete(&self, deck_id: i64) -> Result<()> {
        let now_utc = (self.get_current_time)().to_rfc3339();
        self.conn.execute(
            "UPDATE decks SET status = ?1, completed_at = ?3 WHERE id = ?2",
            params![DeckStatus::Completed.as_str(), deck_id, now_utc],
        )?;
        Ok(())
    }

    pub fn abandon(&self, deck_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET status = ?1 WHERE id = ?2",
            params![DeckStatus::Abandoned.as_str(), deck_id],
        )?;
        Ok(())
    }

    pub fn get_recent(&self, limit: i32) -> Result<Vec<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit], DeckRowFactory::from_row)?;

        let mut decks = Vec::new();
        for deck_result in rows {
            decks.push(deck_result?);
        }
        Ok(decks)
    }

    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::init_connection;

    fn create_test_db() -> Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    fn create_repo(conn: &Connection) -> DecksRepository {
        DecksRepository::new(conn, Box::new(|| Utc::now()))
    }

    #[test]
    fn test_create_deck() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let deck_id = repo.create().unwrap();
        assert_eq!(deck_id, 1);
        assert_eq!(repo.count().unwrap(), 1);
    }

    #[test]
    fn test_get_deck() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let deck_id = repo.create().unwrap();

        let deck = repo.get(deck_id).unwrap().unwrap();
        assert_eq!(deck.id, deck_id);
        assert_eq!(deck.status, crate::deck::DeckStatus::InProgress);
        assert_eq!(deck.total_questions, 0);
    }

    #[test]
    fn test_complete_deck() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let deck_id = repo.create().unwrap();

        repo.complete(deck_id).unwrap();

        let deck = repo.get(deck_id).unwrap().unwrap();
        assert_eq!(deck.status, crate::deck::DeckStatus::Completed);
        assert!(deck.completed_at.is_some());
    }

    #[test]
    fn test_abandon_deck() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let deck_id = repo.create().unwrap();

        repo.abandon(deck_id).unwrap();

        let deck = repo.get(deck_id).unwrap().unwrap();
        assert_eq!(deck.status, crate::deck::DeckStatus::Abandoned);
    }

    #[test]
    fn test_update_deck_summary() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let deck_id = repo.create().unwrap();

        let summary = crate::deck::DeckSummary {
            total_questions: 10,
            correct_answers: 8,
            incorrect_answers: 2,
            total_time_seconds: 25.5,
            average_time_seconds: 2.55,
            accuracy_percentage: 80.0,
        };

        repo.update_summary(deck_id, &summary).unwrap();

        let deck = repo.get(deck_id).unwrap().unwrap();
        assert_eq!(deck.total_questions, 10);
        assert_eq!(deck.correct_answers, 8);
        assert_eq!(deck.incorrect_answers, 2);
        assert_eq!(deck.total_time_seconds, 25.5);
        assert_eq!(deck.average_time_seconds, Some(2.55));
        assert_eq!(deck.accuracy_percentage, Some(80.0));
    }

    #[test]
    fn test_get_recent_decks() {
        let conn = create_test_db();
        let repo = create_repo(&conn);
        let _deck1 = repo.create().unwrap();
        let _deck2 = repo.create().unwrap();
        let _deck3 = repo.create().unwrap();

        let all_decks = repo.get_recent(10).unwrap();
        assert_eq!(all_decks.len(), 3);

        let recent = repo.get_recent(2).unwrap();
        assert_eq!(recent.len(), 2);

        // When timestamps are identical (tests run quickly),
        // we should at least get 2 different decks
        assert_ne!(recent[0].id, recent[1].id);
    }
}

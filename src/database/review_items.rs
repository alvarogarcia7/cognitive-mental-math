use crate::row_factories::ReviewItemRowFactory;
use crate::spaced_repetition::ReviewItem;
use crate::time_format::format_time_difference;
use chrono::{DateTime, Utc};
use log::debug;
use rusqlite::{params, Connection, Result};

pub struct ReviewItemsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ReviewItemsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        ReviewItemsRepository { conn }
    }

    pub fn insert(&self, operation_id: i64, next_review_date: DateTime<Utc>) -> Result<i64> {
        let next_review_str = next_review_date.to_rfc3339();
        debug!(
            "Creating new review item for operation_id={}, next review: {}",
            operation_id,
            format_time_difference(Utc::now(), next_review_date)
        );
        self.conn.execute(
            "INSERT INTO review_items (operation_id, next_review_date)
             VALUES (?1, ?2)",
            params![operation_id, next_review_str],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update(&self, item: &ReviewItem) -> Result<()> {
        let next_review_str = item.next_review_date.to_rfc3339();
        let last_reviewed_str = item.last_reviewed_date.map(|d| d.to_rfc3339());

        debug!(
            "Updating review item id={}: reps={}, interval={} days, ease={:.2}, next review: {}",
            item.id.unwrap_or(0),
            item.repetitions,
            item.interval,
            item.ease_factor,
            format_time_difference(Utc::now(), item.next_review_date)
        );

        self.conn.execute(
            "UPDATE review_items
             SET repetitions = ?1, interval = ?2, ease_factor = ?3,
                 next_review_date = ?4, last_reviewed_date = ?5
             WHERE id = ?6",
            params![
                item.repetitions,
                item.interval,
                item.ease_factor,
                next_review_str,
                last_reviewed_str,
                item.id
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, operation_id: i64) -> Result<Option<ReviewItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, repetitions, interval, ease_factor,
                    next_review_date, last_reviewed_date
             FROM review_items WHERE operation_id = ?1",
        )?;

        let mut rows = stmt.query([operation_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ReviewItemRowFactory::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_due(&self, before_date: DateTime<Utc>) -> Result<Vec<ReviewItem>> {
        let before_str = before_date.to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, repetitions, interval, ease_factor,
                    next_review_date, last_reviewed_date
             FROM review_items
             WHERE next_review_date <= ?1
             ORDER BY next_review_date ASC",
        )?;

        let items = stmt.query_map([&before_str], ReviewItemRowFactory::from_row)?;

        let mut result = Vec::new();
        for item in items {
            result.push(item?);
        }

        debug!("Retrieved {} due review items from database", result.len());
        Ok(result)
    }

    pub fn count_due(&self, before_date: DateTime<Utc>) -> Result<i64> {
        let before_str = before_date.to_rfc3339();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM review_items WHERE next_review_date <= ?1",
            [&before_str],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::init_connection;
    use crate::database::operations::OperationsRepository;

    fn create_test_db() -> Connection {
        init_connection(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_insert_review_item() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let op_id = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let now = chrono::Utc::now();
        let review_id = review_repo.insert(op_id, now).unwrap();

        assert!(review_id > 0);
    }

    #[test]
    fn test_get_review_item() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let op_id = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let now = chrono::Utc::now();
        review_repo.insert(op_id, now).unwrap();

        let item = review_repo.get(op_id).unwrap();
        assert!(item.is_some());
        let review = item.unwrap();
        assert_eq!(review.operation_id, op_id);
        assert_eq!(review.repetitions, 0);
        assert_eq!(review.interval, 0);
        assert_eq!(review.ease_factor, 2.5);
    }

    #[test]
    fn test_update_review_item() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let op_id = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        review_repo.insert(op_id, fixed_date).unwrap();

        let mut item = review_repo.get(op_id).unwrap().unwrap();
        item.repetitions = 1;
        item.interval = 3;
        item.ease_factor = 2.7;
        item.next_review_date = fixed_date + chrono::Duration::days(3);

        review_repo.update(&item).unwrap();

        let updated = review_repo.get(op_id).unwrap().unwrap();
        assert_eq!(updated.repetitions, 1);
        assert_eq!(updated.interval, 3);
        assert_eq!(updated.ease_factor, 2.7);
    }

    #[test]
    fn test_get_due_reviews() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let now = chrono::Utc::now();
        let past = now - chrono::Duration::days(1);
        let future = now + chrono::Duration::days(1);

        let op_id1 = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = ops_repo.insert("ADD", 4, 5, 9, None).unwrap();
        let op_id3 = ops_repo.insert("ADD", 6, 7, 13, None).unwrap();

        // op_id1: due (past date)
        review_repo.insert(op_id1, past).unwrap();
        // op_id2: due (exactly now)
        review_repo.insert(op_id2, now).unwrap();
        // op_id3: not due (future date)
        review_repo.insert(op_id3, future).unwrap();

        let due = review_repo.get_due(now).unwrap();
        assert_eq!(due.len(), 2);

        // Verify the returned items are the correct ones
        let due_ids: Vec<i64> = due.iter().map(|item| item.operation_id).collect();
        assert!(due_ids.contains(&op_id1));
        assert!(due_ids.contains(&op_id2));
        assert!(!due_ids.contains(&op_id3));
    }

    #[test]
    fn test_count_due_reviews() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let past = fixed_date - chrono::Duration::days(1);
        let future = fixed_date + chrono::Duration::days(1);

        let op_id1 = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = ops_repo.insert("ADD", 4, 5, 9, None).unwrap();
        let op_id3 = ops_repo.insert("ADD", 6, 7, 13, None).unwrap();

        review_repo.insert(op_id1, past).unwrap();
        review_repo.insert(op_id2, fixed_date).unwrap();
        review_repo.insert(op_id3, future).unwrap();

        let count = review_repo.count_due(fixed_date).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_nonexistent_review_item() {
        let conn = create_test_db();
        let review_repo = ReviewItemsRepository::new(&conn);
        let result = review_repo.get(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_review_item_with_multiple_operations() {
        let conn = create_test_db();
        let ops_repo = OperationsRepository::new(&conn);
        let review_repo = ReviewItemsRepository::new(&conn);

        let fixed_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        let op_id1 = ops_repo.insert("ADD", 2, 3, 5, None).unwrap();
        let op_id2 = ops_repo.insert("MULTIPLY", 3, 4, 12, None).unwrap();

        review_repo.insert(op_id1, fixed_date).unwrap();
        review_repo
            .insert(op_id2, fixed_date + chrono::Duration::days(1))
            .unwrap();

        let item1 = review_repo.get(op_id1).unwrap().unwrap();
        let item2 = review_repo.get(op_id2).unwrap().unwrap();

        assert_eq!(item1.operation_id, op_id1);
        assert_eq!(item2.operation_id, op_id2);
        assert_ne!(item1.operation_id, item2.operation_id);
    }
}

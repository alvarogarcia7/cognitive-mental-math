# Deck Creation Feature Implementation Plan

**Date:** 2025-11-05
**Objective:** Implement deck system to organize questions into blocks with tracking and summary capabilities

## 1. Feature Overview

A **deck** represents a session of 10 questions that the user works through. Each deck:
- Has a unique identifier
- Contains exactly 10 questions (operations)
- Tracks all user responses for those questions
- Provides performance summary and statistics
- Serves as a logical grouping for practice sessions

### Key Benefits:
- Organize practice sessions into manageable units
- Track progress over time by comparing decks
- Enable session-based analytics (success rate per deck, time per deck, etc.)
- Foundation for future features (deck history, deck replay, deck difficulty progression)

## 2. Requirements Analysis

Based on the backlog document (03-create-deck.md), the feature requires:

1. **Initialize Deck**: Create deck object when a new block starts
   - Unique identifier
   - Creation timestamp

2. **Link Questions to Deck**: Associate each of the 10 questions with the deck
   - Store deck_id with each operation

3. **Store User Responses**: Link answers to both question and deck
   - Enable deck-specific performance tracking

4. **Deck Summary**: Compile statistics at end of block
   - Total questions answered
   - Correct/incorrect counts
   - Performance patterns
   - Store summary in database

## 3. Database Schema Design

### 3.1 New Table: `decks`

```sql
CREATE TABLE decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    status TEXT NOT NULL DEFAULT 'in_progress',  -- 'in_progress', 'completed', 'abandoned'
    total_questions INTEGER NOT NULL DEFAULT 0,
    correct_answers INTEGER NOT NULL DEFAULT 0,
    incorrect_answers INTEGER NOT NULL DEFAULT 0,
    total_time_seconds REAL NOT NULL DEFAULT 0.0,
    average_time_seconds REAL,
    accuracy_percentage REAL
);
```

**Field Descriptions:**
- `id`: Unique deck identifier
- `created_at`: Timestamp when deck was created
- `completed_at`: Timestamp when all questions were answered (NULL if in progress)
- `status`: Current state of the deck
  - `in_progress`: User is still answering questions
  - `completed`: All questions answered
  - `abandoned`: User started new deck without finishing this one
- `total_questions`: Number of questions in deck (should be 10)
- `correct_answers`: Count of correct responses
- `incorrect_answers`: Count of incorrect responses
- `total_time_seconds`: Sum of time spent on all questions
- `average_time_seconds`: Average time per question (computed on completion)
- `accuracy_percentage`: (correct_answers / total_questions) * 100

### 3.2 Modify Table: `operations`

Add `deck_id` column to link operations to decks:

```sql
ALTER TABLE operations ADD COLUMN deck_id INTEGER REFERENCES decks(id);
```

**Migration Note:** For existing operations without deck_id, leave as NULL (historical data).

### 3.3 Modify Table: `answers`

Add `deck_id` column for redundant tracking (enables fast deck-specific queries):

```sql
ALTER TABLE answers ADD COLUMN deck_id INTEGER REFERENCES decks(id);
```

**Rationale for Redundancy:** While deck_id can be joined through operations table, storing it directly in answers enables:
- Faster queries for deck statistics
- Simpler SQL for summary generation
- Better query performance as database grows

### 3.4 Indexes for Performance

```sql
CREATE INDEX idx_deck_operations ON operations(deck_id);
CREATE INDEX idx_deck_answers ON answers(deck_id);
CREATE INDEX idx_deck_status ON decks(status);
CREATE INDEX idx_deck_created ON decks(created_at DESC);
```

## 4. Implementation Architecture

### 4.1 Module Structure

```
src/
├── deck.rs (NEW)
│   ├── Deck struct
│   ├── DeckStatus enum
│   ├── DeckSummary struct
│   └── Helper functions for statistics
├── database.rs (MODIFY)
│   ├── Add decks table creation
│   ├── Modify operations/answers table creation
│   ├── create_deck() -> Result<i64>
│   ├── get_deck(deck_id) -> Result<Option<Deck>>
│   ├── update_deck_summary(deck_id, summary) -> Result<()>
│   ├── complete_deck(deck_id) -> Result<()>
│   ├── abandon_deck(deck_id) -> Result<()>
│   ├── get_recent_decks(limit) -> Result<Vec<Deck>>
│   ├── get_deck_operations(deck_id) -> Result<Vec<OperationRecord>>
│   ├── get_deck_answers(deck_id) -> Result<Vec<AnswerRecord>>
│   └── Modify insert_operation/insert_answer to accept deck_id
├── operations.rs (NO CHANGE)
│   └── No modifications needed
├── gui.rs (MODIFY)
│   ├── Add current_deck_id field to MemoryPracticeApp
│   ├── Create deck on app initialization
│   ├── Pass deck_id when storing operations/answers
│   ├── Calculate and display deck summary in results
│   ├── Mark deck as completed when showing results
│   └── Create new deck when starting new block
└── lib.rs (MODIFY)
    └── Add `pub mod deck;`
```

### 4.2 Deck Module (`src/deck.rs`)

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum DeckStatus {
    InProgress,
    Completed,
    Abandoned,
}

impl DeckStatus {
    pub fn as_str(&self) -> &str {
        match self {
            DeckStatus::InProgress => "in_progress",
            DeckStatus::Completed => "completed",
            DeckStatus::Abandoned => "abandoned",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_progress" => Some(DeckStatus::InProgress),
            "completed" => Some(DeckStatus::Completed),
            "abandoned" => Some(DeckStatus::Abandoned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: DeckStatus,
    pub total_questions: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub total_time_seconds: f64,
    pub average_time_seconds: Option<f64>,
    pub accuracy_percentage: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DeckSummary {
    pub total_questions: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub total_time_seconds: f64,
    pub average_time_seconds: f64,
    pub accuracy_percentage: f64,
}

impl DeckSummary {
    pub fn from_results(results: &[(bool, f64)]) -> Self {
        let total_questions = results.len() as i32;
        let correct_answers = results.iter().filter(|(correct, _)| *correct).count() as i32;
        let incorrect_answers = total_questions - correct_answers;
        let total_time_seconds: f64 = results.iter().map(|(_, time)| time).sum();
        let average_time_seconds = if total_questions > 0 {
            total_time_seconds / total_questions as f64
        } else {
            0.0
        };
        let accuracy_percentage = if total_questions > 0 {
            (correct_answers as f64 / total_questions as f64) * 100.0
        } else {
            0.0
        };

        DeckSummary {
            total_questions,
            correct_answers,
            incorrect_answers,
            total_time_seconds,
            average_time_seconds,
            accuracy_percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_status_as_str() {
        assert_eq!(DeckStatus::InProgress.as_str(), "in_progress");
        assert_eq!(DeckStatus::Completed.as_str(), "completed");
        assert_eq!(DeckStatus::Abandoned.as_str(), "abandoned");
    }

    #[test]
    fn test_deck_status_from_str() {
        assert_eq!(
            DeckStatus::from_str("in_progress"),
            Some(DeckStatus::InProgress)
        );
        assert_eq!(
            DeckStatus::from_str("completed"),
            Some(DeckStatus::Completed)
        );
        assert_eq!(
            DeckStatus::from_str("abandoned"),
            Some(DeckStatus::Abandoned)
        );
        assert_eq!(DeckStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_deck_summary_calculation() {
        let results = vec![
            (true, 2.0),
            (true, 3.0),
            (false, 5.0),
            (true, 1.0),
        ];

        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.total_questions, 4);
        assert_eq!(summary.correct_answers, 3);
        assert_eq!(summary.incorrect_answers, 1);
        assert_eq!(summary.total_time_seconds, 11.0);
        assert_eq!(summary.average_time_seconds, 2.75);
        assert_eq!(summary.accuracy_percentage, 75.0);
    }

    #[test]
    fn test_deck_summary_all_correct() {
        let results = vec![(true, 1.0), (true, 2.0), (true, 3.0)];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.accuracy_percentage, 100.0);
        assert_eq!(summary.correct_answers, 3);
        assert_eq!(summary.incorrect_answers, 0);
    }

    #[test]
    fn test_deck_summary_all_incorrect() {
        let results = vec![(false, 1.0), (false, 2.0), (false, 3.0)];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.accuracy_percentage, 0.0);
        assert_eq!(summary.correct_answers, 0);
        assert_eq!(summary.incorrect_answers, 3);
    }

    #[test]
    fn test_deck_summary_empty() {
        let results: Vec<(bool, f64)> = vec![];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.total_questions, 0);
        assert_eq!(summary.accuracy_percentage, 0.0);
    }
}
```

### 4.3 Database Modifications

**Add to `Database::new()`:**

```rust
// Create decks table
conn.execute(
    "CREATE TABLE IF NOT EXISTS decks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        completed_at DATETIME,
        status TEXT NOT NULL DEFAULT 'in_progress',
        total_questions INTEGER NOT NULL DEFAULT 0,
        correct_answers INTEGER NOT NULL DEFAULT 0,
        incorrect_answers INTEGER NOT NULL DEFAULT 0,
        total_time_seconds REAL NOT NULL DEFAULT 0.0,
        average_time_seconds REAL,
        accuracy_percentage REAL
    )",
    [],
)?;

// Add deck_id to operations (migration safe - allows NULL)
conn.execute(
    "ALTER TABLE operations ADD COLUMN IF NOT EXISTS deck_id INTEGER REFERENCES decks(id)",
    [],
)?;

// Add deck_id to answers
conn.execute(
    "ALTER TABLE answers ADD COLUMN IF NOT EXISTS deck_id INTEGER REFERENCES decks(id)",
    [],
)?;

// Create indexes
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_deck_operations ON operations(deck_id)",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_deck_answers ON answers(deck_id)",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_deck_status ON decks(status)",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_deck_created ON decks(created_at DESC)",
    [],
)?;
```

**New Database Methods:**

```rust
use crate::deck::{Deck, DeckStatus, DeckSummary};

impl Database {
    pub fn create_deck(&self) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO decks (status) VALUES (?1)",
            [DeckStatus::InProgress.as_str()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_deck(&self, deck_id: i64) -> Result<Option<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks WHERE id = ?1"
        )?;

        let mut rows = stmt.query([deck_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Deck {
                id: row.get(0)?,
                created_at: row.get(1)?,
                completed_at: row.get(2)?,
                status: DeckStatus::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(DeckStatus::InProgress),
                total_questions: row.get(4)?,
                correct_answers: row.get(5)?,
                incorrect_answers: row.get(6)?,
                total_time_seconds: row.get(7)?,
                average_time_seconds: row.get(8)?,
                accuracy_percentage: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_deck_summary(&self, deck_id: i64, summary: &DeckSummary) -> Result<()> {
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

    pub fn complete_deck(&self, deck_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET status = ?1, completed_at = CURRENT_TIMESTAMP WHERE id = ?2",
            [DeckStatus::Completed.as_str(), &deck_id.to_string()],
        )?;
        Ok(())
    }

    pub fn abandon_deck(&self, deck_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE decks SET status = ?1 WHERE id = ?2",
            [DeckStatus::Abandoned.as_str(), &deck_id.to_string()],
        )?;
        Ok(())
    }

    pub fn get_recent_decks(&self, limit: i32) -> Result<Vec<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, completed_at, status, total_questions,
                    correct_answers, incorrect_answers, total_time_seconds,
                    average_time_seconds, accuracy_percentage
             FROM decks
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map([limit], |row| {
            Ok(Deck {
                id: row.get(0)?,
                created_at: row.get(1)?,
                completed_at: row.get(2)?,
                status: DeckStatus::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(DeckStatus::InProgress),
                total_questions: row.get(4)?,
                correct_answers: row.get(5)?,
                incorrect_answers: row.get(6)?,
                total_time_seconds: row.get(7)?,
                average_time_seconds: row.get(8)?,
                accuracy_percentage: row.get(9)?,
            })
        })?;

        let mut decks = Vec::new();
        for deck_result in rows {
            decks.push(deck_result?);
        }
        Ok(decks)
    }

    pub fn count_decks(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))?;
        Ok(count)
    }

    // Modify existing methods to accept optional deck_id
    pub fn insert_operation(
        &self,
        operation_type: &str,
        operand1: i32,
        operand2: i32,
        result: i32,
        deck_id: Option<i64>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO operations (operation_type, operand1, operand2, result, deck_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                operation_type,
                operand1,
                operand2,
                result,
                deck_id
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_answer(
        &self,
        operation_id: i64,
        user_answer: i32,
        is_correct: bool,
        time_spent_seconds: f64,
        deck_id: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO answers (operation_id, user_answer, is_correct, time_spent_seconds, deck_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                operation_id,
                user_answer,
                is_correct as i32,
                time_spent_seconds,
                deck_id
            ],
        )?;
        Ok(())
    }
}
```

### 4.4 GUI Modifications

**Changes to `MemoryPracticeApp`:**

```rust
use crate::deck::{DeckSummary};

pub struct MemoryPracticeApp {
    db: Arc<Database>,
    questions: Vec<Operation>,
    current_question_index: usize,
    user_answers: Vec<String>,
    question_start_time: Option<Instant>,
    results: Vec<QuestionResult>,
    state: AppState,
    current_deck_id: Option<i64>,  // NEW
}

impl MemoryPracticeApp {
    pub fn new(db: Arc<Database>) -> Self {
        // Create a new deck
        let deck_id = db.create_deck().ok();

        let questions = generate_question_block(10);
        let user_answers = vec![String::new(); 10];

        Self {
            db,
            questions,
            current_question_index: 0,
            user_answers,
            question_start_time: Some(Instant::now()),
            results: Vec::new(),
            state: AppState::ShowingQuestions,
            current_deck_id: deck_id,
        }
    }

    fn submit_current_answer(&mut self) {
        // ... existing validation code ...

        if let Ok(user_answer) = answer_str.parse::<i32>() {
            let question = &self.questions[self.current_question_index];
            let is_correct = question.check_answer(user_answer);
            let time_spent = self
                .question_start_time
                .map(|start| start.elapsed().as_secs_f64())
                .unwrap_or(0.0);

            // Store in database with deck_id
            if let Ok(operation_id) = self.db.insert_operation(
                question.operation_type.as_str(),
                question.operand1,
                question.operand2,
                question.result,
                self.current_deck_id,  // Pass deck_id
            ) {
                let _ = self.db.insert_answer(
                    operation_id,
                    user_answer,
                    is_correct,
                    time_spent,
                    self.current_deck_id,  // Pass deck_id
                );
            }

            // Store result for display
            self.results.push(QuestionResult {
                operation: question.clone(),
                user_answer,
                is_correct,
                time_spent,
            });

            // Move to next question
            self.current_question_index += 1;

            if self.current_question_index >= self.questions.len() {
                // Calculate and save deck summary
                self.complete_current_deck();
                self.state = AppState::ShowingResults;
            } else {
                self.question_start_time = Some(Instant::now());
            }
        }
    }

    fn complete_current_deck(&mut self) {
        if let Some(deck_id) = self.current_deck_id {
            // Collect results as (is_correct, time_spent) tuples
            let results_data: Vec<(bool, f64)> = self
                .results
                .iter()
                .map(|r| (r.is_correct, r.time_spent))
                .collect();

            // Calculate summary
            let summary = DeckSummary::from_results(&results_data);

            // Update deck with summary
            let _ = self.db.update_deck_summary(deck_id, &summary);

            // Mark deck as completed
            let _ = self.db.complete_deck(deck_id);
        }
    }

    fn start_new_block(&mut self) {
        // Mark previous deck as abandoned if not completed
        if let Some(deck_id) = self.current_deck_id {
            if self.state != AppState::ShowingResults {
                let _ = self.db.abandon_deck(deck_id);
            }
        }

        // Create new deck
        self.current_deck_id = self.db.create_deck().ok();

        self.questions = generate_question_block(10);
        self.user_answers = vec![String::new(); 10];
        self.current_question_index = 0;
        self.question_start_time = Some(Instant::now());
        self.results.clear();
        self.state = AppState::ShowingQuestions;
    }

    pub fn get_current_deck_id(&self) -> Option<i64> {
        self.current_deck_id
    }
}
```

**UI Display for Deck Summary (in results section):**

```rust
// In ShowingResults state
ui.heading("Deck Results");
ui.add_space(10.0);

if let Some(deck_id) = self.current_deck_id {
    ui.label(format!("Deck ID: {}", deck_id));
}

let correct_count = self.results.iter().filter(|r| r.is_correct).count();
let total = self.results.len();
let accuracy = if total > 0 {
    (correct_count as f64 / total as f64) * 100.0
} else {
    0.0
};

ui.label(format!("Score: {}/{} ({:.1}%)", correct_count, total, accuracy));
ui.add_space(10.0);

// Display detailed results...
```

## 5. Testing Strategy

### 5.1 Unit Tests

**`tests/deck_unit_tests.rs`:**
- Test DeckStatus enum conversions
- Test DeckSummary calculations with various scenarios
- Test empty deck summary
- Test 100% correct/incorrect scenarios
- Test edge cases (0 questions, negative times)

**`src/database.rs` tests:**
- Test deck creation
- Test deck retrieval
- Test deck summary update
- Test deck completion
- Test deck abandonment
- Test get_recent_decks
- Test count_decks
- Test operations/answers linked to decks

### 5.2 Integration Tests

**`tests/deck_integration_tests.rs`:**
- Test complete deck workflow: create → add operations → add answers → complete
- Test deck with mixed correct/incorrect answers
- Test multiple decks creation
- Test deck status transitions
- Test querying operations/answers by deck_id
- Test deck summary accuracy

### 5.3 E2E Tests

**Modify existing `tests/e2e_tests.rs`:**
- Test app creates deck on initialization
- Test deck_id is passed when storing operations
- Test deck_id is passed when storing answers
- Test deck completion when all questions answered
- Test deck abandonment when starting new block
- Test new deck creation for new block
- Test deck summary display in results

## 6. Migration Strategy

### Phase 1: Database Schema Update
1. Add decks table
2. Add deck_id columns to operations and answers
3. Create indexes
4. Verify schema changes on existing database

### Phase 2: Core Implementation
1. Implement deck module with structs and enums
2. Add database methods for deck operations
3. Write and pass unit tests

### Phase 3: Integration
1. Modify database methods to accept deck_id
2. Update GUI to create and track decks
3. Implement deck summary calculation
4. Write integration tests

### Phase 4: Testing & Refinement
1. Update all existing tests to handle optional deck_id
2. Run full test suite
3. Manual testing of deck workflow
4. Verify backward compatibility with old data

### Phase 5: Future Enhancements Setup
1. Add deck history view (future feature)
2. Add deck statistics dashboard (future feature)
3. Add deck export functionality (future feature)

## 7. Backward Compatibility

**Handling Existing Data:**
- Operations and answers without deck_id will have NULL values
- This is acceptable and represents historical data before deck feature
- Queries should handle NULL deck_id gracefully
- Future features can optionally exclude NULL deck_id data

**API Changes:**
- `insert_operation()` signature changes to accept `Option<i64>` for deck_id
- `insert_answer()` signature changes to accept `Option<i64>` for deck_id
- All existing tests need to be updated to pass `None` or `Some(deck_id)`

## 8. Success Criteria

- [ ] Decks table created successfully
- [ ] Each question block creates a new deck
- [ ] All operations linked to deck_id
- [ ] All answers linked to deck_id
- [ ] Deck summary calculated correctly
- [ ] Deck status transitions work (in_progress → completed)
- [ ] Abandoned decks handled properly
- [ ] All existing tests updated and passing
- [ ] New deck-specific tests passing (>95% coverage)
- [ ] Performance acceptable (<50ms for deck operations)
- [ ] Build and pre-commit hooks pass

## 9. Future Enhancements (Post-MVP)

1. **Deck History View:**
   - Display list of recent decks
   - Show summary for each deck
   - Allow clicking to see detailed results

2. **Deck Analytics:**
   - Accuracy trend over time
   - Average time trend
   - Difficulty progression
   - Performance by operation type

3. **Deck Tags/Labels:**
   - Tag decks by topic or difficulty
   - Filter decks by tags
   - Group statistics by tags

4. **Deck Export/Import:**
   - Export deck data as JSON/CSV
   - Import decks from other sources
   - Share decks with other users

5. **Deck Replay:**
   - Retry a specific deck
   - Practice only incorrect answers from a deck
   - Time-trial mode for previous decks

6. **Deck Recommendations:**
   - Suggest decks based on weak areas
   - Adaptive difficulty based on deck performance
   - Spaced repetition integration at deck level

## 10. Implementation Timeline Estimate

- **Phase 1 (Database):** 1-2 hours
- **Phase 2 (Core):** 2-3 hours
- **Phase 3 (Integration):** 2-3 hours
- **Phase 4 (Testing):** 2-3 hours
- **Phase 5 (Future Prep):** 1 hour
- **Total:** 8-12 hours

## 11. Open Questions

1. **Abandoned Decks Cleanup:** Should we automatically abandon old in_progress decks after X days?
   - Proposal: Keep all decks, add filter to hide abandoned decks in UI

2. **Deck Size:** Should deck size be configurable (not always 10)?
   - Proposal: Start with fixed 10, make configurable in future enhancement

3. **Partial Deck Completion:** What if user answers 5/10 questions then quits?
   - Proposal: Mark as abandoned, store partial summary

4. **Deck Naming:** Should users be able to name decks?
   - Proposal: Auto-generate name like "Deck #123 - Nov 5, 2025", allow editing later

5. **Multiple Active Decks:** Can user have multiple in_progress decks?
   - Proposal: Only one active deck at a time, starting new block abandons previous

## 12. API Examples

### Creating a Deck
```rust
let deck_id = db.create_deck()?;
```

### Storing Operation with Deck
```rust
let op_id = db.insert_operation("ADD", 5, 10, 15, Some(deck_id))?;
```

### Storing Answer with Deck
```rust
db.insert_answer(op_id, 15, true, 2.5, Some(deck_id))?;
```

### Completing a Deck
```rust
let summary = DeckSummary::from_results(&results);
db.update_deck_summary(deck_id, &summary)?;
db.complete_deck(deck_id)?;
```

### Retrieving Recent Decks
```rust
let recent = db.get_recent_decks(10)?;
for deck in recent {
    println!("Deck {} - {}% accuracy", deck.id, deck.accuracy_percentage.unwrap_or(0.0));
}
```

## 13. Dependencies

**No new external dependencies required.**

All functionality can be implemented using existing dependencies:
- `rusqlite` for database operations
- `chrono` for datetime handling (if not already added from spaced repetition)
- Standard library for everything else

If `chrono` is not yet added:
```toml
[dependencies]
chrono = "0.4"
```

## References

- Current database schema in `src/database.rs`
- Current GUI implementation in `src/gui.rs`
- Backlog requirement: `backlog/03-create-deck.md`

# Spaced Repetition Implementation Plan

**Date:** 2025-11-05
**Objective:** Integrate spaced repetition system to enhance knowledge retention using optimal review intervals

## 1. Library Selection

After researching available Rust crates for spaced repetition, the following options were evaluated:

### Option A: `sra` crate (v0.1.0)
- **Pros:**
  - Simple, focused implementation of SM-2 algorithm
  - Well-documented (100% documentation coverage)
  - MIT/Apache dual license
  - Minimal dependencies
- **Cons:**
  - Relatively new (v0.1.0)
  - Limited features beyond basic SM-2
- **API Overview:**
  - `SM2` struct with `review()` method
  - `Quality` enum for rating responses
  - Takes: repetitions, interval, ease_factor, quality
  - Returns: updated scheduling parameters

### Option B: `spaced-rs` crate (v0.3.1)
- **Pros:**
  - More mature (v0.3.1)
  - Self-adjusting behavior for forgetting curves
  - Incorporates randomization to spread reviews
  - 100% documented
- **Cons:**
  - GPL-3.0 license (more restrictive)
  - More complex API
- **API Overview:**
  - `SchedulingData` struct for item-specific data
  - `UpdateParameters` for configuration
  - `UserReview` enum for difficulty assessment
  - Functions: `schedule()`, `compute_interval()`, `update_adjusting_factor()`

### Option C: `fsrs` crate (FSRS algorithm)
- **Pros:**
  - Modern algorithm (successor to SM-2)
  - Developed by Open Spaced Repetition community
  - More sophisticated than SM-2
- **Cons:**
  - More complex to integrate
  - Heavier dependency
  - May be overkill for basic math operations

### **Recommendation: Use `sra` crate**

**Rationale:**
1. Simple, clean API perfect for our use case
2. Permissive MIT/Apache licensing
3. SM-2 algorithm is well-proven and sufficient for math operations
4. Minimal dependencies keep the project lightweight
5. Easy to migrate to FSRS later if needed

## 2. Database Schema Design

### New Table: `review_items`

```sql
CREATE TABLE review_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id INTEGER UNIQUE NOT NULL,
    repetitions INTEGER NOT NULL DEFAULT 0,
    interval INTEGER NOT NULL DEFAULT 0,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    next_review_date TEXT NOT NULL,  -- ISO 8601 format
    last_reviewed_date TEXT,          -- ISO 8601 format
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (operation_id) REFERENCES operations(id)
);
```

**Field Descriptions:**
- `operation_id`: Links to the specific math operation (unique - one review item per operation)
- `repetitions`: Count of consecutive successful recalls (SM-2 parameter)
- `interval`: Days until next review (SM-2 parameter)
- `ease_factor`: Difficulty multiplier (SM-2 parameter, default 2.5)
- `next_review_date`: When this item should be reviewed next
- `last_reviewed_date`: When this item was last reviewed

### Index for Performance

```sql
CREATE INDEX idx_next_review ON review_items(next_review_date);
```

## 3. Implementation Architecture

### Module Structure

```
src/
├── spaced_repetition.rs  (NEW)
│   ├── ReviewItem struct
│   ├── ReviewScheduler wrapper around sra::SM2
│   ├── Quality mapping to sra::Quality
│   └── Helper functions for date calculations
├── database.rs (MODIFY)
│   ├── Add review_items table creation
│   ├── insert_review_item()
│   ├── update_review_item()
│   ├── get_review_item()
│   ├── get_due_reviews(date) -> Vec<ReviewItem>
│   └── count_due_reviews(date) -> i64
├── operations.rs (MINIMAL CHANGE)
│   └── No changes needed (Operation struct stays the same)
├── gui.rs (MODIFY)
│   ├── Fetch due reviews before generating new questions
│   ├── Display indicator for review vs new items
│   ├── Map user performance to Quality rating
│   └── Update review scheduling after each answer
└── lib.rs (MODIFY)
    └── Add `pub mod spaced_repetition;`
```

### 3.1 Spaced Repetition Module (`src/spaced_repetition.rs`)

```rust
use chrono::{DateTime, Utc, Duration};
use sra::sm_2::{SM2, Quality};

pub struct ReviewItem {
    pub id: Option<i64>,
    pub operation_id: i64,
    pub repetitions: i32,
    pub interval: i32,
    pub ease_factor: f32,
    pub next_review_date: DateTime<Utc>,
    pub last_reviewed_date: Option<DateTime<Utc>>,
}

pub struct ReviewScheduler {
    sm2: SM2,
}

impl ReviewScheduler {
    pub fn new() -> Self {
        Self { sm2: SM2::new() }
    }

    pub fn process_review(&self, item: &ReviewItem, quality: Quality)
        -> (i32, i32, f32, DateTime<Utc>) {
        // Call sra::SM2::review()
        // Calculate next_review_date from interval
        // Return updated parameters
    }
}

// Helper to convert performance to Quality
pub fn performance_to_quality(is_correct: bool, time_spent: f64) -> Quality {
    // Map correctness + speed to Quality enum
    // Quality scale: 0-5 in SM-2
    // 0-2: Incorrect or very slow
    // 3: Correct but difficult
    // 4: Correct with hesitation
    // 5: Perfect recall
}
```

### 3.2 Database Modifications

**Add to `Database::new()`:**
```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS review_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        operation_id INTEGER UNIQUE NOT NULL,
        repetitions INTEGER NOT NULL DEFAULT 0,
        interval INTEGER NOT NULL DEFAULT 0,
        ease_factor REAL NOT NULL DEFAULT 2.5,
        next_review_date TEXT NOT NULL,
        last_reviewed_date TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (operation_id) REFERENCES operations(id)
    )",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_next_review
     ON review_items(next_review_date)",
    [],
)?;
```

**New Methods:**
- `insert_review_item(operation_id, next_review_date) -> Result<i64>`
- `update_review_item(review_item: &ReviewItem) -> Result<()>`
- `get_review_item(operation_id) -> Result<Option<ReviewItem>>`
- `get_due_reviews(before_date: DateTime<Utc>) -> Result<Vec<ReviewItem>>`
- `count_due_reviews(before_date: DateTime<Utc>) -> Result<i64>`

### 3.3 GUI Modifications

**Changes to `MemoryPracticeApp`:**

```rust
pub struct MemoryPracticeApp {
    db: Arc<Database>,
    questions: Vec<Operation>,
    review_items: Vec<ReviewItem>,  // NEW
    is_review_mode: Vec<bool>,       // NEW: track which questions are reviews
    current_question_index: usize,
    user_answers: Vec<String>,
    question_start_time: Option<Instant>,
    results: Vec<QuestionResult>,
    state: AppState,
    scheduler: ReviewScheduler,      // NEW
}
```

**Question Generation Logic:**

```rust
fn generate_question_block(&mut self) {
    let due_reviews = self.db.get_due_reviews(Utc::now()).unwrap_or_default();
    let review_count = due_reviews.len().min(10);

    // Load operations for due reviews
    for review in &due_reviews[..review_count] {
        let operation = self.db.get_operation(review.operation_id).unwrap();
        self.questions.push(operation);
        self.review_items.push(review.clone());
        self.is_review_mode.push(true);
    }

    // Fill remaining with new random questions
    for _ in review_count..10 {
        self.questions.push(Operation::generate_random());
        self.is_review_mode.push(false);
    }
}
```

**Answer Processing:**

```rust
fn submit_current_answer(&mut self) {
    // ... existing code ...

    if self.is_review_mode[self.current_question_index] {
        // Update existing review item
        let quality = performance_to_quality(is_correct, time_spent);
        let review_item = &self.review_items[self.current_question_index];
        let (reps, interval, ease, next_date) =
            self.scheduler.process_review(review_item, quality);

        let updated = ReviewItem {
            id: review_item.id,
            operation_id: review_item.operation_id,
            repetitions: reps,
            interval,
            ease_factor: ease,
            next_review_date: next_date,
            last_reviewed_date: Some(Utc::now()),
        };
        self.db.update_review_item(&updated).unwrap();
    } else {
        // Create new review item for new question
        let next_date = if is_correct {
            Utc::now() + Duration::days(1)
        } else {
            Utc::now() + Duration::minutes(10)  // Retry soon
        };
        self.db.insert_review_item(operation_id, next_date).unwrap();
    }
}
```

**UI Indicators:**

```rust
// In the question display section
if self.is_review_mode[self.current_question_index] {
    ui.label(egui::RichText::new("📚 Review")
        .color(egui::Color32::from_rgb(100, 150, 200)));
} else {
    ui.label(egui::RichText::new("✨ New")
        .color(egui::Color32::from_rgb(100, 200, 100)));
}
```

## 4. Dependencies

**Add to `Cargo.toml`:**

```toml
[dependencies]
eframe = "0.29"
rusqlite = { version = "0.32", features = ["bundled"] }
rand = "0.8"
sra = "0.1"           # NEW: Spaced repetition algorithm
chrono = "0.4"        # NEW: Date/time handling
```

## 5. Testing Strategy

### Unit Tests

**`tests/spaced_repetition_tests.rs`:**
- Test ReviewScheduler initialization
- Test performance_to_quality mapping
- Test date calculations
- Test SM-2 parameter updates

**`tests/database_review_tests.rs`:**
- Test review_items table creation
- Test insert/update/get review items
- Test get_due_reviews with various dates
- Test count_due_reviews
- Test unique constraint on operation_id

### Integration Tests

**`tests/review_workflow_tests.rs`:**
- Test complete review cycle: answer → schedule → retrieve
- Test mixing new questions with reviews
- Test review scheduling based on performance
- Test progression through SM-2 intervals (1 day, 6 days, etc.)

### E2E Tests

**`tests/gui_review_tests.rs`:**
- Test review indicator display
- Test review priority in question blocks
- Test scheduling updates through GUI
- Test multiple review sessions

## 6. Migration Strategy

### Phase 1: Database Migration
1. Add review_items table to existing databases
2. Backfill existing operations as review items with next_review_date = now()
3. Verify data integrity

### Phase 2: Core Implementation
1. Implement spaced_repetition module
2. Add database methods
3. Write and pass unit tests

### Phase 3: GUI Integration
1. Modify question generation
2. Add UI indicators
3. Update answer processing
4. Write integration tests

### Phase 4: Testing & Refinement
1. Run full test suite
2. Manual testing of review workflow
3. Adjust Quality mapping based on UX
4. Performance testing with large review queues

## 7. Success Criteria

- [ ] All existing tests continue to pass
- [ ] New tests for spaced repetition pass (>95% coverage)
- [ ] Due reviews are prioritized in question blocks
- [ ] Review scheduling follows SM-2 algorithm correctly
- [ ] UI clearly indicates review vs new items
- [ ] Performance remains acceptable (<100ms for review lookups)
- [ ] Build and pre-commit hooks pass

## 8. Future Enhancements (Post-MVP)

1. **Statistics Dashboard:**
   - Show retention rates
   - Display upcoming review counts
   - Graph learning progress

2. **Advanced Scheduling:**
   - Upgrade to FSRS algorithm
   - Custom interval modifiers
   - Review load balancing

3. **Review Modes:**
   - "Cram mode" for upcoming reviews
   - "Weak areas" mode for low ease_factor items
   - Filter by operation type

4. **Export/Import:**
   - Export review history
   - Import from other SRS systems (Anki, etc.)

## 9. Open Questions

1. **Quality Mapping:** What time thresholds should determine Quality levels?
   - Proposal: <2s = Perfect (5), 2-5s = Good (4), 5-10s = OK (3), >10s or wrong = Hard (0-2)

2. **Initial Interval:** Should new items start with 1 day or shorter (e.g., 10 minutes)?
   - Proposal: Start with 1 day for correct answers, 10 minutes for incorrect

3. **Review Limit:** Should we cap reviews per session to avoid overwhelming users?
   - Proposal: Max 10 reviews per session, fill rest with new items

4. **Lapsed Items:** How to handle items not reviewed for a long time?
   - Proposal: Reset to minimal interval if >30 days overdue

## 10. Implementation Timeline Estimate

- **Phase 1 (Database):** 1-2 hours
- **Phase 2 (Core):** 2-3 hours
- **Phase 3 (GUI):** 2-3 hours
- **Phase 4 (Testing):** 2-3 hours
- **Total:** 7-11 hours

## References

- [SM-2 Algorithm Description](https://en.wikipedia.org/wiki/SuperMemo#Description_of_SM-2_algorithm)
- [`sra` crate documentation](https://docs.rs/sra/latest/sra/)
- [Open Spaced Repetition](https://github.com/open-spaced-repetition)
- [Implementing SM-2 in Rust](https://borretti.me/article/implementing-sm2-in-rust)

-- Initial database schema
-- Contains all tables, columns, and indexes for the memory practice application

-- Create operations table
CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    operand1 INTEGER NOT NULL,
    operand2 INTEGER NOT NULL,
    result INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deck_id INTEGER REFERENCES decks(id)
);

-- Create answers table
CREATE TABLE IF NOT EXISTS answers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id INTEGER NOT NULL,
    user_answer INTEGER NOT NULL,
    is_correct INTEGER NOT NULL,
    time_spent_seconds REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deck_id INTEGER REFERENCES decks(id),
    FOREIGN KEY (operation_id) REFERENCES operations(id)
);

-- Create decks table
CREATE TABLE IF NOT EXISTS decks (
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
);

-- Create review_items table for spaced repetition
CREATE TABLE IF NOT EXISTS review_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id INTEGER UNIQUE NOT NULL,
    repetitions INTEGER NOT NULL DEFAULT 0,
    interval INTEGER NOT NULL DEFAULT 0,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    next_review_date TEXT NOT NULL,
    last_reviewed_date TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (operation_id) REFERENCES operations(id)
);

-- Create indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_deck_operations ON operations(deck_id);
CREATE INDEX IF NOT EXISTS idx_deck_answers ON answers(deck_id);
CREATE INDEX IF NOT EXISTS idx_deck_status ON decks(status);
CREATE INDEX IF NOT EXISTS idx_deck_created ON decks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_next_review ON review_items(next_review_date);

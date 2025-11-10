-- Create index on operations deck_id
CREATE INDEX IF NOT EXISTS idx_deck_operations ON operations(deck_id);

-- Create index on answers deck_id
CREATE INDEX IF NOT EXISTS idx_deck_answers ON answers(deck_id);

-- Create index on decks status
CREATE INDEX IF NOT EXISTS idx_deck_status ON decks(status);

-- Create index on decks created_at
CREATE INDEX IF NOT EXISTS idx_deck_created ON decks(created_at DESC);

-- Create index on review_items next_review_date
CREATE INDEX IF NOT EXISTS idx_next_review ON review_items(next_review_date);

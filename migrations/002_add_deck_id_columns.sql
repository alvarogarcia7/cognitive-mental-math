-- Add deck_id to operations table
ALTER TABLE operations ADD COLUMN deck_id INTEGER REFERENCES decks(id);

-- Add deck_id to answers table
ALTER TABLE answers ADD COLUMN deck_id INTEGER REFERENCES decks(id);

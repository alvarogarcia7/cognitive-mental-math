use memory_practice::database::Database;
use memory_practice::deck::DeckSummary;

fn main() {
    let db = Database::new("/tmp/perf_test.db").expect("Failed to create test DB");

    // Create some test decks and answers with different operation types

    // Deck 1
    let deck_id = db.create_deck().unwrap();

    // Addition operations
    let op1 = db
        .insert_operation("addition", 5, 3, 8, Some(deck_id))
        .unwrap();
    db.insert_answer(op1, 8, true, 1.5, Some(deck_id)).unwrap();
    db.insert_answer(op1, 8, true, 1.2, Some(deck_id)).unwrap();

    // Subtraction operations
    let op2 = db
        .insert_operation("subtraction", 10, 4, 6, Some(deck_id))
        .unwrap();
    db.insert_answer(op2, 6, true, 2.1, Some(deck_id)).unwrap();

    // Multiplication operations
    let op3 = db
        .insert_operation("multiplication", 7, 8, 56, Some(deck_id))
        .unwrap();
    db.insert_answer(op3, 56, true, 0.8, Some(deck_id)).unwrap();
    db.insert_answer(op3, 56, true, 0.9, Some(deck_id)).unwrap();

    // Complete deck 1
    let summary = DeckSummary {
        total_questions: 3,
        correct_answers: 3,
        incorrect_answers: 0,
        total_time_seconds: 6.5,
        average_time_seconds: 2.17,
        accuracy_percentage: 100.0,
    };
    db.update_deck_summary(deck_id, &summary).unwrap();
    db.complete_deck(deck_id).unwrap();

    // Deck 2 - faster performance
    let deck_id = db.create_deck().unwrap();

    let op4 = db
        .insert_operation("addition", 2, 3, 5, Some(deck_id))
        .unwrap();
    db.insert_answer(op4, 5, true, 0.8, Some(deck_id)).unwrap();
    db.insert_answer(op4, 5, true, 0.9, Some(deck_id)).unwrap();

    let op5 = db
        .insert_operation("subtraction", 15, 7, 8, Some(deck_id))
        .unwrap();
    db.insert_answer(op5, 8, true, 1.5, Some(deck_id)).unwrap();

    let op6 = db
        .insert_operation("multiplication", 6, 7, 42, Some(deck_id))
        .unwrap();
    db.insert_answer(op6, 42, true, 0.7, Some(deck_id)).unwrap();

    let summary = DeckSummary {
        total_questions: 3,
        correct_answers: 3,
        incorrect_answers: 0,
        total_time_seconds: 4.6,
        average_time_seconds: 1.53,
        accuracy_percentage: 100.0,
    };
    db.update_deck_summary(deck_id, &summary).unwrap();
    db.complete_deck(deck_id).unwrap();

    // Deck 3
    let deck_id = db.create_deck().unwrap();

    let op7 = db
        .insert_operation("addition", 8, 9, 17, Some(deck_id))
        .unwrap();
    db.insert_answer(op7, 17, true, 0.9, Some(deck_id)).unwrap();

    let op8 = db
        .insert_operation("subtraction", 20, 8, 12, Some(deck_id))
        .unwrap();
    db.insert_answer(op8, 12, true, 1.2, Some(deck_id)).unwrap();

    let op9 = db
        .insert_operation("multiplication", 9, 4, 36, Some(deck_id))
        .unwrap();
    db.insert_answer(op9, 36, true, 0.6, Some(deck_id)).unwrap();

    let summary = DeckSummary {
        total_questions: 3,
        correct_answers: 3,
        incorrect_answers: 0,
        total_time_seconds: 2.7,
        average_time_seconds: 0.9,
        accuracy_percentage: 100.0,
    };
    db.update_deck_summary(deck_id, &summary).unwrap();
    db.complete_deck(deck_id).unwrap();

    println!("Test database created at /tmp/perf_test.db with sample data");
}

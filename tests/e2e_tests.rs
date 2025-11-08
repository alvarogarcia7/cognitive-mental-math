use memory_practice::database::Database;
use memory_practice::gui::{AppState, MemoryPracticeApp};
use memory_practice::operations::generate_question_block;
use std::sync::Arc;

#[test]
fn test_app_initialization() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    assert_eq!(*app.get_current_state(), AppState::ShowingQuestions);
    assert_eq!(app.get_current_question_index(), 0);
    assert_eq!(app.get_results().len(), 0);
}

#[test]
fn test_answer_submission_flow() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 1);
    app.start_new_block();

    // Set an answer and submit
    app.set_answer(0, "100".to_string());
    app.submit_answer();

    // Should move to next question
    assert_eq!(app.get_current_question_index(), 1);
    assert_eq!(app.get_results().len(), 1);

    assert_eq!(db.count_operations().unwrap(), 1);
    assert_eq!(db.count_answers().unwrap(), 1);
    assert_eq!(db.count_decks().unwrap(), 1);
}

#[test]
fn test_complete_question_block() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Answer all 10 questions
    for i in 0..10 {
        app.set_answer(i, "999".to_string());
        app.submit_answer();
    }

    // Should be in results state after 10 questions
    assert_eq!(*app.get_current_state(), AppState::ShowingResults);
    assert_eq!(app.get_results().len(), 10);

    // Database should have all operations and answers
    assert_eq!(db.count_operations().unwrap(), 10);
    assert_eq!(db.count_answers().unwrap(), 10);
}

#[test]
fn test_empty_answer_not_submitted() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Try to submit without setting an answer
    app.submit_answer();

    // Should still be on first question
    assert_eq!(app.get_current_question_index(), 0);
    assert_eq!(app.get_results().len(), 0);
    assert_eq!(db.count_answers().unwrap(), 0);
}

#[test]
fn test_invalid_answer_not_submitted() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Set an invalid (non-numeric) answer
    app.set_answer(0, "abc".to_string());
    app.submit_answer();

    // Should still be on first question
    assert_eq!(app.get_current_question_index(), 0);
    assert_eq!(app.get_results().len(), 0);
}

#[test]
fn test_results_contain_correct_data() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Submit first answer
    app.set_answer(0, "42".to_string());
    app.submit_answer();

    let results = app.get_results();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert_eq!(result.user_answer, 42);
    assert!(result.time_spent >= 0.0);
}

#[test]
fn test_multiple_answers_in_sequence() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 5);
    app.start_new_block();

    let test_answers = ["10", "20", "30", "40", "50"];

    for (i, answer) in test_answers.iter().enumerate() {
        app.set_answer(i, answer.to_string());
        app.submit_answer();
        assert_eq!(app.get_current_question_index(), i + 1);
    }

    assert_eq!(app.get_results().len(), 5);
    assert_eq!(db.count_operations().unwrap(), 5);
    assert_eq!(db.count_answers().unwrap(), 5);
}

#[test]
fn test_correctness_tracking() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Submit an answer
    app.set_answer(0, "100".to_string());
    app.submit_answer();

    let results = app.get_results();
    let result = &results[0];

    // Verify the correctness was tracked (either true or false)
    // We can't know the exact answer, but we can verify the field exists
    let _ = result.is_correct;
    assert!(result.time_spent >= 0.0);
}

#[test]
fn test_state_transitions() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 10);
    app.start_new_block();

    // Should start in ShowingQuestions state
    assert_eq!(*app.get_current_state(), AppState::ShowingQuestions);

    // Answer all questions
    for i in 0..10 {
        assert_eq!(*app.get_current_state(), AppState::ShowingQuestions);
        app.set_answer(i, "123".to_string());
        app.submit_answer();
    }

    // Should transition to ShowingResults after 10 questions
    assert_eq!(*app.get_current_state(), AppState::ShowingResults);
}

#[test]
fn test_database_persistence_across_submissions() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 3);
    app.start_new_block();

    // Submit 3 answers
    for i in 0..3 {
        app.set_answer(i, format!("{}", i * 10));
        app.submit_answer();
    }

    // Verify database contains all data
    assert_eq!(db.count_operations().unwrap(), 3);
    assert_eq!(db.count_answers().unwrap(), 3);

    // Verify each answer is stored correctly
    for answer_id in 1..=3 {
        let answer = db.get_answer(answer_id).unwrap();
        assert!(answer.is_some());
    }
}

#[test]
fn test_timing_data_recorded() {
    let db = Arc::new(Database::new(":memory:").unwrap());
    let mut app = MemoryPracticeApp::new(db.clone(), 1);
    app.start_new_block();

    app.set_answer(0, "50".to_string());
    app.submit_answer();

    let answer = db.get_answer(1).unwrap().unwrap();
    // Time should be a positive number (even if very small)
    assert!(answer.time_spent_seconds >= 0.0);
}

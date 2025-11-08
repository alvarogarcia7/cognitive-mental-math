use crate::database::Database;
use crate::deck::DeckSummary;
use crate::operations::{Operation, generate_question_block};
use eframe::egui;
use std::sync::Arc;
use std::time::Instant;

pub struct MemoryPracticeApp {
    db: Arc<Database>,
    questions: Vec<Operation>,
    current_question_index: usize,
    user_answers: Vec<String>,
    question_start_time: Option<Instant>,
    results: Vec<QuestionResult>,
    state: AppState,
    current_deck_id: Option<i64>,
    questions_per_block: usize,
}

#[derive(Debug, Clone)]
pub struct QuestionResult {
    pub operation: Operation,
    pub user_answer: i32,
    pub is_correct: bool,
    pub time_spent: f64,
}

#[derive(Debug, PartialEq)]
pub enum AppState {
    ShowingQuestions,
    ShowingResults,
}

impl MemoryPracticeApp {
    pub fn new(db: Arc<Database>, questions_per_block: usize) -> Self {
        Self {
            db,
            questions: Vec::new(),
            current_question_index: 0,
            user_answers: Vec::new(),
            question_start_time: None,
            results: Vec::new(),
            state: AppState::ShowingResults,
            current_deck_id: None,
            questions_per_block,
        }
    }

    fn submit_current_answer(&mut self) {
        if self.current_question_index >= self.questions.len() {
            return;
        }

        let answer_str = &self.user_answers[self.current_question_index];
        if answer_str.is_empty() {
            return;
        }

        if let Ok(user_answer) = answer_str.parse::<i32>() {
            let question = &self.questions[self.current_question_index];
            let is_correct = question.check_answer(user_answer);
            let time_spent = self
                .question_start_time
                .map(|start| start.elapsed().as_secs_f64())
                .unwrap_or(0.0);

            // Store result in memory for display and later batch write
            self.results.push(QuestionResult {
                operation: question.clone(),
                user_answer,
                is_correct,
                time_spent,
            });

            // Move to next question
            self.current_question_index += 1;

            if self.current_question_index >= self.questions.len() {
                // All questions answered - write results to database and complete deck
                self.complete_current_deck();
                self.state = AppState::ShowingResults;
            } else {
                self.question_start_time = Some(Instant::now());
            }
        }
    }

    fn write_results_to_database(&self) {
        if let Some(deck_id) = self.current_deck_id {
            // Write all results to database
            for result in &self.results {
                // Insert operation
                if let Ok(operation_id) = self.db.insert_operation(
                    result.operation.operation_type.as_str(),
                    result.operation.operand1,
                    result.operation.operand2,
                    result.operation.result,
                    Some(deck_id),
                ) {
                    // Insert answer
                    let _ = self.db.insert_answer(
                        operation_id,
                        result.user_answer,
                        result.is_correct,
                        result.time_spent,
                        Some(deck_id),
                    );
                }
            }
        }
    }

    fn complete_current_deck(&mut self) {
        if let Some(deck_id) = self.current_deck_id {
            // Write all results to database first
            self.write_results_to_database();

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

    pub fn start_new_block(&mut self) {
        // Mark previous deck as abandoned if not completed
        if let Some(deck_id) = self.current_deck_id {
            if self.state != AppState::ShowingResults {
                let _ = self.db.abandon_deck(deck_id);
            }
        }

        // Create new deck
        self.current_deck_id = self.db.create_deck().ok();

        self.questions = generate_question_block(self.questions_per_block);
        self.user_answers = vec![String::new(); self.questions_per_block];
        self.current_question_index = 0;
        self.question_start_time = Some(Instant::now());
        self.results.clear();
        self.state = AppState::ShowingQuestions;
    }

    // Helper methods for testing
    pub fn get_current_state(&self) -> &AppState {
        &self.state
    }

    pub fn get_current_question_index(&self) -> usize {
        self.current_question_index
    }

    pub fn get_results(&self) -> &[QuestionResult] {
        &self.results
    }

    pub fn set_answer(&mut self, index: usize, answer: String) {
        if index < self.user_answers.len() {
            self.user_answers[index] = answer;
        }
    }

    pub fn submit_answer(&mut self) {
        self.submit_current_answer();
    }

    pub fn get_current_deck_id(&self) -> Option<i64> {
        self.current_deck_id
    }
}

impl Drop for MemoryPracticeApp {
    fn drop(&mut self) {
        // When app closes, if deck is in progress (not completed), write results and abandon
        if let Some(_deck_id) = self.current_deck_id {
            if self.state != AppState::ShowingResults {
                // Write any answers that were collected to database before abandoning
                self.write_results_to_database();
                let _ = self.db.abandon_deck(_deck_id);
            }
        }
    }
}

impl eframe::App for MemoryPracticeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::ShowingQuestions => {
                    ui.heading("Memory Practice - Math Operations");
                    ui.add_space(20.0);

                    if self.current_question_index < self.questions.len() {
                        let question = &self.questions[self.current_question_index];

                        ui.label(format!(
                            "Question {} of {}",
                            self.current_question_index + 1,
                            self.questions.len()
                        ));
                        ui.add_space(20.0);

                        // Display the question in large font
                        ui.heading(
                            egui::RichText::new(question.to_string())
                                .size(32.0)
                                .strong(),
                        );
                        ui.add_space(20.0);

                        // Answer input
                        ui.horizontal(|ui| {
                            ui.label("Your answer:");
                            let response = ui.text_edit_singleline(
                                &mut self.user_answers[self.current_question_index],
                            );

                            // Auto-focus the text input
                            if !response.lost_focus() {
                                response.request_focus();
                            }

                            // Submit on Enter key
                            if response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                self.submit_current_answer();
                            }
                        });

                        ui.add_space(10.0);

                        if ui.button("Submit Answer").clicked() {
                            self.submit_current_answer();
                        }
                    }
                }
                AppState::ShowingResults => {
                    ui.heading("Deck Results");
                    ui.add_space(10.0);

                    if let Some(deck_id) = self.current_deck_id {
                        ui.label(format!("Deck ID: {}", deck_id));
                    }
                    ui.add_space(10.0);

                    let total = self.results.len();
                    if total == 0 {
                        ui.label("No results yet. Click 'Start New Block' to begin.");
                    } else {
                        let correct_count = self.results.iter().filter(|r| r.is_correct).count();
                        let average_time = if total > 0 {
                            self.results.iter().map(|r| r.time_spent).sum::<f64>() / total as f64
                        } else {
                            0.0
                        };
                        let accuracy = if total > 0 {
                            (correct_count as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };

                        ui.label(format!(
                            "Score: {}/{} ({:.1}%)",
                            correct_count, total, accuracy
                        ));
                        ui.label(format!("Average time: {:.2}s", average_time));
                        ui.add_space(20.0);

                        // Show detailed results
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (i, result) in self.results.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let status = if result.is_correct { "✓" } else { "✗" };
                                    let color = if result.is_correct {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::RED
                                    };

                                    ui.label(format!("{}.", i + 1));
                                    ui.label(result.operation.to_string().replace("?", ""));
                                    ui.label(result.operation.result.to_string());
                                    ui.label(format!("(Your answer: {})", result.user_answer));
                                    ui.label(egui::RichText::new(status).color(color).strong());
                                    ui.label(format!("{:.2}s", result.time_spent));
                                });
                            }
                        });
                    }

                    ui.add_space(20.0);

                    if ui.button("Start New Block").clicked() {
                        self.start_new_block();
                    }
                }
            }
        });
    }
}

pub fn run_app(db: Arc<Database>, is_test_mode: bool) -> Result<(), eframe::Error> {
    // In test mode, use 1 question per block; in production, use 10
    let questions_per_block = if is_test_mode { 1 } else { 10 };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Memory Practice",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MemoryPracticeApp::new(
                db.clone(),
                questions_per_block,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::DeckStatus;

    #[test]
    fn test_deck_abandoned_on_drop_during_questions() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        {
            let mut app = MemoryPracticeApp::new(db.clone(), 10);
            app.start_new_block();
            let deck_id = app.get_current_deck_id().expect("Deck should be created");
            // Verify deck was created as in_progress
            let deck = db
                .get_deck(deck_id)
                .expect("Database access should succeed")
                .expect("Deck should exist");
            assert_eq!(deck.status, DeckStatus::InProgress);
            // app goes out of scope here, Drop will be called
        }

        // After drop, the deck should be marked as abandoned
        let deck = db
            .get_deck(1)
            .expect("Database access should succeed")
            .expect("Deck should still exist");
        assert_eq!(
            deck.status,
            DeckStatus::Abandoned,
            "Deck should be abandoned after app drop"
        );
    }

    #[test]
    fn test_completed_deck_not_abandoned_on_drop() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let deck_id = {
            let mut app = MemoryPracticeApp::new(db.clone(), 10);
            app.start_new_block();
            let deck_id = app.get_current_deck_id().expect("Deck should be created");

            // Simulate completing the deck
            app.state = AppState::ShowingResults;
            app.complete_current_deck();

            deck_id
        };

        // After drop, the deck should still be completed (not abandoned)
        let deck = db
            .get_deck(deck_id)
            .expect("Database access should succeed")
            .expect("Deck should still exist");
        assert_eq!(
            deck.status,
            DeckStatus::Completed,
            "Completed deck should not be abandoned"
        );
    }

    #[test]
    fn test_app_uses_correct_question_count() {
        let db = Arc::new(Database::new(":memory:").unwrap());

        // Test mode: 1 question per block
        let mut app_test = MemoryPracticeApp::new(db.clone(), 1);
        app_test.start_new_block();
        assert_eq!(app_test.questions_per_block, 1);
        assert_eq!(app_test.questions.len(), 1);
        assert_eq!(app_test.user_answers.len(), 1);

        // Production mode: 10 questions per block
        let mut app_prod = MemoryPracticeApp::new(db.clone(), 10);
        app_prod.start_new_block();
        assert_eq!(app_prod.questions_per_block, 10);
        assert_eq!(app_prod.questions.len(), 10);
        assert_eq!(app_prod.user_answers.len(), 10);
    }

    #[test]
    fn test_answers_not_written_immediately() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let mut app = MemoryPracticeApp::new(db.clone(), 2);
        app.start_new_block();
        let _deck_id = app.get_current_deck_id().expect("Deck should be created");

        // Submit only first answer (not completing the deck)
        app.set_answer(0, "42".to_string());
        app.submit_answer();

        // Answer should be in memory
        assert_eq!(app.results.len(), 1);
        assert_eq!(app.results[0].user_answer, 42);

        // But NOT in the database yet (deck is incomplete)
        let operations_in_db = db
            .count_operations()
            .expect("Database access should succeed");
        assert_eq!(
            operations_in_db, 0,
            "Operations should not be written to database immediately"
        );
    }

    #[test]
    fn test_answers_written_on_completion() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let mut app = MemoryPracticeApp::new(db.clone(), 1);
        app.start_new_block();
        let _deck_id = app.get_current_deck_id().expect("Deck should be created");

        // Get the expected answer from the question
        let expected_answer = app.questions[0].result;

        // Submit the correct answer
        app.set_answer(0, expected_answer.to_string());
        app.submit_answer();

        // verify completion happened
        assert_eq!(app.state, AppState::ShowingResults);

        // Now answers should be in the database
        let operations_in_db = db
            .count_operations()
            .expect("Database access should succeed");
        assert_eq!(
            operations_in_db, 1,
            "Operations should be written when deck completes"
        );

        // Verify the operation and answer are correct
        let operation = db
            .get_operation(1)
            .expect("Database access should succeed")
            .expect("Operation should exist");
        assert!(
            !operation.operation_type.is_empty(),
            "Operation type should exist"
        );

        let answer = db
            .get_answer(1)
            .expect("Database access should succeed")
            .expect("Answer should exist");
        assert_eq!(answer.user_answer, expected_answer);
        assert!(answer.is_correct, "Answer should be marked as correct");
    }

    #[test]
    fn test_answers_written_on_drop_abandoned() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        {
            let mut app = MemoryPracticeApp::new(db.clone(), 2);
            app.start_new_block();
            let _deck_id = app.get_current_deck_id().expect("Deck should be created");

            // Submit partial answers (not completing the deck)
            app.set_answer(0, "42".to_string());
            app.submit_answer();

            // Verify not in database yet
            let operations_count = db
                .count_operations()
                .expect("Database access should succeed");
            assert_eq!(operations_count, 0, "Answers not yet written to database");

            // app drops here
        }

        // After drop, answers should be written and deck abandoned
        let operations_in_db = db
            .count_operations()
            .expect("Database access should succeed");
        assert_eq!(
            operations_in_db, 1,
            "Answers should be written when app closes with incomplete deck"
        );

        let deck = db
            .get_deck(1)
            .expect("Database access should succeed")
            .expect("Deck should exist");
        assert_eq!(
            deck.status,
            DeckStatus::Abandoned,
            "Incomplete deck should be abandoned on app close"
        );
    }

    #[test]
    fn test_multiple_answers_written_together() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let mut app = MemoryPracticeApp::new(db.clone(), 3);
        app.start_new_block();

        // Submit all answers
        for i in 0..3 {
            app.set_answer(i, format!("{}", i * 10));
            app.submit_answer();
        }

        // All should be written when deck completes
        let operations_in_db = db
            .count_operations()
            .expect("Database access should succeed");
        assert_eq!(
            operations_in_db, 3,
            "All answers should be written together"
        );

        // Verify all answers are in database
        for i in 1..=3 {
            let answer = db
                .get_answer(i as i64)
                .expect("Database access should succeed")
                .expect(&format!("Answer {} should exist", i));
            assert!(answer.user_answer >= 0);
        }
    }
}

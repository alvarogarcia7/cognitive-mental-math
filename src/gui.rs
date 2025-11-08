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
    pub fn new(db: Arc<Database>, questions: Vec<Operation>) -> Self {
        let user_answers = vec![String::new(); 10];

        // Create a new deck
        let current_deck_id = db.create_deck().ok();

        Self {
            db,
            questions,
            current_question_index: 0,
            user_answers,
            question_start_time: Some(Instant::now()),
            results: Vec::new(),
            state: AppState::ShowingQuestions,
            current_deck_id,
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

            // Store in database with deck_id
            if let Ok(operation_id) = self.db.insert_operation(
                question.operation_type.as_str(),
                question.operand1,
                question.operand2,
                question.result,
                self.current_deck_id,
            ) {
                let _ = self.db.insert_answer(
                    operation_id,
                    user_answer,
                    is_correct,
                    time_spent,
                    self.current_deck_id,
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
        // Mark in-progress deck as abandoned when app closes
        if let Some(deck_id) = self.current_deck_id {
            if self.state != AppState::ShowingResults {
                let _ = self.db.abandon_deck(deck_id);
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

                    let correct_count = self.results.iter().filter(|r| r.is_correct).count();
                    let total = self.results.len();
                    let average_time =
                        self.results.iter().map(|r| r.time_spent).sum::<f64>() / total as f64;
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

                    ui.add_space(20.0);

                    if ui.button("Start New Block").clicked() {
                        self.start_new_block();
                    }
                }
            }
        });
    }
}

pub fn run_app(db: Arc<Database>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Memory Practice",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MemoryPracticeApp::new(
                db,
                generate_question_block(10),
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
            let app = MemoryPracticeApp::new(db.clone(), generate_question_block(10));
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
            let mut app = MemoryPracticeApp::new(db.clone(), generate_question_block(10));
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
}

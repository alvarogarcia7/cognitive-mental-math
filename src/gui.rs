use crate::database::Database;
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
    pub fn new(db: Arc<Database>) -> Self {
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

            // Store in database
            if let Ok(operation_id) = self.db.insert_operation(
                question.operation_type.as_str(),
                question.operand1,
                question.operand2,
                question.result,
            ) {
                let _ = self
                    .db
                    .insert_answer(operation_id, user_answer, is_correct, time_spent);
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
                self.state = AppState::ShowingResults;
            } else {
                self.question_start_time = Some(Instant::now());
            }
        }
    }

    fn start_new_block(&mut self) {
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
                        ui.add_space(10.0);

                        // Display elapsed time
                        if let Some(start_time) = self.question_start_time {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            ui.label(format!("Time: {:.1}s", elapsed));
                            ctx.request_repaint(); // Keep updating the timer
                        }
                        ui.add_space(10.0);

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
                            if self.current_question_index == 0 && !response.lost_focus() {
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
                    ui.heading("Results");
                    ui.add_space(20.0);

                    let correct_count = self.results.iter().filter(|r| r.is_correct).count();
                    let total = self.results.len();
                    let average_time =
                        self.results.iter().map(|r| r.time_spent).sum::<f64>() / total as f64;

                    ui.label(format!("Score: {}/{}", correct_count, total));
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
        Box::new(|_cc| Ok(Box::new(MemoryPracticeApp::new(db)))),
    )
}

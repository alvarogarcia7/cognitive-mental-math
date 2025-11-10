use crate::answer_evaluator_service::AnswerEvaluatorService;
use crate::database::Database;
use crate::deck::DeckSummary;
use crate::operations::{Operation, OperationType};
use crate::spaced_repetition::{ReviewItem, ReviewScheduler};
use crate::time_format::format_time_difference;
use chrono::{DateTime, Utc};
use log::info;
use sra::sm_2::Quality;
use std::sync::Arc;

/// Result of answering a single question
#[derive(Debug, Clone)]
pub struct QuestionResult {
    pub operation: Operation,
    pub user_answer: i32,
    pub is_correct: bool,
    pub time_spent: f64,
    pub is_review: bool,
    pub original_operation_id: Option<i64>,
    /// Quality grade assigned to this answer (0-5, None if not yet graded)
    pub grade: Option<Quality>,
    /// Next review date for spaced repetition (None if answer not correct or not yet scheduled)
    pub next_review_date: Option<DateTime<Utc>>,
}

/// Service layer for quiz operations, decoupled from GUI
pub struct QuizService {
    db: Arc<Database>,
    evaluator_service: AnswerEvaluatorService,
}

impl QuizService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            evaluator_service: AnswerEvaluatorService::new(db.clone()),
            db,
        }
    }

    /// Process a user's answer to a question
    pub fn process_answer(
        &self,
        question: &Operation,
        user_answer: i32,
        time_spent: f64,
    ) -> QuestionResult {
        let is_correct = question.check_answer(user_answer);
        let is_review = question.id.is_some();
        let original_operation_id = question.id;

        QuestionResult {
            operation: question.clone(),
            user_answer,
            is_correct,
            time_spent,
            is_review,
            original_operation_id,
            grade: None,
            next_review_date: None,
        }
    }

    /// Write all results to database with proper review scheduling
    /// Returns updated results with grade and next_review_date populated
    pub fn persist_results(&self, results: &[QuestionResult], deck_id: i64) -> Vec<QuestionResult> {
        let scheduler = ReviewScheduler::new();
        let mut updated_results = Vec::new();

        for result in results {
            let question_str = format!(
                "{} {} {} = ?",
                result.operation.operand1,
                result.operation.operation_type.symbol(),
                result.operation.operand2
            );

            if result.is_review {
                // For reviews, update existing operation
                let updated =
                    self.persist_review_result(&scheduler, result, &question_str, deck_id);
                updated_results.push(updated);
            } else {
                // For new questions, create operation and review item
                let updated =
                    self.persist_new_question_result(&scheduler, result, &question_str, deck_id);
                updated_results.push(updated);
            }
        }

        updated_results
    }

    /// Persist a review result with updated scheduling
    /// Returns the result with grade and next_review_date populated
    fn persist_review_result(
        &self,
        scheduler: &ReviewScheduler,
        result: &QuestionResult,
        question_str: &str,
        deck_id: i64,
    ) -> QuestionResult {
        let mut updated_result = result.clone();

        if let Some(operation_id) = result.original_operation_id {
            if self
                .db
                .insert_answer(
                    operation_id,
                    result.user_answer,
                    result.is_correct,
                    result.time_spent,
                    Some(deck_id),
                )
                .is_ok()
            {
                if let Ok(Some(mut review_item)) = self.db.get_review_item(operation_id) {
                    let stats = self
                        .evaluator_service
                        .get_evaluator(result.operation.operation_type.as_str());

                    let quality = stats.evaluate_performance(result.is_correct, result.time_spent);
                    let (reps, interval, ease, next_date) =
                        scheduler.process_review(&review_item, quality);

                    let quality_str = Self::quality_to_string(quality);

                    info!(
                        "Review: {} | Quality: {} | Next review: {} | Reps: {}, Interval: {} days, Ease: {:.2}",
                        question_str,
                        quality_str,
                        format_time_difference(Utc::now(), next_date),
                        reps,
                        interval,
                        ease
                    );

                    review_item.repetitions = reps;
                    review_item.interval = interval;
                    review_item.ease_factor = ease;
                    review_item.next_review_date = next_date;
                    review_item.last_reviewed_date = Some(Utc::now());

                    let _ = self.db.update_review_item(&review_item);

                    // Update the result with grade and next review date
                    updated_result.grade = Some(quality);
                    updated_result.next_review_date = Some(next_date);
                }
            }
        }

        updated_result
    }

    /// Persist a new question result with initial review scheduling
    /// Returns the result with grade and next_review_date populated
    fn persist_new_question_result(
        &self,
        scheduler: &ReviewScheduler,
        result: &QuestionResult,
        question_str: &str,
        deck_id: i64,
    ) -> QuestionResult {
        let mut updated_result = result.clone();

        if let Ok(operation_id) = self.db.insert_operation(
            result.operation.operation_type.as_str(),
            result.operation.operand1,
            result.operation.operand2,
            result.operation.result,
            Some(deck_id),
        ) {
            if self
                .db
                .insert_answer(
                    operation_id,
                    result.user_answer,
                    result.is_correct,
                    result.time_spent,
                    Some(deck_id),
                )
                .is_ok()
            {
                let stats = self
                    .evaluator_service
                    .get_evaluator(result.operation.operation_type.as_str());

                let quality = stats.evaluate_performance(result.is_correct, result.time_spent);

                // Create a review item with SM-2 defaults and let the scheduler determine timing
                let mut review_item = ReviewItem {
                    id: None,
                    operation_id,
                    repetitions: 0,
                    interval: 0,
                    ease_factor: 2.5,
                    next_review_date: Utc::now(),
                    last_reviewed_date: None,
                };

                let (reps, interval, ease, next_date) =
                    scheduler.process_review(&review_item, quality);

                let quality_str = Self::quality_to_string(quality);

                info!(
                    "New question: {} | Quality: {} | First review: {} | Reps: {}, Interval: {} days, Ease: {:.2}",
                    question_str,
                    quality_str,
                    format_time_difference(Utc::now(), next_date),
                    reps,
                    interval,
                    ease
                );

                review_item.repetitions = reps;
                review_item.interval = interval;
                review_item.ease_factor = ease;
                review_item.next_review_date = next_date;

                let _ = self.db.insert_review_item(operation_id, next_date);
                let _ = self.db.update_review_item(&review_item);

                // Update the result with grade and next review date
                updated_result.grade = Some(quality);
                updated_result.next_review_date = Some(next_date);
            }
        }

        updated_result
    }

    /// Complete a deck with summary statistics
    pub fn complete_deck(&self, deck_id: i64, results: &[QuestionResult]) {
        // Collect results as (is_correct, time_spent) tuples
        let results_data: Vec<(bool, f64)> = results
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

    /// Fetch due review questions for a deck
    pub fn fetch_due_reviews(&self) -> Vec<Operation> {
        let mut questions = Vec::new();
        let now = Utc::now();

        if let Ok(due_reviews) = self.db.get_due_reviews(now) {
            let num_due = due_reviews.len();
            info!("Found {} review question(s) due for practice", num_due);

            for (idx, review_item) in due_reviews.iter().enumerate() {
                if let Ok(Some(op_record)) = self.db.get_operation(review_item.operation_id) {
                    if let Some(op_type) = OperationType::from_str(&op_record.operation_type) {
                        let mut operation =
                            Operation::new(op_type, op_record.operand1, op_record.operand2);
                        operation.id = Some(op_record.id);

                        if idx == 0 {
                            info!(
                                "First review question: {} {} {} = {} (Reps: {}, Current interval: {} days, Ease: {:.2})",
                                op_record.operand1,
                                operation.operation_type.symbol(),
                                op_record.operand2,
                                op_record.result,
                                review_item.repetitions,
                                review_item.interval,
                                review_item.ease_factor
                            );
                        }

                        questions.push(operation);
                    }
                }
            }
        }

        questions
    }

    /// Convert SM-2 quality grade to human-readable string
    pub fn quality_to_string(quality: Quality) -> String {
        match quality {
            Quality::Grade0 => "Grade0 (Incorrect)".to_string(),
            Quality::Grade3 => "Grade3 (Serious difficulty)".to_string(),
            Quality::Grade4 => "Grade4 (After hesitation)".to_string(),
            Quality::Grade5 => "Grade5 (Perfect)".to_string(),
            _ => "N/A".to_string(),
        }
    }
}

use chrono::{DateTime, Utc};
use clap::Parser;
use memory_practice::quiz_service::QuizService;
use memory_practice::spaced_repetition::{ReviewItem, ReviewScheduler};
use memory_practice::time_format::format_time_difference;
use sra::sm_2::Quality;

/// Demonstrates SM-2 spaced repetition scheduling with different quality grades
#[derive(Parser, Debug)]
#[command(name = "SM2 Scheduler")]
#[command(about = "Demonstrates SM-2 spaced repetition scheduling", long_about = None)]
struct Args {
    /// Current number of repetitions (non-negative integer)
    #[arg(
        value_name = "NUM",
        help = "Current number of repetitions (non-negative integer)"
    )]
    repetitions: i32,

    /// Current interval in days (non-negative integer)
    #[arg(
        value_name = "DAYS",
        help = "Current interval in days (non-negative integer)"
    )]
    interval: i32,

    /// Current ease factor (floating point, typically 1.3 - 2.6)
    #[arg(
        value_name = "FACTOR",
        help = "Current ease factor (floating point, typically 1.3 - 2.6)"
    )]
    ease_factor: f32,
}

fn main() {
    let args = Args::parse();

    let scheduler = ReviewScheduler::new();
    let now = Utc::now();

    let review_item = ReviewItem {
        id: Some(1),
        operation_id: 1,
        repetitions: args.repetitions,
        interval: args.interval,
        ease_factor: args.ease_factor,
        next_review_date: now,
        last_reviewed_date: None,
    };

    println!(
        "SM-2 Scheduling Results for: reps={}, interval={}, ease={:.2}",
        args.repetitions, args.interval, args.ease_factor
    );

    compute_and_print(&scheduler, &review_item, now, Quality::Grade0);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade3);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade4);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade5);
}

fn compute_and_print(
    review_scheduler: &ReviewScheduler,
    review_item: &ReviewItem,
    now: DateTime<Utc>,
    quality: Quality,
) {
    let (reps, interval, ease, next_date) = review_scheduler.process_review(review_item, quality);

    let grade_description = QuizService::quality_to_string(quality);

    let relative_next_date = format_time_difference(now, next_date);
    println!(
        "Grade: {} | Next review: {} | Reps: {} | Interval: {} | Ease: {:.2}",
        grade_description, relative_next_date, reps, interval, ease
    );
}

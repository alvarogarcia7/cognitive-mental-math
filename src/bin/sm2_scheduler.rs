use chrono::{DateTime, Utc};
use memory_practice::quiz_service::QuizService;
use memory_practice::spaced_repetition::{ReviewItem, ReviewScheduler};
use memory_practice::time_format::format_time_difference;
use sra::sm_2::Quality;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (repetitions, interval, ease_factor) = validate_input(args);

    let scheduler = ReviewScheduler::new();
    let now = Utc::now();

    let review_item = ReviewItem {
        id: Some(1),
        operation_id: 1,
        repetitions,
        interval,
        ease_factor,
        next_review_date: now,
        last_reviewed_date: None,
    };

    println!(
        "SM-2 Scheduling Results for: reps={}, interval={}, ease={:.2}",
        repetitions, interval, ease_factor
    );

    compute_and_print(&scheduler, &review_item, now, Quality::Grade0);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade3);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade4);
    compute_and_print(&scheduler, &review_item, now, Quality::Grade5);
}

fn validate_input(args: Vec<String>) -> (i32, i32, f32) {
    if args.len() != 4 {
        eprintln!("Usage: {} <repetitions> <interval> <ease_factor>", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  <repetitions>  Current number of repetitions (non-negative integer)");
        eprintln!("  <interval>     Current interval in days (non-negative integer)");
        eprintln!("  <ease_factor>  Current ease factor (floating point, typically 1.3 - 2.6)");
        eprintln!();
        eprintln!("Example: {} 3 10 2.5", args[0]);
        std::process::exit(1);
    }

    let repetitions: i32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: repetitions must be a non-negative integer");
            std::process::exit(1);
        }
    };

    let interval: i32 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: interval must be a non-negative integer");
            std::process::exit(1);
        }
    };

    let ease_factor: f32 = match args[3].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: ease_factor must be a valid floating point number");
            std::process::exit(1);
        }
    };
    (repetitions, interval, ease_factor)
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

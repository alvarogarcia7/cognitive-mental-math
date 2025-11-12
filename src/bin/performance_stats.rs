use chrono::Utc;
use clap::Parser;
use memory_practice::database::{Analytics, Database};
use memory_practice::spaced_repetition::AnswerTimedEvaluator;
use std::path::PathBuf;

/// Analyzes performance statistics across different time periods
#[derive(Parser, Debug)]
#[command(name = "Performance Stats")]
#[command(about = "Analyzes performance statistics across different time periods", long_about = None)]
struct Args {
    /// Path to the SQLite database file
    #[arg(
        value_name = "DATABASE_FILE",
        help = "Path to the SQLite database file"
    )]
    database_file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let db_path = args.database_file.to_string_lossy();

    let db = match Database::new(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch all statistics in 3 database queries (one per time period)
    let analytics = Analytics::new(&db.conn);
    let global_stats = match analytics.time_statistics().all_operations() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching global statistics: {}", e);
            std::process::exit(1);
        }
    };

    if global_stats.is_empty() {
        println!("No operation types found in the database.");
        return;
    }

    let analytics1 = Analytics::new(&db.conn);
    let last_30_days_stats = match analytics1.time_statistics().all_operations_last_30_days() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 30 days statistics: {}", e);
            std::process::exit(1);
        }
    };

    let analytics2 = Analytics::new(&db.conn);
    let last_10_decks_stats = match analytics2.time_statistics().all_operations_last_10_decks() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 10 decks statistics: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch accuracy statistics
    let analytics3 = Analytics::new(&db.conn);
    let global_accuracy_stats = match analytics3.accuracy().all_operations() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching global accuracy statistics: {}", e);
            std::process::exit(1);
        }
    };

    let analytics4 = Analytics::new(&db.conn);
    let last_30_days_accuracy_stats = match analytics4.accuracy().all_operations_last_30_days() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 30 days accuracy statistics: {}", e);
            std::process::exit(1);
        }
    };

    let analytics = Analytics::new(&db.conn);
    let result = analytics.accuracy().all_operations_last_10_decks();
    let last_10_decks_accuracy_stats = match result {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 10 decks accuracy statistics: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch total accuracy
    let total_accuracy = match analytics.accuracy().total_accuracy() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching total accuracy: {}", e);
            std::process::exit(1);
        }
    };

    let total_accuracy_last_30_days = analytics
        .accuracy()
        .total_accuracy_last_30_days()
        .unwrap_or((0, 0, 0.0));
    let total_accuracy_last_10_decks = analytics
        .accuracy()
        .total_accuracy_last_10_decks()
        .unwrap_or((0, 0, 0.0));

    // Calculate consecutive days streak
    let consecutive_days_streak = analytics.streak().calculate_consecutive_days().unwrap_or(0);

    // Get days with and without answers in the last 10 days
    let now = Utc::now();
    let days_with_answers = analytics
        .streak()
        .get_days_with_answers(now)
        .unwrap_or_default();
    let missing_days = analytics
        .streak()
        .get_missing_days(10, now)
        .unwrap_or_default();

    println!("Performance Analysis Report");
    println!("===========================");
    println!();
    println!("Consecutive Days Streak: {} days", consecutive_days_streak);
    if !days_with_answers.is_empty() {
        println!(
            "Days with answers (last 10 days): {}",
            days_with_answers.join(", ")
        );
    }
    if !missing_days.is_empty() {
        println!(
            "Days without answers (last 10 days): {}",
            missing_days.join(", ")
        );
    }
    println!();

    // Iterate through all operation types (sorted for consistent output)
    let mut operation_types: Vec<_> = global_stats.keys().cloned().collect();
    operation_types.sort();

    for op_type in operation_types {
        println!("Operation Type: {}", op_type);
        println!("{}", "-".repeat(60));

        // Look up statistics for this operation type
        let global = global_stats.get(&op_type).copied();
        let last_30 = last_30_days_stats.get(&op_type).copied();
        let last_10 = last_10_decks_stats.get(&op_type).copied();

        let global_accuracy = global_accuracy_stats.get(&op_type).copied();
        let last_30_accuracy = last_30_days_accuracy_stats.get(&op_type).copied();
        let last_10_accuracy = last_10_decks_accuracy_stats.get(&op_type).copied();

        // Print completion stats
        println!("Completed Operations:");
        print_accuracy_stats("Global (all time)", &global_accuracy);
        print_accuracy_stats("Last 30 days", &last_30_accuracy);
        print_accuracy_stats("Last 10 decks", &last_10_accuracy);

        println!();
        println!("Time Statistics (in seconds):");

        // Print global stats
        print_stats("Global (all time)", &global);

        // Print last 30 days stats
        if let (Some(global_eval), Some(last_30_eval)) = (global, last_30) {
            if stats_are_same(&global_eval, &last_30_eval) {
                println!("  Last 30 days - Same data");
            } else {
                print_stats("Last 30 days", &Some(last_30_eval));
                print_improvement(
                    global_eval.average,
                    last_30_eval.average,
                    "Last 30 days vs Global",
                );
            }
        } else {
            print_stats("Last 30 days", &last_30);
        }

        // Print last 10 decks stats
        if let (Some(global_eval), Some(last_10_eval)) = (global, last_10) {
            if stats_are_same(&global_eval, &last_10_eval) {
                println!("  Last 10 decks - Same data");
            } else {
                print_stats("Last 10 decks", &Some(last_10_eval));
                print_improvement(
                    global_eval.average,
                    last_10_eval.average,
                    "Last 10 decks vs Global",
                );
            }
        } else {
            print_stats("Last 10 decks", &last_10);
        }

        // Compare last 30 days vs last 10 decks
        if let (Some(last_30_eval), Some(last_10_eval)) = (last_30, last_10)
            && !stats_are_same(&last_30_eval, &last_10_eval)
        {
            print_improvement(
                last_30_eval.average,
                last_10_eval.average,
                "Last 10 decks vs Last 30 days",
            );
        }

        println!();
    }

    // Print overall accuracy statistics
    println!();
    println!("Overall Accuracy Statistics");
    println!("===========================");
    println!();
    println!("Total Completed Operations:");
    print_accuracy_stats("Global (all time)", &Some(total_accuracy));
    if total_accuracy_last_30_days.1 > 0 {
        // Check if data is the same
        if total_accuracy == total_accuracy_last_30_days {
            println!("  Last 30 days - Same data");
        } else {
            print_accuracy_stats("Last 30 days", &Some(total_accuracy_last_30_days));
        }
    }
    if total_accuracy_last_10_decks.1 > 0 {
        // Check if data is the same
        if total_accuracy == total_accuracy_last_10_decks {
            println!("  Last 10 decks - Same data");
        } else {
            print_accuracy_stats("Last 10 decks", &Some(total_accuracy_last_10_decks));
        }
    }
}

/// Print statistics for a given time period
fn print_stats(label: &str, stats: &Option<AnswerTimedEvaluator>) {
    match stats {
        Some(eval) => {
            println!(
                "  {} - Average: {:.2}s, Std Dev: {:.2}s",
                label, eval.average, eval.standard_deviation
            );
        }
        None => {
            println!("  {} - No data available", label);
        }
    }
}

/// Print accuracy statistics
fn print_accuracy_stats(label: &str, stats: &Option<(i64, i64, f64)>) {
    match stats {
        Some((correct, total, accuracy)) => {
            println!(
                "  {} - {}/{} correct ({:.1}%)",
                label, correct, total, accuracy
            );
        }
        None => {
            println!("  {} - No data available", label);
        }
    }
}

/// Check if two timing evaluators have the same stats (within tolerance)
fn stats_are_same(eval1: &AnswerTimedEvaluator, eval2: &AnswerTimedEvaluator) -> bool {
    // Consider stats the same if average and std dev are equal within 0.001 tolerance
    (eval1.average - eval2.average).abs() < 0.001
        && (eval1.standard_deviation - eval2.standard_deviation).abs() < 0.001
}

/// Print improvement (or decline) between two average times
fn print_improvement(from_avg: f64, to_avg: f64, label: &str) {
    let improvement = from_avg - to_avg;
    let improvement_percent = (improvement / from_avg) * 100.0;

    if improvement > 0.001 {
        println!(
            "    ✓ {} - Improvement: {:.2}s ({:.1}%) faster",
            label, improvement, improvement_percent
        );
    } else if improvement < -0.001 {
        println!(
            "    ✗ {} - Decline: {:.2}s ({:.1}%) slower",
            label, -improvement, -improvement_percent
        );
    } else {
        println!("    • {} - No significant change", label);
    }
}

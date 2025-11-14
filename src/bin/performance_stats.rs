use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use memory_practice::database::analytics::{
    AccuracyRepository, StreakRepository, TimeStatisticsRepository,
};
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

    /// Disable colored output
    #[arg(long, help = "Disable colored output")]
    no_color: bool,
}

fn main() {
    let args = Args::parse();
    let db_path = args.database_file.to_string_lossy();
    let use_color = !args.no_color;

    let db = match Database::new(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch all statistics in 3 database queries (one per time period)
    let analytics = Analytics::new(&db.conn);
    let global_stats = match TimeStatisticsRepository::new(analytics.conn).all_operations() {
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

    let last_30_days_stats =
        match TimeStatisticsRepository::new(analytics.conn).all_operations_last_30_days() {
            Ok(stats) => stats,
            Err(e) => {
                eprintln!("Error fetching last 30 days statistics: {}", e);
                std::process::exit(1);
            }
        };

    let last_10_decks_stats =
        match TimeStatisticsRepository::new(analytics.conn).all_operations_last_10_decks() {
            Ok(stats) => stats,
            Err(e) => {
                eprintln!("Error fetching last 10 decks statistics: {}", e);
                std::process::exit(1);
            }
        };

    // Fetch accuracy statistics
    let global_accuracy_stats = match AccuracyRepository::new(analytics.conn).all_operations() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching global accuracy statistics: {}", e);
            std::process::exit(1);
        }
    };

    let last_30_days_accuracy_stats =
        match AccuracyRepository::new(analytics.conn).all_operations_last_30_days() {
            Ok(stats) => stats,
            Err(e) => {
                eprintln!("Error fetching last 30 days accuracy statistics: {}", e);
                std::process::exit(1);
            }
        };

    let analytics = Analytics::new(&db.conn);
    let result = AccuracyRepository::new(analytics.conn).all_operations_last_10_decks();
    let last_10_decks_accuracy_stats = match result {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 10 decks accuracy statistics: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch total accuracy
    let total_accuracy = match AccuracyRepository::new(analytics.conn).total_accuracy() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching total accuracy: {}", e);
            std::process::exit(1);
        }
    };

    let total_accuracy_last_30_days = AccuracyRepository::new(analytics.conn)
        .total_accuracy_last_30_days()
        .unwrap_or((0, 0, 0.0));
    let total_accuracy_last_10_decks = AccuracyRepository::new(analytics.conn)
        .total_accuracy_last_10_decks()
        .unwrap_or((0, 0, 0.0));

    // Calculate consecutive days streak
    let consecutive_days_streak = StreakRepository::new(analytics.conn)
        .calculate_consecutive_days()
        .unwrap_or(0);

    // Get days with and without answers in the last 10 days
    let now = Utc::now();
    let days_with_answers = StreakRepository::new(analytics.conn)
        .get_days_with_answers(now)
        .unwrap_or_default();
    let missing_days = StreakRepository::new(analytics.conn)
        .get_missing_days(10, now)
        .unwrap_or_default();

    let header = if use_color {
        "Performance Analysis Report".cyan().bold().to_string()
    } else {
        "Performance Analysis Report".to_string()
    };
    println!("{}", header);
    println!("===========================");
    println!();
    let streak_label = if use_color {
        "Consecutive Days Streak:".yellow().to_string()
    } else {
        "Consecutive Days Streak:".to_string()
    };
    println!("{} {} days", streak_label, consecutive_days_streak);
    if !days_with_answers.is_empty() {
        let days_label = if use_color {
            "Days with answers (last 10 days):".green().to_string()
        } else {
            "Days with answers (last 10 days):".to_string()
        };
        println!(
            "{} {}",
            days_label,
            days_with_answers.join(", ")
        );
    }
    if !missing_days.is_empty() {
        let missing_label = if use_color {
            "Days without answers (last 10 days):".red().to_string()
        } else {
            "Days without answers (last 10 days):".to_string()
        };
        println!(
            "{} {}",
            missing_label,
            missing_days.join(", ")
        );
    }
    println!();

    // Iterate through all operation types (sorted for consistent output)
    let mut operation_types: Vec<_> = global_stats.keys().cloned().collect();
    operation_types.sort();

    for op_type in operation_types {
        let op_label = if use_color {
            format!("Operation Type: {}", op_type.magenta().bold())
        } else {
            format!("Operation Type: {}", op_type)
        };
        println!("{}", op_label);
        println!("{}", "-".repeat(60));

        // Look up statistics for this operation type
        let global = global_stats.get(&op_type).copied();
        let last_30 = last_30_days_stats.get(&op_type).copied();
        let last_10 = last_10_decks_stats.get(&op_type).copied();

        let global_accuracy = global_accuracy_stats.get(&op_type).copied();
        let last_30_accuracy = last_30_days_accuracy_stats.get(&op_type).copied();
        let last_10_accuracy = last_10_decks_accuracy_stats.get(&op_type).copied();

        // Print completion stats
        let completion_label = if use_color {
            "Completed Operations:".blue().bold().to_string()
        } else {
            "Completed Operations:".to_string()
        };
        println!("{}", completion_label);
        print_accuracy_stats("Global (all time)", &global_accuracy, use_color);
        print_accuracy_stats("Last 30 days", &last_30_accuracy, use_color);
        print_accuracy_stats("Last 10 decks", &last_10_accuracy, use_color);

        println!();
        let time_label = if use_color {
            "Time Statistics (in seconds):".blue().bold().to_string()
        } else {
            "Time Statistics (in seconds):".to_string()
        };
        println!("{}", time_label);

        // Print global stats
        print_stats("Global (all time)", &global, use_color);

        // Print last 30 days stats
        if let (Some(global_eval), Some(last_30_eval)) = (global, last_30) {
            if stats_are_same(&global_eval, &last_30_eval) {
                println!("  Last 30 days - Same data");
            } else {
                print_stats("Last 30 days", &Some(last_30_eval), use_color);
                print_improvement(
                    global_eval.average,
                    last_30_eval.average,
                    "Last 30 days vs Global",
                    use_color,
                );
            }
        } else {
            print_stats("Last 30 days", &last_30, use_color);
        }

        // Print last 10 decks stats
        if let (Some(global_eval), Some(last_10_eval)) = (global, last_10) {
            if stats_are_same(&global_eval, &last_10_eval) {
                println!("  Last 10 decks - Same data");
            } else {
                print_stats("Last 10 decks", &Some(last_10_eval), use_color);
                print_improvement(
                    global_eval.average,
                    last_10_eval.average,
                    "Last 10 decks vs Global",
                    use_color,
                );
            }
        } else {
            print_stats("Last 10 decks", &last_10, use_color);
        }

        // Compare last 30 days vs last 10 decks
        if let (Some(last_30_eval), Some(last_10_eval)) = (last_30, last_10)
            && !stats_are_same(&last_30_eval, &last_10_eval)
        {
            print_improvement(
                last_30_eval.average,
                last_10_eval.average,
                "Last 10 decks vs Last 30 days",
                use_color,
            );
        }

        println!();
    }

    // Print overall accuracy statistics
    println!();
    let overall_label = if use_color {
        "Overall Accuracy Statistics".cyan().bold().to_string()
    } else {
        "Overall Accuracy Statistics".to_string()
    };
    println!("{}", overall_label);
    println!("===========================");
    println!();
    let total_label = if use_color {
        "Total Completed Operations:".blue().bold().to_string()
    } else {
        "Total Completed Operations:".to_string()
    };
    println!("{}", total_label);
    print_accuracy_stats("Global (all time)", &Some(total_accuracy), use_color);
    if total_accuracy_last_30_days.1 > 0 {
        // Check if data is the same
        if total_accuracy == total_accuracy_last_30_days {
            println!("  Last 30 days - Same data");
        } else {
            print_accuracy_stats("Last 30 days", &Some(total_accuracy_last_30_days), use_color);
        }
    }
    if total_accuracy_last_10_decks.1 > 0 {
        // Check if data is the same
        if total_accuracy == total_accuracy_last_10_decks {
            println!("  Last 10 decks - Same data");
        } else {
            print_accuracy_stats("Last 10 decks", &Some(total_accuracy_last_10_decks), use_color);
        }
    }
}

/// Print statistics for a given time period
fn print_stats(label: &str, stats: &Option<AnswerTimedEvaluator>, use_color: bool) {
    match stats {
        Some(eval) => {
            if use_color {
                let label_colored = label.cyan();
                let average_colored = format!("{:.2}s", eval.average).green();
                let std_dev_colored = format!("{:.2}s", eval.standard_deviation).yellow();
                println!(
                    "  {} - Average: {}, Std Dev: {}",
                    label_colored, average_colored, std_dev_colored
                );
            } else {
                println!(
                    "  {} - Average: {:.2}s, Std Dev: {:.2}s",
                    label, eval.average, eval.standard_deviation
                );
            }
        }
        None => {
            println!("  {} - No data available", label);
        }
    }
}

/// Print accuracy statistics
fn print_accuracy_stats(label: &str, stats: &Option<(i64, i64, f64)>, use_color: bool) {
    match stats {
        Some((correct, total, accuracy)) => {
            if use_color {
                let label_colored = label.cyan();
                let accuracy_percent = format!("{:.1}%", accuracy);
                let accuracy_colored = if *accuracy >= 90.0 {
                    accuracy_percent.green()
                } else if *accuracy >= 75.0 {
                    accuracy_percent.yellow()
                } else {
                    accuracy_percent.red()
                };
                println!(
                    "  {} - {}/{} correct ({})",
                    label_colored, correct, total, accuracy_colored
                );
            } else {
                println!(
                    "  {} - {}/{} correct ({:.1}%)",
                    label, correct, total, accuracy
                );
            }
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
fn print_improvement(from_avg: f64, to_avg: f64, label: &str, use_color: bool) {
    let improvement = from_avg - to_avg;
    let improvement_percent = (improvement / from_avg) * 100.0;

    if improvement > 0.001 {
        if use_color {
            let label_colored = label.cyan();
            let improvement_colored = format!("{:.2}s ({:.1}%)", improvement, improvement_percent).green();
            println!(
                "    ✓ {} - Improvement: {} faster",
                label_colored, improvement_colored
            );
        } else {
            println!(
                "    ✓ {} - Improvement: {:.2}s ({:.1}%) faster",
                label, improvement, improvement_percent
            );
        }
    } else if improvement < -0.001 {
        if use_color {
            let label_colored = label.cyan();
            let decline_colored = format!("{:.2}s ({:.1}%)", -improvement, -improvement_percent).red();
            println!(
                "    ✗ {} - Decline: {} slower",
                label_colored, decline_colored
            );
        } else {
            println!(
                "    ✗ {} - Decline: {:.2}s ({:.1}%) slower",
                label, -improvement, -improvement_percent
            );
        }
    } else {
        println!("    • {} - No significant change", label);
    }
}

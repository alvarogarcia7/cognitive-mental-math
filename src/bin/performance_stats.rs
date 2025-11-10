use memory_practice::database::Database;
use memory_practice::spaced_repetition::AnswerTimedEvaluator;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <database_file>", args[0]);
        eprintln!();
        eprintln!("Analyzes performance statistics across different time periods.");
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  <database_file>  Path to the SQLite database file");
        eprintln!();
        eprintln!("Example: {} ~/memory_practice.db", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];

    let db = match Database::new(db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    // Fetch all statistics in 3 database queries (one per time period)
    let global_stats = match db.compute_time_statistics_all_operations() {
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

    let last_30_days_stats = match db.compute_time_statistics_all_operations_last_30_days() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 30 days statistics: {}", e);
            std::process::exit(1);
        }
    };

    let last_10_decks_stats = match db.compute_time_statistics_all_operations_last_10_decks() {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching last 10 decks statistics: {}", e);
            std::process::exit(1);
        }
    };

    println!("Performance Analysis Report");
    println!("===========================");
    println!();

    // Iterate through all operation types (sorted for consistent output)
    let mut operation_types: Vec<_> = global_stats.keys().cloned().collect();
    operation_types.sort();

    for op_type in operation_types {
        println!("Operation Type: {}", op_type);
        println!("{}", "-".repeat(60));
        println!("Time Statistics (in seconds):");

        // Look up statistics for this operation type
        let global = global_stats.get(&op_type).copied();
        let last_30 = last_30_days_stats.get(&op_type).copied();
        let last_10 = last_10_decks_stats.get(&op_type).copied();

        // Print global stats
        print_stats("Global (all time)", &global);

        // Print last 30 days stats
        print_stats("Last 30 days", &last_30);
        if let (Some(global_eval), Some(last_30_eval)) = (global, last_30) {
            print_improvement(
                global_eval.average,
                last_30_eval.average,
                "Last 30 days vs Global",
            );
        }

        // Print last 10 decks stats
        print_stats("Last 10 decks", &last_10);
        if let (Some(global_eval), Some(last_10_eval)) = (global, last_10) {
            print_improvement(
                global_eval.average,
                last_10_eval.average,
                "Last 10 decks vs Global",
            );
        }

        // Compare last 30 days vs last 10 decks
        if let (Some(last_30_eval), Some(last_10_eval)) = (last_30, last_10) {
            print_improvement(
                last_30_eval.average,
                last_10_eval.average,
                "Last 10 decks vs Last 30 days",
            );
        }

        println!();
    }
}

/// Print statistics for a given time period
fn print_stats(label: &str, stats: &Option<AnswerTimedEvaluator>) {
    match stats {
        Some(eval) => {
            println!(
                "  {} - Average: {:.3}s, Std Dev: {:.3}s",
                label, eval.average, eval.standard_deviation
            );
        }
        None => {
            println!("  {} - No data available", label);
        }
    }
}

/// Print improvement (or decline) between two average times
fn print_improvement(from_avg: f64, to_avg: f64, label: &str) {
    let improvement = from_avg - to_avg;
    let improvement_percent = (improvement / from_avg) * 100.0;

    if improvement > 0.001 {
        println!(
            "    ✓ {} - Improvement: {:.3}s ({:.1}%) faster",
            label, improvement, improvement_percent
        );
    } else if improvement < -0.001 {
        println!(
            "    ✗ {} - Decline: {:.3}s ({:.1}%) slower",
            label, -improvement, -improvement_percent
        );
    } else {
        println!("    • {} - No significant change", label);
    }
}

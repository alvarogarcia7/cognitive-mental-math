use std::process::Command;

/// End-to-end test for performance_stats binary with colored output
#[test]
fn test_performance_stats_with_color() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "performance_stats",
            "--",
            "data/sample.db",
            "--override-date",
            "2025-11-17",
        ])
        .output()
        .expect("Failed to execute performance_stats");

    assert!(
        output.status.success(),
        "performance_stats command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Use insta to create a snapshot of the output
    insta::assert_snapshot!("performance_stats_color_output", stdout);
}

/// End-to-end test for performance_stats binary without colored output
#[test]
fn test_performance_stats_no_color() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "performance_stats",
            "--",
            "data/sample.db",
            "--no-color",
            "--override-date",
            "2025-11-17",
        ])
        .output()
        .expect("Failed to execute performance_stats");

    assert!(
        output.status.success(),
        "performance_stats command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Use insta to create a snapshot of the output
    insta::assert_snapshot!("performance_stats_no_color_output", stdout);
}

/// Test that performance_stats handles empty database gracefully
#[test]
fn test_performance_stats_empty_db() {
    // Create a temporary empty database file
    use std::fs;
    let temp_db = "temp_empty_test.db";

    // Clean up any existing temp file
    let _ = fs::remove_file(temp_db);

    let output = Command::new("cargo")
        .args(["run", "--bin", "performance_stats", "--", temp_db])
        .output()
        .expect("Failed to execute performance_stats");

    // Clean up the temp file
    let _ = fs::remove_file(temp_db);

    assert!(
        output.status.success(),
        "performance_stats should succeed with empty database: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No operation types found in the database"),
        "Expected message about no operation types, got: {}",
        stdout
    );
}

/// Regression test: operation types whose answers are all incorrect must still be listed
/// (time statistics only cover correct answers, so they must not decide what exists)
#[test]
fn test_performance_stats_lists_operations_with_no_correct_answers() {
    use memory_practice::database::Database;
    use std::fs;

    let temp_db = std::env::temp_dir().join(format!(
        "performance_stats_all_wrong_{}.db",
        std::process::id()
    ));
    let _ = fs::remove_file(&temp_db);

    // Database::new runs the migrations, creating the schema
    drop(Database::new(&temp_db.to_string_lossy()).expect("Failed to create database"));

    {
        let conn = rusqlite::Connection::open(&temp_db).expect("Failed to open database");
        conn.execute_batch(
            "INSERT INTO decks (status, total_questions, incorrect_answers, completed_at)
                VALUES ('completed', 3, 3, '2025-11-17 10:00:00');
             INSERT INTO operations (operation_type, operand1, operand2, result, deck_id)
                VALUES ('ADD', 2, 3, 5, 1), ('MULTIPLY', 3, 4, 12, 1);
             INSERT INTO answers (operation_id, user_answer, is_correct, time_spent_seconds, deck_id)
                VALUES (1, 4, 0, 2.0, 1), (1, 6, 0, 3.0, 1), (2, 10, 0, 4.0, 1);",
        )
        .expect("Failed to insert rows");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_performance_stats"))
        .arg(&temp_db)
        .arg("--no-color")
        .output()
        .expect("Failed to execute performance_stats");

    let _ = fs::remove_file(&temp_db);

    assert!(
        output.status.success(),
        "performance_stats command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("No operation types found in the database"),
        "Operations with only incorrect answers should be listed, got: {}",
        stdout
    );
    assert!(stdout.contains("Operation Type: ADD"), "got: {}", stdout);
    assert!(
        stdout.contains("Operation Type: MULTIPLY"),
        "got: {}",
        stdout
    );
    assert!(
        stdout.contains("Global (all time) - 0/2 correct (0.0%)"),
        "got: {}",
        stdout
    );
    assert!(
        stdout.contains("Global (all time) - 0/1 correct (0.0%)"),
        "got: {}",
        stdout
    );
}

/// Test performance_stats with help flag
#[test]
fn test_performance_stats_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "performance_stats", "--", "--help"])
        .output()
        .expect("Failed to execute performance_stats");

    assert!(
        output.status.success(),
        "performance_stats --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzes performance statistics"),
        "Help output should contain description"
    );
    assert!(
        stdout.contains("DATABASE_FILE"),
        "Help output should mention DATABASE_FILE"
    );
    assert!(
        stdout.contains("--no-color"),
        "Help output should mention --no-color flag"
    );
}

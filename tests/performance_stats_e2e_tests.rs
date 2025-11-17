use std::process::Command;

/// End-to-end test for performance_stats binary with colored output
#[test]
fn test_performance_stats_with_color() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "performance_stats",
            "--",
            "data/sample.db",
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
        .args(&[
            "run",
            "--bin",
            "performance_stats",
            "--",
            "data/sample.db",
            "--no-color",
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
        .args(&[
            "run",
            "--bin",
            "performance_stats",
            "--",
            temp_db,
        ])
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

/// Test performance_stats with help flag
#[test]
fn test_performance_stats_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "performance_stats", "--", "--help"])
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

use memory_practice::database_factory::DatabaseFactory;
use memory_practice::gui;
use std::sync::Arc;

#[allow(clippy::arc_with_non_send_sync)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger - can be configured with RUST_LOG environment variable
    // Examples: RUST_LOG=debug, RUST_LOG=info, RUST_LOG=memory_practice=debug
    env_logger::builder().format_timestamp_millis().init();

    // Detect database configuration from command line arguments
    let config = DatabaseFactory::detect_config();
    let is_test_mode = config.is_test_mode;

    // Create database based on detected configuration
    let db = Arc::new(DatabaseFactory::create(config)?);

    // Run the GUI application
    gui::run_app(db, is_test_mode)?;

    Ok(())
}

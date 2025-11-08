use memory_practice::database_factory::DatabaseFactory;
use memory_practice::gui;
use std::sync::Arc;

#[allow(clippy::arc_with_non_send_sync)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect database configuration from command line arguments
    let config = DatabaseFactory::detect_config();
    let is_test_mode = config.is_test_mode;

    // Create database based on detected configuration
    let db = Arc::new(DatabaseFactory::create(config)?);

    // Run the GUI application
    gui::run_app(db, is_test_mode)?;

    Ok(())
}

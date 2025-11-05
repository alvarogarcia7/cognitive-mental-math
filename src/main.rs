use memory_practice::database::Database;
use memory_practice::gui;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = Arc::new(Database::new("memory_practice.db")?);

    // Run the GUI application
    gui::run_app(db)?;

    Ok(())
}

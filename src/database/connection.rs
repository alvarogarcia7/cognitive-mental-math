use log::debug;
use rusqlite::Connection;
use rusqlite::Result;

// Embed migrations from the migrations directory
refinery::embed_migrations!("migrations");

/// Initializes the database connection and runs migrations
pub fn init_connection(db_path: &str) -> Result<Connection> {
    let mut conn = Connection::open(db_path)?;

    // Run embedded migrations from the migrations folder
    match migrations::runner().run(&mut conn) {
        Ok(_) => {
            debug!("Migrations completed successfully");
        }
        Err(e) => {
            eprintln!("Refinery migration error: {}", e);
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
    }

    Ok(conn)
}

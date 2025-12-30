use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::fs;

pub async fn init_db() -> Result<SqlitePool, String> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:assistant.db".to_string());

    // Create database file if it doesn't exist
    if database_url.starts_with("sqlite:") {
        let path = &database_url[7..];
        if !fs::metadata(path).is_ok() {
            fs::File::create(path).map_err(|e| format!("Failed to create db file: {}", e))?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Run migrations (schema)
    let schema = fs::read_to_string("schema.sql")
        .map_err(|e| format!("Failed to read schema.sql: {}", e))?;

    sqlx::query(&schema)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    Ok(pool)
}

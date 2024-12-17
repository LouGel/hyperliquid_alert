use crate::global_data::get_pool;
use sqlx::postgres::PgPool;
use std::error::Error;
use std::ops::Deref;
use std::path::Path;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    SchemaError(String),
    SqlxError(sqlx::Error),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError::SqlxError(err)
    }
}

pub async fn check_and_init_db() -> Result<(), DatabaseError> {
    let pool = get_pool();

    // Check if our tables exist by looking for the 'chat' table
    let table_exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'chat'
        ) as "exists!"
        "#
    )
    .fetch_one(pool.deref())
    .await?
    .expect("Failed to check table existence");

    if !table_exists {
        println!("Initializing database schema...");

        // Read the SQL schema file
        let schema_path = "src/db/sql/bdd.sql";
        if !Path::new(schema_path).exists() {
            return Err(DatabaseError::SchemaError(format!(
                "Schema file not found at: {}",
                schema_path
            )));
        }

        let schema = std::fs::read_to_string(schema_path)
            .map_err(|e| DatabaseError::SchemaError(e.to_string()))?;

        // Execute schema in a transaction
        let mut tx = pool.begin().await?;

        // Split and execute schema (handling potential dollar-quoted blocks)
        for statement in split_sql_statements(&schema) {
            if !statement.trim().is_empty() {
                sqlx::query(&statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e))?;
            }
        }

        // Commit the transaction
        tx.commit().await?;

        println!("Database schema initialized successfully");
    } else {
        println!("Database schema already exists");
    }

    Ok(())
}

// Helper function to split SQL statements while preserving dollar-quoted blocks
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag = String::new();

    for line in sql.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        if !in_dollar_quote {
            // Check for dollar quote start
            if let Some(pos) = trimmed.find("$$") {
                in_dollar_quote = true;
                dollar_quote_tag = trimmed[..pos].to_string();
            }

            // Add line to current statement
            current_statement.push_str(line);
            current_statement.push('\n');

            // If not in dollar quote and line ends with semicolon, split statement
            if !in_dollar_quote && trimmed.ends_with(';') {
                statements.push(current_statement.clone());
                current_statement.clear();
            }
        } else {
            // Add line to current statement
            current_statement.push_str(line);
            current_statement.push('\n');

            // Check for matching dollar quote end
            if trimmed.ends_with(&format!("$$")) {
                in_dollar_quote = false;
                dollar_quote_tag.clear();
            }
        }
    }

    // Add any remaining statement
    if !current_statement.trim().is_empty() {
        statements.push(current_statement);
    }

    statements
}

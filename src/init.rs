use crate::*;
use global_data::POOL;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn init_pool(database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("DB Pool instantiation");

    let pool = Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Set POOL only if it hasn't been initialized yet
    POOL.set(Arc::new(pool))
        .map_err(|_| "Global pool has already been set")?;

    Ok(())
}

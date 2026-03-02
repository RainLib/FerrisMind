use crate::config::AppConfig;
use crate::db::init_db;
use surrealdb_types::Value;

#[tokio::test]
async fn test_cloud_connection() -> anyhow::Result<()> {
    // Explicitly initialize rustls for tests (in case default provider is not set)
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("Loading configuration from environment...");
    let config = AppConfig::from_env()?;

    println!(
        "Attempting to connect to SurrealDB Cloud at {}...",
        config.surreal.addr
    );
    let db = init_db(&config.surreal).await?;

    println!("Connection and authentication successful!");

    // Perform a simple query to verify
    let mut response = db.query("INFO FOR DB").await?;
    let _: Option<Value> = response.take(0)?;

    println!("Query successful!");

    Ok(())
}

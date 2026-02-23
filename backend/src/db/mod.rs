pub mod schema;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::info;

use crate::config::SurrealConfig;

pub type Db = Surreal<Client>;

pub async fn init_db(config: &SurrealConfig) -> anyhow::Result<Db> {
    info!("Connecting to SurrealDB at {}", config.addr);

    let db = Surreal::new::<Ws>(&config.addr).await?;

    db.signin(Root {
        username: &config.user,
        password: &config.pass,
    })
    .await?;

    db.use_ns(&config.ns).use_db(&config.db).await?;

    info!(
        "Connected to SurrealDB (ns={}, db={})",
        config.ns, config.db
    );

    // Run schema migrations
    schema::apply_schema(&db).await?;

    // Ensure mock user exists for development
    let _: Vec<serde_json::Value> = db
        .query("UPSERT user:mock_dev_user SET username = 'dev_user', email = 'dev@example.com', password_hash = 'nopass'")
        .await?
        .take(0)?;

    Ok(db)
}

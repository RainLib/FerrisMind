pub mod schema;

use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::info;

use crate::config::SurrealConfig;

pub type Db = Surreal<Any>;

pub async fn init_db(config: &SurrealConfig) -> anyhow::Result<Db> {
    info!("Connecting to SurrealDB at {}", config.addr);

    // Open a connection as in user snippet
    let db = any::connect(&config.addr).await?;

    // Select namespace and database (using notebooklm values if config is empty, or config values if provided)
    let ns = if config.ns.is_empty() {
        "main"
    } else {
        &config.ns
    };
    let db_name = if config.db.is_empty() {
        "notebooklm"
    } else {
        &config.db
    };

    db.use_ns(ns).use_db(db_name).await?;
    info!("Using SurrealDB (ns={}, db={})", ns, db_name);

    // Authenticate: Priority to Token if provided, else use credentials
    if let Some(token) = &config.token {
        db.authenticate(token).await?;
        info!("Authenticated to SurrealDB using JWT token");
    } else {
        db.signin(Root {
            username: config.user.clone(),
            password: config.pass.clone(),
        })
        .await?;
        info!("Signed in to SurrealDB as {}", config.user);
    }

    // Run schema migrations
    schema::apply_schema(&db).await?;

    // Ensure mock user exists for development
    db.query("UPSERT user:mock_dev_user SET username = 'dev_user', email = 'dev@example.com', password_hash = 'nopass'")
        .await?
        .check()?;

    Ok(db)
}

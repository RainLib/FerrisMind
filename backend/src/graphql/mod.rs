pub mod guard;
pub mod mutation;
pub mod query;
pub mod types;

use async_graphql::{EmptySubscription, Schema};

use crate::config::JwtConfig;
use crate::db::Db;

use mutation::MutationRoot;
use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with database and config injected into context.
pub fn build_schema(db: Db, jwt_config: JwtConfig) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .data(jwt_config)
        .finish()
}

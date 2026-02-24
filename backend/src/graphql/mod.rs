pub mod guard;
pub mod mutation;
pub mod query;
pub mod types;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    Extension,
};

use async_graphql::{EmptySubscription, Schema};

use crate::config::{IngestConfig, JwtConfig};
use crate::db::Db;
use crate::llm::manager::LlmManager;
use std::sync::Arc;

pub use mutation::MutationRoot;
pub use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with database and config injected into context.
pub fn build_schema(
    db: Db,
    jwt_config: JwtConfig,
    llm_manager: Arc<LlmManager>,
    ingest_config: IngestConfig,
) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .data(jwt_config)
        .data(llm_manager)
        .data(ingest_config)
        .finish()
}

pub async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql_handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

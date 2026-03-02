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

use crate::auth::middleware::OptionalAuth;
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

pub async fn graphql_handler(
    schema: Extension<AppSchema>,
    auth: OptionalAuth,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut gql_req = req.into_inner();
    if let Some(claims) = auth.0 {
        gql_req = gql_req.data(claims);
    }
    schema.execute(gql_req).await.into()
}

pub async fn graphiql_handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

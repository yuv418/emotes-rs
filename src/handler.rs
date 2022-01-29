use actix_web::HttpRequest;
use actix_web::{web, HttpResponse};
use async_graphql::Schema;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::graphql_schema::{mutation::Mutation, query::Query};

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/api")))
}

pub async fn api_graphql_handler(
    pool: web::Data<Arc<PgPool>>,
    schema: web::Data<Schema<Query, Mutation, EmptySubscription>>,
    request: HttpRequest,
    graphql_request: GraphQLRequest,
) -> GraphQLResponse {
    // Get the token from the header, convert it to a uesr, and then use the `data` function to populate that for guarding later
    schema.execute(graphql_request.into_inner()).await.into()
}

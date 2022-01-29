use actix_web::HttpRequest;
use actix_web::{web, HttpResponse};
use async_graphql::Response;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema, ServerError,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::types::*;

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
    // Get the token from the header, convert it to a user, and then use the `data` function to populate that for guarding later

    let err_response = |msg| -> Response {
        async_graphql::Response::from_errors(vec![ServerError::new(msg, None)]).into()
    };

    if let Some(token) = request.headers().get("Token") {
        if let Ok(token_str) = token.to_str() {
            if let Ok(Some(emote_user)) =
                SerializedEmoteToken::to_emote_user(Arc::clone(&pool), token_str).await
            {
                let graphql_request = graphql_request.into_inner().data(emote_user);
                schema.execute(graphql_request).await.into()
            } else {
                err_response("Token is invalid; unauthorized").into()
            }
        } else {
            err_response("Token is in invalid format; failed to convert to string").into()
        }
    } else {
        err_response("Missing required token field").into()
    }
}

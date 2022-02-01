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
use log::info;

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
    } else if *crate::graphql_schema::guards::FIRST_RUN.read().unwrap() {
        // Handle first-run mode existing. There will be no emote user, which means you can only create a user.
        schema.execute(graphql_request.into_inner()).await.into()
    } else {
        err_response("Missing required token field").into()
    }
}
pub async fn emote_display_handler(
    request: HttpRequest,
    pool: web::Data<Arc<PgPool>>,
) -> HttpResponse {
    let dir_slug = request.match_info().get("dir_slug").unwrap();
    let emote_slug = request.match_info().get("emote_slug").unwrap();
    let options = request.match_info().get("options").map(|x| x.to_owned());
    emote_display(pool, dir_slug.to_owned(), emote_slug.to_owned(), options).await
}

async fn emote_display(
    pool: web::Data<Arc<PgPool>>,
    dir_slug: String,
    emote_slug: String,
    options: Option<String>,
) -> HttpResponse {
    use std::io::Read;
    info!(
        "requested an emote:\n\tdir_slug: {}\n\temote_slug: {}\n\toptions: {:?}",
        dir_slug, emote_slug, options
    );

    // Discord only renders gifs if the URL ends with .gif
    let emote_slug = if options.is_none() && emote_slug.ends_with(".gif") {
        emote_slug.trim_end_matches(".gif").to_owned()
    } else {
        emote_slug
    };

    let options = if let Some(options) = options {
        if options.ends_with(".gif") {
            Some(options.trim_end_matches(".gif").to_owned())
        } else {
            Some(options)
        }
    } else {
        options
    };

    if let Ok(Some(emote)) = Emote::by_slug(Arc::clone(&pool), dir_slug + "/" + &emote_slug).await {
        let (mut width, mut height, mut multiplier) = match emote.emote_type {
            EmoteType::Standard => (64, None, 1), // height is automatic
            EmoteType::Sticker => (256, None, 1),
        };

        // Parse options
        // TODO rework this
        if let Some(options) = options {
            let options: Vec<&str> = options.split("x").collect();
            if options.len() == 1 {
                // just width
                width = options[0].parse().unwrap();
            } else if options.len() == 2 {
                if options[0] == "" {
                    // multiplier format: "x10"
                    multiplier = options[1].parse().unwrap();
                } else {
                    width = options[0].parse().unwrap();
                    // you could do 64xx10 and that would omit height
                    height = if options[1] != "" {
                        Some(options[1].parse().unwrap())
                    } else {
                        None
                    };
                }
            } else if options.len() == 3 {
                width = options[0].parse().unwrap();
                height = Some(options[1].parse().unwrap());
                multiplier = options[2].parse().unwrap();
            }
        }

        // this is a "for now" thing TODO delete this part when we implement resizing VipsImages by height and width
        if let Some(_) = height {
            return HttpResponse::InternalServerError().json(EmoteMsg::new(
                    "Emotes cannot be resized by height yet. Please try again without the height, or wait for this feature to be implemented."
                ));
        }
        // right now, multiplier does nothing

        let corresponding_emote_image =
            EmoteImage::by_emote_and_size(Arc::clone(&pool), emote.uuid, width, height).await;
        if let Ok(None) = corresponding_emote_image {
            if let Ok(true) =
                EmoteImage::resize_image(Arc::clone(&pool), emote.uuid, width, height).await
            {
                return HttpResponse::NotFound().json(EmoteMsg::new(
                    "Emote was not created in that size. Emote resizer dispatched.",
                ));
            };
        } else if let Ok(Some(image)) = corresponding_emote_image {
            if image.processing {
                return HttpResponse::NotFound()
                    .json(EmoteMsg::new("Emote resizer is processing this emote."));
            }

            return HttpResponse::Ok().content_type(&*image.content_type).body(
                if let Ok(emote_bytes) = image.get_emote_bytes() {
                    emote_bytes
                } else {
                    return HttpResponse::InternalServerError().json(EmoteMsg::new(
                        "Failed to open file for emote. You should delete this emote.",
                    ));
                },
            );
        }
    }

    HttpResponse::NotFound().json(EmoteMsg::new("Emote not found")) // TODO use JSON
}

use serde::Serialize;

#[derive(Serialize)]
pub struct EmoteMsg {
    msg: String,
}
impl EmoteMsg {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}

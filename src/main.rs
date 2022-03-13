use actix_cors::Cors;
use actix_web::{
    guard,
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
use anyhow::{Context, Result};
use async_graphql::EmptySubscription;
use async_graphql::Schema;
use dotenv::dotenv;
use log::*;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod config;
mod graphql_schema;
mod handler;
mod image;
mod storage;
mod types;

use config::EMOTES_CONFIG;

// A lot of this is copied from attendance-rs
#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_pool = Arc::new(
        PgPoolOptions::new()
            // TODO make this variable configurable in the JSON file
            .max_connections(EMOTES_CONFIG.db_max_connections)
            .connect(&EMOTES_CONFIG.db_url)
            .await
            .with_context(|| "Failed to connect to PostgreSQL.")?,
    );

    // Do any left-over migrations
    sqlx::migrate!()
        .run(&*db_pool)
        .await
        .with_context(|| "Failed to migrate the database!")?;

    // Configure first-run mode by checking how many administrator users **with tokens** are in the database
    if let Some(count) = sqlx::query!(
        "SELECT COUNT(emote_token.uuid) as count FROM emote_token INNER JOIN emote_user ON emote_user.uuid = emote_token.emote_user_uuid WHERE emote_user.administrator = ($1)",
        true
    )
    .fetch_one(&*db_pool)
    .await?
    .count
    {
        if count < 1 {
            info!("There are no admin user tokens in the database. Enabling first-run mode.");
            *graphql_schema::guards::FIRST_RUN.write().unwrap() = true;
        } else {
            info!("Disabling first-run mode, there are emote users in the database.");
        }
    }

    info!("Database connection success, starting up actix");

    let schema = Schema::build(
        graphql_schema::query::Query,
        graphql_schema::mutation::Mutation,
        EmptySubscription,
    )
    .data(Arc::clone(&db_pool))
    .finish();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec![
                "GET", "HEAD", "POST", "OPTIONS", "PUT", "PATCH", "DELETE",
            ])
            .supports_credentials()
            .allowed_headers(vec!["Token", "X-Apollo-Tracing"])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(1800);

        App::new()
            .app_data(web::Data::new(Arc::clone(&db_pool)))
            .app_data(web::Data::new(schema.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .service(
                web::resource("/api")
                    .guard(guard::Post())
                    .to(handler::api_graphql_handler),
            )
            .service(
                web::resource("/playground")
                    .guard(guard::Get())
                    .to(handler::graphql_playground),
            )
            .service(
                web::resource([
                    "/{dir_slug}/{emote_slug}",
                    "/{dir_slug}/{emote_slug}/{options}",
                ])
                .guard(guard::Get())
                .to(handler::emote_display_handler),
            )
        // TODO add one for actually getting the emotes
    })
    .bind(&EMOTES_CONFIG.http_bind)
    .with_context(|| "Failed to start the GraphQL HTTP server")?
    .run()
    .await?;

    Ok(())
}

use actix_cors::Cors;
use actix_web::{guard, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::{Context, Result};
use async_graphql::EmptySubscription;
use async_graphql::Schema;
use dotenv::dotenv;
use lazy_static::lazy_static;
use log::*;
use sqlx::postgres::PgPoolOptions;
use std::fs::File;
use std::sync::Arc;

mod config;
mod graphql_schema;
mod handler;
mod types;

lazy_static! {
    static ref EMOTES_CONFIG: config::EmotesConfig = serde_json::from_reader(
        File::open(
            &dotenv::var("EMOTES_CONFIG_FILE")
                .with_context(|| "Failed to read emotes config file env-var")
                .unwrap()
        )
        .with_context(|| "Failed to open specified emotes config file")
        .unwrap()
    )
    .with_context(|| "Failed to parse specified emotes config file")
    .unwrap();
}

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
            .data(Arc::clone(&db_pool))
            .data(schema.clone())
            .wrap(cors)
            .wrap(Logger::default())
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
        // TODO add one for actually getting the emotes
    })
    .bind(&EMOTES_CONFIG.http_bind)
    .with_context(|| "Failed to start the GraphQL HTTP server")?
    .run()
    .await;

    Ok(())
}

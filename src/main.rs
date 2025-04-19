mod config;
mod controllers;
mod models;
mod schema;

use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controllers::post_tweet;
use dotenv::dotenv;
use std::env;

#[derive(OpenApi)]
#[openapi(
    paths(controllers::tweet_controller::post_tweet),
    components(schemas(models::Post, models::CreatePostRequest, models::User))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Set up database connection pool
    let pool = crate::config::db::establish_connection_pool();

    // Optional: Log the port we're running on
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_address = format!("{}:{}", host, port);

    println!("Starting server at {}", bind_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add database connection pool to app state
            .app_data(web::Data::new(pool.clone()))
            .service(post_tweet)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}

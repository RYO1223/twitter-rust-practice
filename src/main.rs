mod config;
mod controllers;
mod models;
mod schema;

use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controllers::post_controller::{
    create_post, delete_post, get_all_posts, get_post_by_id, update_post,
};
use dotenv::dotenv;
use std::env;

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::post_controller::get_all_posts,
        controllers::post_controller::get_post_by_id,
        controllers::post_controller::create_post,
        controllers::post_controller::update_post,
        controllers::post_controller::delete_post
    ),
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
            .service(get_all_posts)
            .service(get_post_by_id)
            .service(create_post)
            .service(update_post)
            .service(delete_post)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}

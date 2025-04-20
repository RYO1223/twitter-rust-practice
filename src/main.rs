mod controllers;
mod middlewares;
mod models;
mod schema;
mod util;

use actix_web::{App, HttpServer, middleware::Logger, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controllers::auth_controller::{login, register};
use controllers::post_controller::{
    create_post, delete_post, get_all_posts, get_post_by_id, update_post,
};
use dotenv::dotenv;
use middlewares::auth::AuthMiddleware;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Set up database connection pool
    let pool = util::db::establish_connection_pool();

    // Optional: Log the port we're running on
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_address = format!("{}:{}", host, port);

    println!("Starting server at {}", bind_address);

    // Start HTTP server
    HttpServer::new(move || {
        // Create auth middleware with ignored routes
        let auth_middleware = AuthMiddleware::new()
            .ignore("/auth/register")
            .ignore("/auth/login")
            .ignore("/swagger-ui")
            .ignore("/api-docs/openapi.json");

        App::new()
            // Add database connection pool to app state
            .app_data(web::Data::new(pool.clone()))
            // Add logging middleware
            .wrap(Logger::default())
            // Public routes (no auth required)
            .service(web::scope("/auth").service(register).service(login))
            // Public endpoint to get all posts
            .service(get_all_posts)
            .service(get_post_by_id)
            // Protected routes (auth required)
            .wrap(auth_middleware) // Apply auth middleware with ignore routes
            .service(create_post)
            .service(update_post)
            .service(delete_post)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", util::api_doc::ApiDoc::openapi()),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}

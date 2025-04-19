mod controllers;
mod models;

use actix_web::{App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controllers::post_tweet;

#[derive(OpenApi)]
#[openapi(
    paths(controllers::tweet_controller::post_tweet),
    components(schemas(models::Post))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(post_tweet).service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

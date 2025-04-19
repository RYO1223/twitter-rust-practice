use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, OpenApi)]
struct Post {
    user: String,
    content: String,
}

async fn post_tweet(post: web::Json<Post>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "user": post.user,
        "content": post.content,
    }))
}

#[utoipa::path(
    post,
    path = "/post_tweet",
    request_body = Post,
    responses(
        (status = 200, description = "Tweet posted successfully", body = Post)
    )
)]
async fn post_tweet(post: web::Json<Post>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "user": post.user,
        "content": post.content,
    }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let openapi = Post::openapi();

    HttpServer::new(move || {
        App::new()
            .route("/post_tweet", web::post().to(post_tweet))
            .service(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

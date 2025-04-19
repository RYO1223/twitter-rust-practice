use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, ToSchema)]
struct Post {
    user: String,
    content: String,
}

#[utoipa::path(
    post,
    path = "/post_tweet",
    responses(
        (status = 200, description = "Tweet posted successfully", body = Post),
        (status = 400, description = "Invalid input")
    )
)]
#[post("/post_tweet")]
async fn post_tweet(post: web::Json<Post>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "user": post.user,
        "content": post.content,
    }))
}

#[derive(OpenApi)]
#[openapi(paths(post_tweet), components(schemas(Post)))]
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

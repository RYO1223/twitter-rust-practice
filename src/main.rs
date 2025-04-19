use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;

#[derive(Serialize, Deserialize)]
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/post_tweet", web::post().to(post_tweet))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

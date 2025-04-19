use actix_web::{HttpResponse, Responder, post, web};
use serde_json::json;

use crate::models::Post;

#[utoipa::path(
    post,
    path = "/post_tweet",
    request_body = Post,
    responses(
        (status = 200, description = "Tweet posted successfully", body = Post),
        (status = 400, description = "Invalid input")
    )
)]
#[post("/post_tweet")]
pub async fn post_tweet(post: web::Json<Post>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "user": post.user,
        "content": post.content,
    }))
}

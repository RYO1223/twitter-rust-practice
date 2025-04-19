use actix_web::{HttpResponse, Responder, post, web};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde_json::json;

use crate::config::db::DbPool;
use crate::models::{CreatePostRequest, NewPost, NewUser, Post, User};
use crate::schema::{posts, users};

#[utoipa::path(
    post,
    path = "/post_tweet",
    request_body = CreatePostRequest,
    responses(
        (status = 200, description = "Tweet posted successfully", body = Post),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Server error")
    )
)]
#[post("/post_tweet")]
pub async fn post_tweet(
    pool: web::Data<DbPool>,
    post_req: web::Json<CreatePostRequest>,
) -> impl Responder {
    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        // Find or create user
        let user = users::table
            .filter(users::username.eq(&post_req.username))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|_| "Database error finding user")?
            .unwrap_or_else(|| {
                // Create new user if not found
                let new_user = NewUser {
                    username: post_req.username.clone(),
                };

                diesel::insert_into(users::table)
                    .values(&new_user)
                    .get_result::<User>(&mut conn)
                    .map_err(|_| "Failed to create user")
                    .unwrap()
            });

        // Create new post
        let new_post = NewPost {
            user_id: user.id,
            content: post_req.content.clone(),
        };

        // Insert post into database
        let post = diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(&mut conn)
            .map_err(|_| "Failed to create post")?;

        Ok::<_, &'static str>((user, post))
    })
    .await;

    match result {
        Ok(Ok((user, post))) => HttpResponse::Ok().json(json!({
            "id": post.id,
            "user_id": post.user_id,
            "username": user.username,
            "content": post.content,
            "created_at": post.created_at,
        })),
        Ok(Err(e)) => HttpResponse::BadRequest().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

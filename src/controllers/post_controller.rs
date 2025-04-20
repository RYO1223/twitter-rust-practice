use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::models::{
    post::{NewPost, Post},
    user::{AuthedUserId, User},
};
use crate::schema::{posts, users};
use crate::util::db::DbPool;

/// Get all posts
#[utoipa::path(
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of all posts", body = Vec<Post>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    )
)]
#[get("/posts")]
pub async fn get_all_posts(pool: web::Data<DbPool>) -> impl Responder {
    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        // Fetch all posts with user information
        let posts_result = posts::table
            .load::<Post>(&mut conn)
            .map_err(|_| "Failed to load posts")?;

        // For each post, get the user information
        let mut posts_with_users = Vec::new();
        for post in posts_result {
            let user = users::table
                .find(post.user_id)
                .first::<User>(&mut conn)
                .map_err(|_| "Failed to find user")?;

            posts_with_users.push((post, user));
        }

        Ok::<_, &'static str>(posts_with_users)
    })
    .await;

    match result {
        Ok(Ok(posts_with_users)) => {
            // Convert to JSON response
            let response_data = posts_with_users
                .into_iter()
                .map(|(post, user)| {
                    json!({
                        "id": post.id,
                        "user_id": post.user_id,
                        "username": user.username,
                        "content": post.content,
                        "created_at": post.created_at,
                    })
                })
                .collect::<Vec<_>>();

            HttpResponse::Ok().json(response_data)
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

/// Get a post by ID
#[utoipa::path(
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "The post was found", body = Post),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Server error")
    )
)]
#[get("/posts/{id}")]
pub async fn get_post_by_id(pool: web::Data<DbPool>, id: web::Path<i32>) -> impl Responder {
    let post_id = id.into_inner();

    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        // Find the post by ID
        let post = posts::table
            .find(post_id)
            .first::<Post>(&mut conn)
            .optional()
            .map_err(|_| "Database error finding post")?;

        // If post is found, get the user information
        if let Some(post) = post {
            let user = users::table
                .find(post.user_id)
                .first::<User>(&mut conn)
                .map_err(|_| "Failed to find user")?;

            Ok::<_, &'static str>(Some((post, user)))
        } else {
            Ok(None)
        }
    })
    .await;

    match result {
        Ok(Ok(Some((post, user)))) => HttpResponse::Ok().json(json!({
            "id": post.id,
            "user_id": post.user_id,
            "username": user.username,
            "content": post.content,
            "created_at": post.created_at,
        })),
        Ok(Ok(None)) => HttpResponse::NotFound().json(json!({
            "error": "Post not found"
        })),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

/// Used for API requests when creating a new post
#[derive(Deserialize, ToSchema)]
pub struct CreatePostRequest {
    /// Content of the tweet
    pub content: String,
}

/// Create a new post
#[utoipa::path(
    request_body = CreatePostRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Post created successfully", body = Post),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    )
)]
#[post("/posts")]
pub async fn create_post(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    post_req: web::Json<CreatePostRequest>,
) -> impl Responder {
    let user_id = req.extensions().get::<AuthedUserId>().unwrap().0;

    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        // Create new post
        let new_post = NewPost {
            user_id: user_id,
            content: post_req.content.clone(),
        };

        // Insert post into database
        let post = diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(&mut conn)
            .map_err(|_| "Failed to create post")?;

        Ok::<_, &'static str>(post)
    })
    .await;

    match result {
        Ok(Ok(post)) => HttpResponse::Created().json(post),
        Ok(Err(e)) => HttpResponse::BadRequest().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

/// Update an existing post
#[utoipa::path(
    request_body = CreatePostRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Post updated successfully", body = Post),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Server error")
    )
)]
#[put("/posts/{id}")]
pub async fn update_post(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
    post_req: web::Json<CreatePostRequest>,
) -> impl Responder {
    let post_id = id.into_inner();

    let user_id = req.extensions().get::<AuthedUserId>().unwrap().0;

    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        let post = posts::table
            .find(post_id)
            .filter(posts::user_id.eq(user_id))
            .first::<Post>(&mut conn)
            .optional()
            .map_err(|_| "Database error finding post")?;

        if post.is_none() {
            return Ok::<_, &'static str>(None);
        }
        let post = post.unwrap();

        // Update post
        let updated_post = diesel::update(&post)
            .set(posts::content.eq(&post_req.content))
            .get_result::<Post>(&mut conn)
            .map_err(|_| "Failed to update post")?;

        Ok::<Option<Post>, &'static str>(Some(updated_post))
    })
    .await;

    match result {
        Ok(Ok(Some(post))) => HttpResponse::Ok().json(post),
        Ok(Ok(None)) => HttpResponse::NotFound().json(json!({
            "error": "Post not found"
        })),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

/// Delete a post
#[utoipa::path(
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 204, description = "Post deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Server error")
    )
)]
#[delete("/posts/{id}")]
pub async fn delete_post(pool: web::Data<DbPool>, id: web::Path<i32>) -> impl Responder {
    let post_id = id.into_inner();

    // Use a web::block to offload database operations to a separate thread
    let result = web::block(move || {
        let mut conn = pool.get().map_err(|_| "Failed to get DB connection")?;

        // Check if post exists
        let post_exists = posts::table
            .find(post_id)
            .first::<Post>(&mut conn)
            .optional()
            .map_err(|_| "Database error checking post")?;

        if post_exists.is_none() {
            return Ok::<_, &'static str>(false);
        }

        // Delete post
        let deleted = diesel::delete(posts::table.find(post_id))
            .execute(&mut conn)
            .map_err(|_| "Failed to delete post")?;

        Ok(deleted > 0)
    })
    .await;

    match result {
        Ok(Ok(true)) => HttpResponse::NoContent().finish(),
        Ok(Ok(false)) => HttpResponse::NotFound().json(json!({
            "error": "Post not found"
        })),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

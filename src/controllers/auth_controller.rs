use actix_web::{HttpResponse, Responder, post, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::util::db::DbPool;
use crate::middlewares::auth::Authentication;
use crate::models::user::{NewUser, User};

#[derive(Deserialize, Serialize,ToSchema)]
#[schema(
    example = json!({
        "username": "johndoe",
        "password": "password123"
    })
)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
#[schema(
    example = json!({
        "username": "johndoe",
        "password": "password123"
    })
)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    user_data: web::Json<RegisterRequest>,
) -> impl Responder {
    use crate::schema::users;

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection error"),
    };

    // Hash the password
    let password_hash = match hash(&user_data.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Password hashing failed"),
    };

    let new_user = NewUser {
        username: user_data.username.clone(),
        password_hash,
    };

    // Create the user in the database
    let user: User = match web::block(move || {
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(&mut conn)
    })
    .await
    {
        Ok(Ok(user)) => user,
        Ok(Err(_)) => return HttpResponse::InternalServerError().body("User creation failed"),
        Err(_) => {
            return HttpResponse::InternalServerError().body("User creation operation failed");
        }
    };

    // Generate a token for the newly registered user
    let token = match Authentication::create_token(user.id, &user.username) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().body("Token generation failed"),
    };

    // Return the token and user information
    HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
    })
}

/// Login an existing user
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid username or password"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[post("/login")]
pub async fn login(pool: web::Data<DbPool>, login_data: web::Json<LoginRequest>) -> impl Responder {
    use crate::schema::users::dsl::*;

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection error"),
    };

    // Store password before moving login_data
    let password = login_data.password.clone();
    let username_clone = login_data.username.clone();

    // Find the user by username
    let user_result = web::block(move || {
        users
            .filter(username.eq(&username_clone))
            .first::<User>(&mut conn)
            .optional()
    })
    .await;

    let user = match user_result {
        Ok(Ok(Some(user))) => user,
        Ok(Ok(None)) => return HttpResponse::Unauthorized().body("Invalid username or password"),
        Ok(Err(_)) => return HttpResponse::InternalServerError().body("Database error"),
        Err(_) => return HttpResponse::InternalServerError().body("Operation failed"),
    };

    // Verify password
    let password_matches = match verify(&password, &user.password_hash) {
        Ok(matches) => matches,
        Err(_) => return HttpResponse::InternalServerError().body("Password verification failed"),
    };

    if !password_matches {
        return HttpResponse::Unauthorized().body("Invalid username or password");
    }

    // Generate JWT token
    let token = match Authentication::create_token(user.id, &user.username) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().body("Token generation failed"),
    };

    // Return the token and user information
    HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
    })
}

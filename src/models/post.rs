use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::user::User;
use crate::schema::posts;

/// Represents a tweet post with user information and content in the database
#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations, Debug, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "user_id": 1,
    "content": "Hello world from Rust!", 
    "created_at": "2025-04-19T07:30:00"
}))]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct Post {
    /// Unique identifier for the post
    pub id: i32,
    /// ID of the user who created the post
    pub user_id: i32,
    /// Content of the tweet
    pub content: String,
    /// Timestamp when the post was created
    #[schema(value_type = String, format = "date-time", example = "2025-04-19T07:30:00")]
    pub created_at: NaiveDateTime,
}

/// Used for creating new posts in the database
#[derive(Deserialize, Insertable, ToSchema)]
#[diesel(table_name = posts)]
pub struct NewPost {
    /// ID of the user creating the post
    pub user_id: i32,
    /// Content of the tweet
    pub content: String,
}

use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema::users;

/// Represents a user in the database
#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "username": "johndoe", 
    "created_at": "2025-04-19T07:30:00"
}))]
#[diesel(table_name = users)]
pub struct User {
    /// Unique identifier for the user
    pub id: i32,
    /// Username of the user
    pub username: String,
    /// Timestamp when the user was created
    #[schema(value_type = String, format = "date-time", example = "2025-04-19T07:30:00")]
    pub created_at: NaiveDateTime,
}

/// Used for creating new users in the database
#[derive(Deserialize, Insertable, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    /// Username for the new user
    pub username: String,
}
